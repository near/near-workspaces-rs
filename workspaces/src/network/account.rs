use near_crypto::{InMemorySigner};
use near_primitives::types::AccountId;

pub struct Account {
    pub(crate) id: AccountId,
}

impl Account {
    pub fn id(&self) -> AccountId {
        self.id.clone()
    }
}

unsafe impl Sync for Account {}
unsafe impl Send for Account {}

pub struct Contract {
    pub(crate) account: Account,
    pub(crate) signer: InMemorySigner,
}

impl Contract {
    pub fn id(&self) -> AccountId {
        self.account.id.clone()
    }
}

unsafe impl Sync for Contract {}
unsafe impl Send for Contract {}
