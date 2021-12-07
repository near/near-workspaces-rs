use std::hash::{Hash, Hasher};

use crate::{types::{AccountId, InMemorySigner}, Worker, Network};

use super::CallExecutionDetails;

pub struct Account {
    pub(crate) id: AccountId,
    pub(crate) signer: InMemorySigner,
}

impl Account {
    pub(crate) fn new(id: AccountId, signer: InMemorySigner) -> Self {
        Self { id, signer }
    }

    pub fn id(&self) -> &AccountId {
        &self.id
    }

    pub(crate) fn signer(&self) -> &InMemorySigner {
        &self.signer
    }

    pub fn call_other<'a, T: Network>(&self, worker: &'a Worker<T>, other: AccountId, function: String) -> CallBuilder<'a, T> {
        CallBuilder::new(worker, other, self.signer.clone(), function)
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Account {}

impl Hash for Account {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// TODO: allow users to create Contracts so that they can call into
// them without deploying the contract themselves.
#[derive(PartialEq, Eq, Hash)]
pub struct Contract {
    pub(crate) account: Account,
}

impl Contract {
    pub(crate) fn new(id: AccountId, signer: InMemorySigner) -> Self {
        Self {
            account: Account::new(id, signer),
        }
    }

    pub(crate) fn account(account: Account) -> Self {
        Self { account }
    }

    pub fn id(&self) -> &AccountId {
        &self.account.id
    }

    pub(crate) fn signer(&self) -> &InMemorySigner {
        self.account.signer()
    }

    pub fn call<'a, T: Network>(&'a self, worker: &'a Worker<T>, function: String) -> CallBuilder<'a, T> {
        CallBuilder::new(worker, self.id().clone(), self.signer().clone(), function)
    }

    pub fn call_other<'a, T: Network>(&self, worker: &'a Worker<T>, other: AccountId, function: String) -> CallBuilder<'a, T> {
        self.account.call_other(worker, other, function)
    }
}

pub struct CallBuilder<'a, T> {
    worker: &'a Worker<T>,
    signer: InMemorySigner,
    contract_id: AccountId,

    function: String,
    args: Option<Vec<u8>>,
    deposit: Option<u128>,
    gas: Option<u64>,
}

impl<'a, T: Network> CallBuilder<'a, T> {
    fn new(worker: &'a Worker<T>, contract_id: AccountId, signer: InMemorySigner, function: String) -> Self {
        Self {
            worker,
            signer,
            contract_id,
            function,
            args: None,
            deposit: None,
            gas: None,
        }
    }

    pub fn with_args(mut self, args: Vec<u8>) -> Self {
        self.args = Some(args);
        self
    }

    pub fn with_deposit(mut self, deposit: u128) -> Self {
        self.deposit = Some(deposit);
        self
    }

    pub fn with_gas(mut self, gas: u64) -> Self {
        self.gas = Some(gas);
        self
    }

    pub async fn transact(self) -> anyhow::Result<CallExecutionDetails> {
        self.worker.client()
            .call(
                &self.signer,
                self.contract_id,
                self.function,
                self.args.expect("required `with_gas` to be specified as apart of Call"),
                self.gas,
                self.deposit,
            )
            .await
            .map(Into::into)
    }
}
