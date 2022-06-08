mod impls;

use std::sync::Arc;

use crate::network::{Betanet, Mainnet, Sandbox, Testnet};
use crate::result::Result;
use crate::Network;

/// The `Worker` type allows us to interact with any NEAR related networks, such
/// as mainnet and testnet. This controls where the environment the worker is
/// running on top of is. Refer to this for all network related actions such as
/// deploying a contract, or interacting with transactions.
pub struct Worker<T> {
    workspace: Arc<T>,
}

impl<T> Worker<T>
where
    T: Network,
{
    pub(crate) fn new(network: T) -> Self {
        Self {
            workspace: Arc::new(network),
        }
    }
}

/// Spin up a new sandbox instance, and grab a [`Worker`] that interacts with it.
pub async fn sandbox() -> Result<Worker<Sandbox>> {
    Ok(Worker::new(Sandbox::new().await?))
}

/// Connect to the [testnet](https://explorer.testnet.near.org/) network, and grab
/// a [`Worker`] that can interact with it.
pub async fn testnet() -> Result<Worker<Testnet>> {
    Ok(Worker::new(Testnet::new().await?))
}

/// Connect to the [testnet archival](https://near-nodes.io/intro/node-types#archival-node)
/// network, and grab a [`Worker`] that can interact with it.
pub async fn testnet_archival() -> Result<Worker<Testnet>> {
    Ok(Worker::new(Testnet::archival().await?))
}

/// Connect to the [mainnet](https://explorer.near.org/) network, and grab
/// a [`Worker`] that can interact with it.
pub async fn mainnet() -> Result<Worker<Mainnet>> {
    Ok(Worker::new(Mainnet::new().await?))
}

/// Connect to the [mainnet archival](https://near-nodes.io/intro/node-types#archival-node)
/// network, and grab a [`Worker`] that can interact with it.
pub async fn mainnet_archival() -> Result<Worker<Mainnet>> {
    Ok(Worker::new(Mainnet::archival().await?))
}

/// Connect to the betanet network, and grab a [`Worker`] that can interact with it.
pub async fn betanet() -> Result<Worker<Betanet>> {
    Ok(Worker::new(Betanet::new().await?))
}

/// Run a locally scoped task where a [`sandbox`] instanced [`Worker`] is supplied.
pub async fn with_sandbox<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Sandbox>) -> T,
    T: core::future::Future,
{
    Ok(task(sandbox().await?).await)
}

/// Run a locally scoped task where a [`testnet`] instanced [`Worker`] is supplied.
pub async fn with_testnet<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Testnet>) -> T,
    T: core::future::Future,
{
    Ok(task(testnet().await?).await)
}

/// Run a locally scoped task where a [`testnet_archival`] instanced [`Worker`] is supplied.
pub async fn with_testnet_archival<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Testnet>) -> T,
    T: core::future::Future,
{
    Ok(task(testnet_archival().await?).await)
}

/// Run a locally scoped task where a [`mainnet`] instanced [`Worker`] is supplied.
pub async fn with_mainnet<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Mainnet>) -> T,
    T: core::future::Future,
{
    Ok(task(mainnet().await?).await)
}

/// Run a locally scoped task where a [`mainnet_archival`] instanced [`Worker`] is supplied.
pub async fn with_mainnet_archival<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Mainnet>) -> T,
    T: core::future::Future,
{
    Ok(task(mainnet_archival().await?).await)
}

/// Run a locally scoped task where a [`betanet`] instanced [`Worker`] is supplied.
pub async fn with_betanet<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Betanet>) -> T,
    T: core::future::Future,
{
    Ok(task(betanet().await?).await)
}
