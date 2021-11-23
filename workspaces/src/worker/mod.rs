mod impls;

use std::sync::Arc;

use crate::network::{Network, Sandbox};

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

pub fn sandbox() -> Worker<Sandbox> {
    Worker::new(Sandbox::new())
}
