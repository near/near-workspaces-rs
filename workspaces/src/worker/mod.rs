mod impls;

use std::sync::Arc;

use crate::network::{Network, Sandbox};

#[derive(Clone)]
pub struct Worker<T> {
    workspace: Arc<T>,
}

impl<T> Worker<T> where T: Network {
    pub(crate) fn new(network: T) -> Self {
        Self {
            workspace: Arc::new(network),
        }
    }
}

pub async fn sandbox<F, T>(task: F) -> <T as core::future::Future>::Output
where
    F: Fn(Worker<Sandbox>) -> T,
    T: core::future::Future,
{
    let worker = Worker::new(Sandbox::new());

    task(worker).await
}
