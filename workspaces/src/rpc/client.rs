use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

use near_jsonrpc_client::{methods::RpcMethod, JsonRpcClient, JsonRpcMethodCallResult};

use crate::runtime::context::MISSING_RUNTIME_ERROR;

// Default number of retries with different nonce before giving up on a transaction.
const TX_NONCE_RETRY_NUMBER: usize = 12;
// Default wait until next retry in millis.
const TX_NONCE_RETRY_WAIT: u64 = 100;


fn rt_current_addr() -> String {
    crate::runtime::context::current()
        .expect(MISSING_RUNTIME_ERROR)
        .rpc_addr()
}

pub(crate) fn json_client() -> JsonRpcClient {
    JsonRpcClient::connect(&rt_current_addr())
}

pub(crate) fn client() -> RetryClient {
    RetryClient::new()
}

pub struct RetryClient {
    json_client: JsonRpcClient,
}

impl RetryClient {
    fn new() -> Self {
        Self {
            json_client: json_client(),
        }
    }

    pub(crate) async fn call<M: RpcMethod>(
        &self,
        method: &M,
    ) -> JsonRpcMethodCallResult<M::Result, M::Error> {
        let mut c = 0;
        // Need this because of the FnMut trait requirement:
        let task = || {
            c += 1;
            println!("trying {}", c);
            self.json_client.clone().call(method)
        };

        retry(task).await
    }
}

pub(crate) async fn retry<R, E, T, F>(task: F) -> T::Output
// anyhow::Result<T::Output>
where
    F: FnMut() -> T,
    T: core::future::Future<Output = Result<R, E>>,
{
    let retry_strategy = ExponentialBackoff::from_millis(TX_NONCE_RETRY_WAIT)
        .map(jitter) // add jitter to delays
        .take(TX_NONCE_RETRY_NUMBER);

    Retry::spawn(retry_strategy, task).await
}
