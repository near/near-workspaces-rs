mod impls;

use std::sync::Arc;

use crate::network::{Mainnet, Network, Sandbox, Testnet};

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
pub async fn sandbox() -> anyhow::Result<Worker<Sandbox>> {
    Ok(Worker::new(Sandbox::new().await?))
}

/// Connect to the [testnet](https://explorer.testnet.near.org/) network, and grab
/// a [`Worker`] that can interact with it.
pub fn testnet() -> Worker<Testnet> {
    Worker::new(Testnet::new())
}

/// Connect to the [testnet archival](https://near-nodes.io/intro/node-types#archival-node)
/// network, and grab a [`Worker`] that can interact with it.
pub fn testnet_archival() -> Worker<Testnet> {
    Worker::new(Testnet::archival())
}

/// Connect to the [mainnet](https://explorer.near.org/) network, and grab
/// a [`Worker`] that can interact with it.
pub fn mainnet() -> Worker<Mainnet> {
    Worker::new(Mainnet::new())
}

/// Connect to the [mainnet archival](https://near-nodes.io/intro/node-types#archival-node)
/// network, and grab a [`Worker`] that can interact with it.
pub fn mainnet_archival() -> Worker<Mainnet> {
    Worker::new(Mainnet::archival())
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
pub async fn with_testnet<F, T>(task: F) -> T::Output
where
    F: Fn(Worker<Testnet>) -> T,
    T: core::future::Future,
{
    task(testnet()).await
}

/// Run a locally scoped task with a [`testnet_archival`] instanced [`Worker`] is supplied.
pub async fn with_testnet_archival<F, T>(task: F) -> T::Output
where
    F: Fn(Worker<Testnet>) -> T,
    T: core::future::Future,
{
    task(testnet_archival()).await
}

/// Run a locally scoped task with a [`mainnet`] instanced [`Worker`] is supplied.
pub async fn with_mainnet<F, T>(task: F) -> T::Output
where
    F: Fn(Worker<Mainnet>) -> T,
    T: core::future::Future,
{
    task(mainnet()).await
}

/// Run a locally scoped task with a [`mainnet_archival`] instanced [`Worker`] is supplied.
pub async fn with_mainnet_archival<F, T>(task: F) -> T::Output
where
    F: Fn(Worker<Mainnet>) -> T,
    T: core::future::Future,
{
    task(mainnet_archival()).await
}
