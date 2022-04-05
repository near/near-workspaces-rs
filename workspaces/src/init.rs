use crate::network::{Mainnet, Sandbox, Testnet};
use std::sync::Arc;

/// Spin up a new sandbox instance, and grab a [`Worker`] that interacts with it.
pub async fn sandbox() -> anyhow::Result<Arc<Sandbox>> {
    Ok(Arc::new(Sandbox::new().await?))
}

/// Connect to the [testnet](https://explorer.testnet.near.org/) network, and grab
/// a [`Worker`] that can interact with it.
pub async fn testnet() -> anyhow::Result<Arc<Testnet>> {
    Ok(Arc::new(Testnet::new().await?))
}

/// Connect to the [testnet archival](https://near-nodes.io/intro/node-types#archival-node)
/// network, and grab a [`Worker`] that can interact with it.
pub async fn testnet_archival() -> anyhow::Result<Arc<Testnet>> {
    Ok(Arc::new(Testnet::archival().await?))
}

/// Connect to the [mainnet](https://explorer.near.org/) network, and grab
/// a [`Worker`] that can interact with it.
pub async fn mainnet() -> anyhow::Result<Arc<Mainnet>> {
    Ok(Arc::new(Mainnet::new().await?))
}

/// Connect to the [mainnet archival](https://near-nodes.io/intro/node-types#archival-node)
/// network, and grab a [`Worker`] that can interact with it.
pub async fn mainnet_archival() -> anyhow::Result<Arc<Mainnet>> {
    Ok(Arc::new(Mainnet::archival().await?))
}

/// Run a locally scoped task with a [`sandbox`] instanced [`Worker`] is supplied.
pub async fn with_sandbox<F, T>(task: F) -> anyhow::Result<T::Output>
where
    F: Fn(Arc<Sandbox>) -> T,
    T: core::future::Future,
{
    Ok(task(sandbox().await?).await)
}

/// Run a locally scoped task with a [`testnet`] instanced [`Worker`] is supplied.
pub async fn with_testnet<F, T>(task: F) -> anyhow::Result<T::Output>
where
    F: Fn(Arc<Testnet>) -> T,
    T: core::future::Future,
{
    Ok(task(testnet().await?).await)
}

/// Run a locally scoped task with a [`testnet_archival`] instanced [`Worker`] is supplied.
pub async fn with_testnet_archival<F, T>(task: F) -> anyhow::Result<T::Output>
where
    F: Fn(Arc<Testnet>) -> T,
    T: core::future::Future,
{
    Ok(task(testnet_archival().await?).await)
}

/// Run a locally scoped task with a [`mainnet`] instanced [`Worker`] is supplied.
pub async fn with_mainnet<F, T>(task: F) -> anyhow::Result<T::Output>
where
    F: Fn(Arc<Mainnet>) -> T,
    T: core::future::Future,
{
    Ok(task(mainnet().await?).await)
}

/// Run a locally scoped task with a [`mainnet_archival`] instanced [`Worker`] is supplied.
pub async fn with_mainnet_archival<F, T>(task: F) -> anyhow::Result<T::Output>
where
    F: Fn(Arc<Mainnet>) -> T,
    T: core::future::Future,
{
    Ok(task(mainnet_archival().await?).await)
}
