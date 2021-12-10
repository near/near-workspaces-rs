use crate::types::{AccountId, InMemorySigner, Balance};
use crate::{Network, Worker};

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

    /// Call a contract on the network specified within `worker`, and return
    /// a Builder object that we will make use to populate the rest of the
    /// call details.
    pub fn call<'a, T: Network>(
        &self,
        worker: &'a Worker<T>,
        contract_id: AccountId,
        function: String,
    ) -> CallBuilder<'a, T> {
        CallBuilder::new(worker, contract_id, self.signer.clone(), function)
    }

    /// Transfer near to an account specified by `receiver_id` with the amount
    /// specified by `amount_yocto`. Returns the execution details of this
    /// transaction
    pub async fn transfer_near<T: Network>(
        &self,
        worker: &Worker<T>,
        receiver_id: AccountId,
        amount_yocto: Balance,
    ) -> anyhow::Result<CallExecutionDetails> {
        worker
            .transfer_near(self.signer(), receiver_id, amount_yocto)
            .await
    }

    /// Deletes the current account, and returns the execution details of this
    /// transaction. The beneciary will receive the funds of the account deleted
    pub async fn delete_account<T: Network>(
        self,
        worker: &Worker<T>,
        beneficiary_id: AccountId,
    ) -> anyhow::Result<CallExecutionDetails> {
        worker
            .delete_account(self.id, &self.signer, beneficiary_id)
            .await
    }
}

// TODO: allow users to create Contracts so that they can call into
// them without deploying the contract themselves.
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

    pub fn as_account(&self) -> &Account {
        &self.account
    }

    pub(crate) fn signer(&self) -> &InMemorySigner {
        self.account.signer()
    }

    /// Call the current contract's function using the contract's own account
    /// details to do the signing. Returns a Builder object that we will make
    /// use to populate the rest of the call details.
    ///
    /// If we want to make use of the contract's account to call into a
    /// different contract besides the current one, use
    /// `contract.as_account().call` instead.
    pub fn call<'a, T: Network>(
        &self,
        worker: &'a Worker<T>,
        function: String,
    ) -> CallBuilder<'a, T> {
        self.account.call(worker, self.id().clone(), function)
    }

    /// Call a view function into the current contract. Returns a result that
    /// yields a JSON string object.
    pub async fn view<T: Network>(
        &self,
        worker: &Worker<T>,
        function: String,
        args: Vec<u8>,
    ) -> anyhow::Result<String> {
        worker.view(self.id().clone(), function, args).await
    }

    /// Deletes the current contract, and returns the execution details of this
    /// transaction. The beneciary will receive the funds of the account deleted
    pub async fn delete_contract<T: Network>(
        self,
        worker: &Worker<T>,
        beneficiary_id: AccountId,
    ) -> anyhow::Result<CallExecutionDetails> {
        self.account.delete_account(worker, beneficiary_id).await
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
    fn new(
        worker: &'a Worker<T>,
        contract_id: AccountId,
        signer: InMemorySigner,
        function: String,
    ) -> Self {
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
        self.worker
            .client()
            .call(
                &self.signer,
                self.contract_id,
                self.function,
                self.args
                    .expect("required `with_args` to be specified as apart of Call"),
                self.gas,
                self.deposit,
            )
            .await
            .map(Into::into)
    }

    pub async fn view(self) -> anyhow::Result<Vec<u8>> {
        self.worker
            .client()
            .view(
                self.contract_id,
                self.function,
                self.args
                    .expect("required `with_args` to be specified as apart of View"),
            )
            .await
            .map(Into::into)
    }
}
