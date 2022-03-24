use near_primitives::views::AccountView;

use crate::types::{AccountId, Balance, InMemorySigner};
use crate::{CryptoHash, Network, Worker};

use super::transaction::{CallTransaction, CreateAccountTransaction, Transaction};
use super::{CallExecution, CallExecutionDetails, ViewResultDetails};

pub struct Account {
    pub(crate) id: AccountId,
    pub(crate) signer: InMemorySigner,
}

impl Account {
    /// Create a new account with the given path to the credentials JSON file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Self {
        let signer = InMemorySigner::from_file(path.as_ref());
        let id = signer.0.account_id.clone();
        Self::new(id, signer)
    }

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
    /// a [`CallTransaction`] object that we will make use to populate the
    /// rest of the call details.
    pub fn call<'a, 'b, T: Network>(
        &self,
        worker: &'a Worker<T>,
        contract_id: &AccountId,
        function: &'b str,
    ) -> CallTransaction<'a, 'b, T> {
        CallTransaction::new(
            worker,
            contract_id.to_owned(),
            self.signer.clone(),
            function,
        )
    }

    /// Transfer NEAR to an account specified by `receiver_id` with the amount
    /// specified by `amount`. Returns the execution details of this transaction
    pub async fn transfer_near<T: Network>(
        &self,
        worker: &Worker<T>,
        receiver_id: &AccountId,
        amount: Balance,
    ) -> anyhow::Result<CallExecutionDetails> {
        worker
            .transfer_near(self.signer(), receiver_id, amount)
            .await
    }

    /// Deletes the current account, and returns the execution details of this
    /// transaction. The beneficiary will receive the funds of the account deleted
    pub async fn delete_account<T: Network>(
        self,
        worker: &Worker<T>,
        beneficiary_id: &AccountId,
    ) -> anyhow::Result<CallExecutionDetails> {
        worker
            .delete_account(&self.id, &self.signer, beneficiary_id)
            .await
    }

    /// Views the current account's details such as balance and storage usage.
    pub async fn view_account<T: Network>(
        &self,
        worker: &Worker<T>,
    ) -> anyhow::Result<AccountDetails> {
        worker.view_account(&self.id).await
    }

    /// Create a new sub account. Returns a [`CreateAccountTransaction`] object
    /// that we can make use of to fill out the rest of the details. The subaccount
    /// id will be in the form of: "{new_account_id}.{parent_account_id}"
    pub fn create_subaccount<'a, 'b, T: Network>(
        &self,
        worker: &'a Worker<T>,
        new_account_id: &'b str,
    ) -> CreateAccountTransaction<'a, 'b, T> {
        CreateAccountTransaction::new(
            worker,
            self.signer.clone(),
            self.id().clone(),
            new_account_id,
        )
    }

    /// Deploy contract code or WASM bytes to the account, and return us a new
    /// [`Contract`] object that we can use to interact with the contract.
    pub async fn deploy<T: Network>(
        &self,
        worker: &Worker<T>,
        wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract>> {
        let outcome = worker
            .client()
            .deploy(&self.signer, self.id(), wasm.as_ref().into())
            .await?;

        Ok(CallExecution {
            result: Contract::new(self.id().clone(), self.signer().clone()),
            details: outcome.into(),
        })
    }

    /// Start a batch transaction, using the current account as the signer and
    /// making calls into the contract provided by `contract_id`. Returns a
    /// [`Transaction`] object that we can use to add Actions to the batched
    /// transaction. Call `transact` to send the batched transaction to the
    /// network.
    pub fn batch<'a, T: Network>(
        &self,
        worker: &'a Worker<T>,
        contract_id: &AccountId,
    ) -> Transaction<'a> {
        Transaction::new(worker.client(), self.signer().clone(), contract_id.clone())
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
    /// details to do the signing. Returns a [`CallTransaction`] object that
    /// we will make use to populate the rest of the call details.
    ///
    /// If we want to make use of the contract's account to call into a
    /// different contract besides the current one, use
    /// `contract.as_account().call` instead.
    pub fn call<'a, 'b, T: Network>(
        &self,
        worker: &'a Worker<T>,
        function: &'b str,
    ) -> CallTransaction<'a, 'b, T> {
        self.account.call(worker, self.id(), function)
    }

    /// Call a view function into the current contract. Returns a result that
    /// yields a JSON string object.
    pub async fn view<T: Network>(
        &self,
        worker: &Worker<T>,
        function: &str,
        args: Vec<u8>,
    ) -> anyhow::Result<ViewResultDetails> {
        worker.view(self.id(), function, args).await
    }

    /// View the WASM code bytes of this contract.
    pub async fn view_code<T: Network>(&self, worker: &Worker<T>) -> anyhow::Result<Vec<u8>> {
        worker.view_code(self.id()).await
    }

    /// Views the current contract's details such as balance and storage usage.
    pub async fn view_account<T: Network>(
        &self,
        worker: &Worker<T>,
    ) -> anyhow::Result<AccountDetails> {
        worker.view_account(self.id()).await
    }

    /// Deletes the current contract, and returns the execution details of this
    /// transaction. The beneciary will receive the funds of the account deleted
    pub async fn delete_contract<T: Network>(
        self,
        worker: &Worker<T>,
        beneficiary_id: &AccountId,
    ) -> anyhow::Result<CallExecutionDetails> {
        self.account.delete_account(worker, beneficiary_id).await
    }

    /// Start a batch transaction, using the current contract as the signer and
    /// making calls into this contract. Returns a [`Transaction`] object that
    /// we can use to add Actions to the batched transaction. Call `transact`
    /// to send the batched transaction to the network.
    pub fn batch<'a, T: Network>(&self, worker: &'a Worker<T>) -> Transaction<'a> {
        Transaction::new(worker.client(), self.signer().clone(), self.id().clone())
    }
}

/// Details of an Account or Contract. This is an non-exhaustive list of items
/// that the account stores in the blockchain state.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct AccountDetails {
    pub balance: Balance,
    pub locked: Balance,
    pub code_hash: CryptoHash,
    pub storage_usage: u64,
}

impl From<AccountView> for AccountDetails {
    fn from(account: AccountView) -> Self {
        Self {
            balance: account.amount,
            locked: account.locked,
            code_hash: CryptoHash(account.code_hash.0),
            storage_usage: account.storage_usage,
        }
    }
}
