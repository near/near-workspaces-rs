use std::collections::HashMap;
use std::fmt::Debug;

use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

use near_jsonrpc_client::methods::query::RpcQueryRequest;
use near_jsonrpc_client::{methods, JsonRpcClient, MethodCallResult};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::account::{AccessKey, AccessKeyPermission};
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::{
    Action, AddKeyAction, CreateAccountAction, DeleteAccountAction, DeployContractAction,
    FunctionCallAction, SignedTransaction, TransferAction,
};
use near_primitives::types::{Balance, BlockId, Finality, Gas, StoreKey};
use near_primitives::views::{
    AccessKeyView, AccountView, ContractCodeView, FinalExecutionOutcomeView, QueryRequest,
};

use crate::network::ViewResultDetails;
use crate::rpc::tool;
use crate::types::{AccountId, InMemorySigner, PublicKey, Signer};

pub(crate) const DEFAULT_CALL_FN_GAS: Gas = 10_000_000_000_000;
pub(crate) const DEFAULT_CALL_DEPOSIT: Balance = 0;
const ERR_INVALID_VARIANT: &str =
    "Incorrect variant retrieved while querying: maybe a bug in RPC code?";

/// A client that wraps around JsonRpcClient, and provides more capabilities such
/// as retry w/ exponential backoff and utility functions for sending transactions.
pub struct Client {
    rpc_addr: String,
}

impl Client {
    pub(crate) fn new(rpc_addr: String) -> Self {
        Self { rpc_addr }
    }

    pub(crate) async fn query_broadcast_tx(
        &self,
        method: &methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest,
    ) -> MethodCallResult<
        FinalExecutionOutcomeView,
        near_jsonrpc_primitives::types::transactions::RpcTransactionError,
    > {
        retry(|| async {
            // A new client is required since using a shared one between different threads can
            // cause https://github.com/hyperium/hyper/issues/2112
            let result = JsonRpcClient::new_client()
                .connect(&self.rpc_addr)
                .call(method)
                .await;
            match &result {
                Ok(response) => {
                    // When user sets logging level to INFO we only print one-liners with submitted
                    // actions and the resulting status. If the level is DEBUG or lower, we print
                    // the entire request and response structures.
                    if tracing::level_enabled!(tracing::Level::DEBUG) {
                        tracing::debug!(
                            target: "workspaces",
                            "Calling RPC method {:?} succeeded with {:?}",
                            method,
                            response
                        );
                    } else {
                        tracing::info!(
                            target: "workspaces",
                            "Submitting transaction with actions {:?} succeeded with status {:?}",
                            method.signed_transaction.transaction.actions,
                            response.status
                        );
                    }
                }
                Err(error) => {
                    tracing::error!(
                        target: "workspaces",
                        "Calling RPC method {:?} resulted in error {:?}",
                        method,
                        error
                    );
                }
            };
            result
        })
        .await
    }

    pub(crate) async fn query<M>(&self, method: &M) -> MethodCallResult<M::Response, M::Error>
    where
        M: methods::RpcMethod + Debug,
        M::Response: Debug,
        M::Error: Debug,
    {
        retry(|| async {
            // A new client is required since using a shared one between different threads can
            // cause https://github.com/hyperium/hyper/issues/2112
            let result = JsonRpcClient::new_client()
                .connect(&self.rpc_addr)
                .call(method)
                .await;
            tracing::debug!(
                target: "workspaces",
                "Querying RPC with {:?} resulted in {:?}",
                method,
                result
            );
            result
        })
        .await
    }

    async fn send_tx_and_retry(
        &self,
        signer: &InMemorySigner,
        receiver_id: &AccountId,
        action: Action,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        send_batch_tx_and_retry(self, signer, receiver_id, vec![action]).await
    }

    pub(crate) async fn call(
        &self,
        signer: &InMemorySigner,
        contract_id: &AccountId,
        method_name: String,
        args: Vec<u8>,
        gas: Gas,
        deposit: Balance,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        self.send_tx_and_retry(
            signer,
            contract_id,
            FunctionCallAction {
                args,
                method_name,
                gas,
                deposit,
            }
            .into(),
        )
        .await
    }

