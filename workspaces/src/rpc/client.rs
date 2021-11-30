// TODO: Remove this when near-jsonrpc-client crate no longer defaults to deprecation for
//       warnings about unstable API.
#![allow(deprecated)]

use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

use near_jsonrpc_client::{methods, JsonRpcClient, JsonRpcMethodCallResult};
use near_primitives::transaction::SignedTransaction;
use near_primitives::views::FinalExecutionOutcomeView;

use crate::runtime::context::MISSING_RUNTIME_ERROR;

fn rt_current_addr() -> String {
    crate::runtime::context::current()
        .expect(MISSING_RUNTIME_ERROR)
        .rpc_addr()
}

fn json_client() -> JsonRpcClient {
    JsonRpcClient::connect(&rt_current_addr())
}

pub(crate) fn new() -> RetryClient {
    RetryClient::new()
}

pub(crate) struct RetryClient;

impl RetryClient {
    fn new() -> Self {
        Self {}
    }

    pub(crate) async fn call<M: methods::RpcMethod>(
        &self,
        method: &M,
    ) -> JsonRpcMethodCallResult<M::Result, M::Error> {
        retry(|| async { json_client().call(method).await }).await
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
