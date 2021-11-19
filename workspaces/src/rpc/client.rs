// TODO: Remove this when near-jsonrpc-client crate no longer defaults to deprecation for
//       warnings about unstable API.
#![allow(deprecated)]

use near_jsonrpc_client::methods::query::RpcQueryRequest;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::hash::CryptoHash;
use near_primitives::types::{AccountId, Finality, FunctionArgs};
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

use near_crypto::{InMemorySigner, PublicKey, Signer};
use near_jsonrpc_client::{methods, JsonRpcClient, JsonRpcMethodCallResult};
use near_primitives::transaction::{Action, FunctionCallAction, SignedTransaction, TransferAction};
use near_primitives::types::Balance;
use near_primitives::views::{AccessKeyView, FinalExecutionOutcomeView, QueryRequest};

use crate::runtime::context::MISSING_RUNTIME_ERROR;
use crate::DEFAULT_CALL_FN_GAS;

fn rt_current_addr() -> String {
    crate::runtime::context::current()
        .expect(MISSING_RUNTIME_ERROR)
        .rpc_addr()
}

fn json_client() -> JsonRpcClient {
    JsonRpcClient::connect(&rt_current_addr())
}

pub(crate) fn new() -> Client {
    Client::new()
}

/// A client that wraps around JsonRpcClient, and provides more capabilities such
/// as retry w/ exponential backoff and utility functions for sending transactions.
pub struct Client;

impl Client {
    fn new() -> Self {
        Self {}
    }

    pub(crate) async fn call<M: methods::RpcMethod>(
        &self,
        method: &M,
    ) -> JsonRpcMethodCallResult<M::Result, M::Error> {
        retry(|| async { json_client().call(method).await }).await
    }

    async fn send_tx_and_retry(
        &self,
        signer: &InMemorySigner,
        receiver_id: AccountId,
        actions: Vec<Action>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        send_tx_and_retry(|| async {
            let (AccessKeyView { nonce, .. }, block_hash) =
                access_key(signer.account_id.clone(), signer.public_key()).await?;

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

    pub async fn _call(
        &self,
        signer: &InMemorySigner,
        contract_id: AccountId,
        method_name: String,
        args: Vec<u8>,
        gas: Option<u64>,
        deposit: Option<Balance>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        let actions = vec![Action::FunctionCall(FunctionCallAction {
            args,
            method_name,
            gas: gas.unwrap_or(DEFAULT_CALL_FN_GAS),
            deposit: deposit.unwrap_or(0),
        })];

        self.send_tx_and_retry(signer, contract_id, actions).await
    }

    // TODO: return a type T instead of Value
    pub async fn view(
        &self,
        contract_id: AccountId,
        method_name: String,
        args: FunctionArgs,
    ) -> anyhow::Result<serde_json::Value> {
        let query_resp = self
            .call(&RpcQueryRequest {
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

    pub async fn transfer_near(
        &self,
        signer: &InMemorySigner,
        receiver_id: AccountId,
        amount_yocto: Balance,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        self.send_tx_and_retry(
            signer,
            receiver_id,
            vec![Action::Transfer(TransferAction {
                deposit: amount_yocto,
            })],
        )
        .await
    }
}

pub(crate) async fn access_key(
    account_id: AccountId,
    pk: PublicKey,
) -> anyhow::Result<(AccessKeyView, CryptoHash)> {
    let query_resp = new()
        .call(&methods::query::RpcQueryRequest {
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

pub(crate) async fn send_tx(tx: SignedTransaction) -> anyhow::Result<FinalExecutionOutcomeView> {
    self::new()
        .call(&methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: tx.clone(),
        })
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub(crate) async fn send_tx_and_retry<T, F>(task: F) -> anyhow::Result<FinalExecutionOutcomeView>
where
    F: Fn() -> T,
    T: core::future::Future<Output = anyhow::Result<SignedTransaction>>,
{
    retry(|| async { send_tx(task().await?).await }).await
}