    pub(crate) async fn view(
        &self,
        contract_id: AccountId,
        method_name: String,
        args: Vec<u8>,
    ) -> anyhow::Result<ViewResultDetails> {
        let query_resp = self
            .query(&RpcQueryRequest {
                block_reference: Finality::None.into(), // Optimisitic query
                request: QueryRequest::CallFunction {
                    account_id: contract_id,
                    method_name,
                    args: args.into(),
                },
            })
            .await?;

        match query_resp.kind {
            QueryResponseKind::CallResult(result) => Ok(result.into()),
            _ => anyhow::bail!(ERR_INVALID_VARIANT),
        }
    }

    pub(crate) async fn view_state(
        &self,
        contract_id: AccountId,
        prefix: Option<StoreKey>,
    ) -> anyhow::Result<HashMap<String, Vec<u8>>> {
        self.view_state_raw(contract_id, prefix, None)
            .await?
            .into_iter()
            .map(|(k, v)| Ok((String::from_utf8(k)?, v.to_vec())))
            .collect()
    }

    pub(crate) async fn view_state_raw(
        &self,
        contract_id: AccountId,
        prefix: Option<StoreKey>,
        block_id: Option<BlockId>,
    ) -> anyhow::Result<HashMap<Vec<u8>, Vec<u8>>> {
        let block_reference = block_id
            .map(Into::into)
            .unwrap_or_else(|| Finality::None.into());

        let query_resp = self
            .query(&methods::query::RpcQueryRequest {
                block_reference,
                request: QueryRequest::ViewState {
                    account_id: contract_id,
                    prefix: prefix.clone().unwrap_or_else(|| vec![].into()),
                },
            })
            .await?;

        match query_resp.kind {
            QueryResponseKind::ViewState(state) => tool::into_state_map(&state.values),
            _ => anyhow::bail!(ERR_INVALID_VARIANT),
        }
    }

    pub(crate) async fn view_account(
        &self,
        account_id: AccountId,
        block_id: Option<BlockId>,
    ) -> anyhow::Result<AccountView> {
        let block_reference = block_id
            .map(Into::into)
            .unwrap_or_else(|| Finality::None.into());

        let query_resp = self
            .query(&methods::query::RpcQueryRequest {
                block_reference,
                request: QueryRequest::ViewAccount { account_id },
            })
            .await?;

        match query_resp.kind {
            QueryResponseKind::ViewAccount(account) => Ok(account),
            _ => anyhow::bail!(ERR_INVALID_VARIANT),
        }
    }

    pub(crate) async fn view_code(
        &self,
        account_id: AccountId,
        block_id: Option<BlockId>,
    ) -> anyhow::Result<ContractCodeView> {
        let block_reference = block_id
            .map(Into::into)
            .unwrap_or_else(|| Finality::None.into());

        let query_resp = self
            .query(&methods::query::RpcQueryRequest {
                block_reference,
                request: QueryRequest::ViewCode { account_id },
            })
            .await?;

        match query_resp.kind {
            QueryResponseKind::ViewCode(code) => Ok(code),
            _ => anyhow::bail!(ERR_INVALID_VARIANT),
        }
    }

    pub(crate) async fn deploy(
        &self,
        signer: &InMemorySigner,
        contract_id: &AccountId,
        wasm: Vec<u8>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        self.send_tx_and_retry(
            signer,
            contract_id,
            DeployContractAction { code: wasm }.into(),
        )
        .await
    }

    // TODO: write tests that uses transfer_near
    pub(crate) async fn transfer_near(
        &self,
        signer: &InMemorySigner,
        receiver_id: &AccountId,
        amount_yocto: Balance,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        self.send_tx_and_retry(
            signer,
            receiver_id,
            TransferAction {
                deposit: amount_yocto,
            }
            .into(),
        )
        .await
    }

