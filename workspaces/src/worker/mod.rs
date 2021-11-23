mod impls;


use crate::network::{Network, Sandbox};

// TODO: create contract

struct Workspace<T> {
    // network: Box<dyn Network>,
    network: T,
}

// TODO: implement Rc<Workspace> so we can do clone() to copy context
pub struct Worker<T> {
    workspace: T,
}

impl<T> Worker<T> where T: Network {
    pub(crate) fn new(network: T) -> Self {
        Self {
            workspace: network,
        }
    }
}

pub async fn sandbox<F, T>(task: F) -> <T as core::future::Future>::Output
where
    F: Fn(Worker<Sandbox>) -> T,
    T: core::future::Future,
{
    let worker = Worker::new(Sandbox::default());

    task(worker).await
}
