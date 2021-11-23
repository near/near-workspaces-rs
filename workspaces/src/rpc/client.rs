// TODO: Remove this when near-jsonrpc-client crate no longer defaults to deprecation for
//       warnings about unstable API.
#![allow(deprecated)]

use std::collections::HashMap;

use near_jsonrpc_client::methods::query::RpcQueryRequest;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::account::{AccessKey, AccessKeyPermission};
use near_primitives::hash::CryptoHash;
use near_primitives::types::{AccountId, Finality, FunctionArgs, StoreKey};
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

use near_crypto::{InMemorySigner, PublicKey, Signer};
use near_jsonrpc_client::{methods, JsonRpcClient, JsonRpcMethodCallResult};
use near_primitives::transaction::{
    Action, AddKeyAction, CreateAccountAction, DeleteAccountAction, DeployContractAction,
    FunctionCallAction, SignedTransaction, TransferAction,
};
use near_primitives::types::Balance;
use near_primitives::views::{AccessKeyView, FinalExecutionOutcomeView, QueryRequest};

use crate::rpc::tool;
use crate::runtime::context::MISSING_RUNTIME_ERROR;
use crate::{DEFAULT_CALL_FN_GAS, ERR_INVALID_VARIANT};

fn rt_current_addr() -> String {
    crate::runtime::context::current()
        .expect(MISSING_RUNTIME_ERROR)
        .rpc_addr()
}

fn json_client(addr: &str) -> JsonRpcClient {
    JsonRpcClient::connect(addr)
}

pub(crate) fn new() -> Client {
    Client::new(rt_current_addr())
}

/// A client that wraps around JsonRpcClient, and provides more capabilities such
/// as retry w/ exponential backoff and utility functions for sending transactions.
pub struct Client {
    rpc_addr: String,
}

impl Client {
    pub(crate) fn new(rpc_addr: String) -> Self {
        Self { rpc_addr }
    }

    // TODO: rename to call_and_retry
    pub(crate) async fn query<M: methods::RpcMethod>(
        &self,
        method: &M,
    ) -> JsonRpcMethodCallResult<M::Result, M::Error> {
        retry(|| async { json_client(&self.rpc_addr).call(method).await }).await
    }

    async fn send_tx_and_retry(
        &self,
        signer: &InMemorySigner,
        receiver_id: AccountId,
        action: Action,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        send_batch_tx_and_retry(self, signer, receiver_id, vec![action]).await
    }

    pub async fn call(
        &self,
        signer: &InMemorySigner,
        contract_id: AccountId,
        method_name: String,
        args: Vec<u8>,
        gas: Option<u64>,
        deposit: Option<Balance>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        self.send_tx_and_retry(
            signer,
            contract_id,
            FunctionCallAction {
                args,
                method_name,
                gas: gas.unwrap_or(DEFAULT_CALL_FN_GAS),
                deposit: deposit.unwrap_or(0),
            }
            .into(),
        )
        .await
    }

    // TODO: return a type T instead of Value
    pub async fn view(
        &self,
        contract_id: AccountId,
        method_name: String,
        args: FunctionArgs,
    ) -> anyhow::Result<serde_json::Value> {
        let query_resp = self
            .query(&RpcQueryRequest {
                block_reference: Finality::None.into(), // Optimisitic query
                request: QueryRequest::CallFunction {
                    account_id: contract_id,
                    method_name,
                    args,
                },
            })
            .await?;

        let call_result = match query_resp.kind {
            QueryResponseKind::CallResult(result) => result.result,
            _ => return Err(anyhow::anyhow!("Error call result")),
        };

        let result = std::str::from_utf8(&call_result)?;
        Ok(serde_json::from_str(result)?)
    }

    pub async fn view_state(
        &self,
        contract_id: AccountId,
        prefix: Option<StoreKey>,
    ) -> anyhow::Result<HashMap<String, Vec<u8>>> {
        let query_resp = self
            .query(&methods::query::RpcQueryRequest {
                block_reference: Finality::None.into(), // Optimisitic query
                request: QueryRequest::ViewState {
                    account_id: contract_id.clone(),
                    prefix: prefix.clone().unwrap_or_else(|| vec![].into()),
                },
            })
            .await?;

        match query_resp.kind {
            QueryResponseKind::ViewState(state) => tool::into_state_map(&state.values),
            _ => Err(anyhow::anyhow!(ERR_INVALID_VARIANT)),
        }
    }

    pub async fn deploy(
        &self,
        signer: &InMemorySigner,
        contract_id: AccountId,
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
    pub async fn transfer_near(
        &self,
        signer: &InMemorySigner,
        receiver_id: AccountId,
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

    pub async fn create_account(
        &self,
        signer: &InMemorySigner,
        new_account_id: AccountId,
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
                    public_key: new_account_pk,
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

    pub async fn create_account_and_deploy(
        &self,
        signer: &InMemorySigner,
        new_account_id: AccountId,
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
                    public_key: new_account_pk,
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
    pub async fn delete_account(
        &self,
        signer: &InMemorySigner,
        account_id: AccountId,
        beneficiary_id: AccountId,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
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
    account_id: AccountId,
    pk: PublicKey,
) -> anyhow::Result<(AccessKeyView, CryptoHash)> {
    let query_resp = client
        .query(&methods::query::RpcQueryRequest {
            block_reference: Finality::Final.into(),
            request: QueryRequest::ViewAccessKey {
                account_id,
                public_key: pk,
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
        .query(&methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: tx.clone(),
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
    receiver_id: AccountId,
    actions: Vec<Action>,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    send_tx_and_retry(client, || async {
        let (AccessKeyView { nonce, .. }, block_hash) =
            access_key(client, signer.account_id.clone(), signer.public_key()).await?;

        Ok(SignedTransaction::from_actions(
            nonce + 1,
            signer.account_id.clone(),
            receiver_id.clone(),
            signer as &dyn Signer,
            actions.clone(),
            block_hash,
        ))
    })
    .await
}