    pub(crate) async fn create_account(
        &self,
        signer: &InMemorySigner,
        new_account_id: &AccountId,
        new_account_pk: PublicKey,
        amount: Balance,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        send_batch_tx_and_retry(
            self,
            signer,
            new_account_id,
            vec![
                CreateAccountAction {}.into(),
                AddKeyAction {
                    public_key: new_account_pk.into(),
                    access_key: AccessKey {
                        nonce: 0,
                        permission: AccessKeyPermission::FullAccess,
                    },
                }
                .into(),
                TransferAction { deposit: amount }.into(),
            ],
        )
        .await
    }

    pub(crate) async fn create_account_and_deploy(
        &self,
        signer: &InMemorySigner,
        new_account_id: &AccountId,
        new_account_pk: PublicKey,
        amount: Balance,
        code: Vec<u8>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        send_batch_tx_and_retry(
            self,
            signer,
            new_account_id,
            vec![
                CreateAccountAction {}.into(),
                AddKeyAction {
                    public_key: new_account_pk.into(),
                    access_key: AccessKey {
                        nonce: 0,
                        permission: AccessKeyPermission::FullAccess,
                    },
                }
                .into(),
                TransferAction { deposit: amount }.into(),
                DeployContractAction { code }.into(),
            ],
        )
        .await
    }

    // TODO: write tests that uses delete_account
    pub(crate) async fn delete_account(
        &self,
        signer: &InMemorySigner,
        account_id: &AccountId,
        beneficiary_id: &AccountId,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let beneficiary_id = beneficiary_id.to_owned();
        self.send_tx_and_retry(
            signer,
            account_id,
            DeleteAccountAction { beneficiary_id }.into(),
        )
        .await
    }
}

pub(crate) async fn access_key(
    client: &Client,
    account_id: near_primitives::account::id::AccountId,
    public_key: near_crypto::PublicKey,
) -> anyhow::Result<(AccessKeyView, CryptoHash)> {
    let query_resp = client
        .query(&methods::query::RpcQueryRequest {
            block_reference: Finality::None.into(),
            request: QueryRequest::ViewAccessKey {
                account_id,
                public_key,
            },
        })
        .await?;

    match query_resp.kind {
        QueryResponseKind::AccessKey(access_key) => Ok((access_key, query_resp.block_hash)),
        _ => Err(anyhow::anyhow!("Could not retrieve access key")),
    }
}

pub(crate) async fn retry<R, E, T, F>(task: F) -> T::Output
where
    F: FnMut() -> T,
    T: core::future::Future<Output = Result<R, E>>,
{
    // Exponential backoff starting w/ 10ms for maximum retry of 5 times:
    let retry_strategy = ExponentialBackoff::from_millis(10).map(jitter).take(5);

    Retry::spawn(retry_strategy, task).await
}

pub(crate) async fn send_tx(
    client: &Client,
    tx: SignedTransaction,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    client
        .query_broadcast_tx(&methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: tx,
        })
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub(crate) async fn send_tx_and_retry<T, F>(
    client: &Client,
    task: F,
) -> anyhow::Result<FinalExecutionOutcomeView>
where
    F: Fn() -> T,
    T: core::future::Future<Output = anyhow::Result<SignedTransaction>>,
{
    retry(|| async { send_tx(client, task().await?).await }).await
}

async fn send_batch_tx_and_retry(
    client: &Client,
    signer: &InMemorySigner,
    receiver_id: &AccountId,
    actions: Vec<Action>,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    send_tx_and_retry(client, || async {
        let (AccessKeyView { nonce, .. }, block_hash) = access_key(
            client,
            signer.inner().account_id.clone(),
            signer.inner().public_key(),
        )
        .await?;

        Ok(SignedTransaction::from_actions(
            nonce + 1,
            signer.inner().account_id.clone(),
            receiver_id.clone(),
            signer.inner() as &dyn Signer,
            actions.clone(),
            block_hash,
        ))
    })
    .await
}
