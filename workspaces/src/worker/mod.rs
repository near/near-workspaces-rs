mod impls;

use std::sync::Arc;

use crate::network::{Mainnet, Sandbox, Testnet};
use crate::Network;

pub struct Worker<T> {
    pub(crate) workspace: Arc<T>,
}

impl<T> Clone for Worker<T> {
    fn clone(&self) -> Self {
        Self {
            workspace: Arc::clone(&self.workspace),
        }
    }
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
pub async fn sandbox() -> anyhow::Result<Worker<Sandbox>> {
    Ok(Worker::new(Sandbox::new().await?))
}

/// Connect to the [testnet](https://explorer.testnet.near.org/) network, and grab
/// a [`Worker`] that can interact with it.
pub async fn testnet() -> anyhow::Result<Worker<Testnet>> {
    Ok(Worker::new(Testnet::new().await?))
}

/// Connect to the [testnet archival](https://near-nodes.io/intro/node-types#archival-node)
/// network, and grab a [`Worker`] that can interact with it.
pub async fn testnet_archival() -> anyhow::Result<Worker<Testnet>> {
    Ok(Worker::new(Testnet::archival().await?))
}

/// Connect to the [mainnet](https://explorer.near.org/) network, and grab
/// a [`Worker`] that can interact with it.
pub async fn mainnet() -> anyhow::Result<Worker<Mainnet>> {
    Ok(Worker::new(Mainnet::new().await?))
}

/// Connect to the [mainnet archival](https://near-nodes.io/intro/node-types#archival-node)
/// network, and grab a [`Worker`] that can interact with it.
pub async fn mainnet_archival() -> anyhow::Result<Worker<Mainnet>> {
    Ok(Worker::new(Mainnet::archival().await?))
}

/// Run a locally scoped task with a [`sandbox`] instanced [`Worker`] is supplied.
pub async fn with_sandbox<F, T>(task: F) -> anyhow::Result<T::Output>
where
    F: Fn(Worker<Sandbox>) -> T,
    T: core::future::Future,
{
    Ok(task(sandbox().await?).await)
}

/// Run a locally scoped task with a [`testnet`] instanced [`Worker`] is supplied.
pub async fn with_testnet<F, T>(task: F) -> anyhow::Result<T::Output>
where
    F: Fn(Worker<Testnet>) -> T,
    T: core::future::Future,
{
    Ok(task(testnet().await?).await)
}

/// Run a locally scoped task with a [`testnet_archival`] instanced [`Worker`] is supplied.
pub async fn with_testnet_archival<F, T>(task: F) -> anyhow::Result<T::Output>
where
    F: Fn(Worker<Testnet>) -> T,
    T: core::future::Future,
{
    Ok(task(testnet_archival().await?).await)
}

/// Run a locally scoped task with a [`mainnet`] instanced [`Worker`] is supplied.
pub async fn with_mainnet<F, T>(task: F) -> anyhow::Result<T::Output>
where
    F: Fn(Worker<Mainnet>) -> T,
    T: core::future::Future,
{
    Ok(task(mainnet().await?).await)
}

/// Run a locally scoped task with a [`mainnet_archival`] instanced [`Worker`] is supplied.
pub async fn with_mainnet_archival<F, T>(task: F) -> anyhow::Result<T::Output>
where
    F: Fn(Worker<Mainnet>) -> T,
    T: core::future::Future,
{
    Ok(task(mainnet_archival().await?).await)
}
