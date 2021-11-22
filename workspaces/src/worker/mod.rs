use near_crypto::PublicKey;
use near_primitives::{account::Account, types::AccountId};

use crate::network::{Contract, Network, NetworkActions};

// TODO: create contract

struct Workspace<T> {
    // network: Box<dyn Network>,
    network: T,
}

// TODO: implement Rc<Workspace> so we can do clone() to copy context
pub struct Worker<T> {
    ctx: Workspace<T>,
}

impl<T> Worker<T> where T: Network {
}
