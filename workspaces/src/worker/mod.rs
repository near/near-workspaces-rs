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

pub fn sandbox() -> Worker<Sandbox> {
    Worker::new(Sandbox::new())
}

pub fn testnet() -> Worker<Testnet> {
    Worker::new(Testnet::new())
}

pub fn mainnet() -> Worker<Mainnet> {
    Worker::new(Mainnet::new())
}

pub fn mainnet_archival() -> Worker<Mainnet> {
    Worker::new(Mainnet::archival())
}

pub async fn with_sandbox<F, T>(task: F) -> T::Output
where
    F: Fn(Worker<Sandbox>) -> T,
    T: core::future::Future,
{
    task(sandbox()).await
}

pub async fn with_testnet<F, T>(task: F) -> T::Output
where
    F: Fn(Worker<Testnet>) -> T,
    T: core::future::Future,
{
    task(testnet()).await
}

pub async fn with_mainnet<F, T>(task: F) -> T::Output
where
    F: Fn(Worker<Mainnet>) -> T,
    T: core::future::Future,
{
    task(mainnet()).await
}

pub async fn with_mainnet_archival<F, T>(task: F) -> T::Output
where
    F: Fn(Worker<Mainnet>) -> T,
    T: core::future::Future,
{
    task(mainnet_archival()).await
}
