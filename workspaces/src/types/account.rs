use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use near_primitives::views::AccountView;

use crate::error::ErrorKind;
use crate::types::{AccountId, Balance, InMemorySigner, SecretKey};
use crate::{CryptoHash, Network, Worker};

use crate::operations::{CallTransaction, CreateAccountTransaction, Transaction};
use crate::result::{Execution, ExecutionFinalResult, Result, ViewResultDetails};

/// `Account` is directly associated to an account in the network provided by the
/// [`Worker`] that creates it. This type offers methods to interact with any
/// network, such as creating transactions and calling into contract functions.
#[derive(Clone)]
pub struct Account {
    pub(crate) id: AccountId,
    pub(crate) signer: InMemorySigner,
    worker: Worker<dyn Network>,
}

impl fmt::Debug for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Account").field("id", &self.id).finish()
    }
}

impl Account {
    /// Create a new account with the given path to the credentials JSON file
    pub fn from_file(
        path: impl AsRef<std::path::Path>,
        worker: &Worker<impl Network + 'static>,
    ) -> Result<Self> {
        let signer = InMemorySigner::from_file(path.as_ref())?;
        let id = signer.account_id.clone();
        Ok(Self::new(id, signer, worker.clone().coerce()))
    }

    /// Create an [`Account`] object from an [`AccountId`] and [`SecretKey`].
    pub fn from_secret_key(
        id: AccountId,
        sk: SecretKey,
        worker: &Worker<impl Network + 'static>,
    ) -> Self {
        Self {
            id: id.clone(),
            signer: InMemorySigner::from_secret_key(id, sk),
            worker: worker.clone().coerce(),
        }
    }

    pub(crate) fn new(id: AccountId, signer: InMemorySigner, worker: Worker<dyn Network>) -> Self {
        Self { id, signer, worker }
    }

    /// Grab the current account identifier
    pub fn id(&self) -> &AccountId {
        &self.id
    }

    pub(crate) fn signer(&self) -> &InMemorySigner {
        &self.signer
    }

    /// Call a contract on the network specified within `worker`, and return
    /// a [`CallTransaction`] object that we will make use to populate the
    /// rest of the call details. Note that the current [`Account`]'s secret
    /// key is used as the signer of the transaction.
    pub fn call<'a, 'b>(
        &'a self,
        contract_id: &AccountId,
        function: &'b str,
    ) -> CallTransaction<'a, 'b> {
        CallTransaction::new(
            &self.worker,
            contract_id.to_owned(),
            self.signer.clone(),
            function,
        )
    }

    /// View call to a specified contract function. Returns a result which can
    /// be deserialized into borsh or JSON.
    pub async fn view(
        &self,
        contract_id: &AccountId,
        function: &str,
        args: Vec<u8>,
    ) -> Result<ViewResultDetails> {
        self.worker.view(contract_id, function, args).await
    }

    /// Transfer NEAR to an account specified by `receiver_id` with the amount
    /// specified by `amount`. Returns the execution details of this transaction
    pub async fn transfer_near(
        &self,
        receiver_id: &AccountId,
        amount: Balance,
    ) -> Result<ExecutionFinalResult> {
        self.worker
            .transfer_near(self.signer(), receiver_id, amount)
            .await
    }

    /// Deletes the current account, and returns the execution details of this
    /// transaction. The beneficiary will receive the funds of the account deleted
    pub async fn delete_account(self, beneficiary_id: &AccountId) -> Result<ExecutionFinalResult> {
        self.worker
            .delete_account(&self.id, &self.signer, beneficiary_id)
            .await
    }

    /// Views the current account's details such as balance and storage usage.
    pub async fn view_account(&self) -> Result<AccountDetails> {
        self.worker.view_account(&self.id).await
    }

    /// Create a new sub account. Returns a [`CreateAccountTransaction`] object
    /// that we can make use of to fill out the rest of the details. The subaccount
    /// id will be in the form of: "{new_account_id}.{parent_account_id}"
    pub fn create_subaccount<'a, 'b>(
        &'a self,
        new_account_id: &'b str,
    ) -> CreateAccountTransaction<'a, 'b> {
        CreateAccountTransaction::new(
            &self.worker,
            self.signer.clone(),
            self.id().clone(),
            new_account_id,
        )
    }

    /// Deploy contract code or WASM bytes to the account, and return us a new
    /// [`Contract`] object that we can use to interact with the contract.
    pub async fn deploy(&self, wasm: &[u8]) -> Result<Execution<Contract>> {
        let outcome = self
            .worker
            .client()
            .deploy(&self.signer, self.id(), wasm.as_ref().into())
            .await?;

        Ok(Execution {
            result: Contract::new(
                self.id().clone(),
                self.signer().clone(),
                self.worker.clone(),
            ),
            details: ExecutionFinalResult::from_view(outcome),
        })
    }

    /// Start a batch transaction, using the current account as the signer and
    /// making calls into the contract provided by `contract_id`. Returns a
    /// [`Transaction`] object that we can use to add Actions to the batched
    /// transaction. Call `transact` to send the batched transaction to the
    /// network.
    pub fn batch(&self, contract_id: &AccountId) -> Transaction {
        Transaction::new(
            self.worker.client(),
            self.signer().clone(),
            contract_id.clone(),
        )
    }

    /// Store the credentials of this account locally in the directory provided.
    pub async fn store_credentials(&self, save_dir: impl AsRef<Path>) -> Result<()> {
        let savepath = save_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(save_dir).map_err(|e| ErrorKind::Io.custom(e))?;

        let mut savepath = savepath.join(self.id.to_string());
        savepath.set_extension("json");

        crate::rpc::tool::write_cred_to_file(&savepath, &self.id, &self.secret_key().0);

        Ok(())
    }

    /// Get the keys of this account. The public key can be retrieved from the secret key.
    pub fn secret_key(&self) -> &SecretKey {
        &self.signer.secret_key
    }

    /// Sets the [`SecretKey`] of this account. Future transactions will be signed
    /// using this newly provided key.
    pub fn set_secret_key(&mut self, sk: SecretKey) {
        self.signer.secret_key = sk;
    }
}

/// `Contract` is directly associated to a contract in the network provided by the
/// [`Worker`] that creates it. This type offers methods to interact with any
/// network, such as creating transactions and calling into contract functions.
#[derive(Clone)]
pub struct Contract {
    pub(crate) account: Account,
}

impl fmt::Debug for Contract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Contract")
            .field("id", &self.account.id)
            .finish()
    }
}

impl Contract {
    /// Create a [`Contract`] object from an [`AccountId`] and [`SecretKey`].
    pub fn from_secret_key(
        id: AccountId,
        sk: SecretKey,
        worker: &Worker<impl Network + 'static>,
    ) -> Self {
        Self::account(Account::from_secret_key(id, sk, worker))
    }

    pub(crate) fn new(id: AccountId, signer: InMemorySigner, worker: Worker<dyn Network>) -> Self {
        Self {
            account: Account::new(id, signer, worker),
        }
    }

    pub(crate) fn account(account: Account) -> Self {
        Self { account }
    }

    /// Grab the current contract's account identifier
    pub fn id(&self) -> &AccountId {
        &self.account.id
    }

    /// Treat this [`Contract`] object as an [`Account`] type. This does nothing
    /// on chain/network, and is merely allowing `Account::*` functions to be
    /// used from this `Contract`.
    pub fn as_account(&self) -> &Account {
        &self.account
    }

    /// Treat this [`Contract`] object as an [`Account`] type. This does nothing
    /// on chain/network, and is merely allowing `Account::*` functions to be
    /// used from this `Contract`.
    pub fn as_account_mut(&mut self) -> &mut Account {
        &mut self.account
    }

    pub(crate) fn signer(&self) -> &InMemorySigner {
        self.account.signer()
    }

    /// Call the current contract's function using the contract's own account
    /// secret key to do the signing. Returns a [`CallTransaction`] object that
    /// we will make use to populate the rest of the call details.
    ///
    /// If we want to make use of the contract's secret key as a signer to call
    /// into another contract, use `contract.as_account().call` instead.
    pub fn call<'a>(&self, function: &'a str) -> CallTransaction<'_, 'a> {
        self.account.call(self.id(), function)
    }

    /// Call a view function into the current contract. Returns a result which can
    /// be deserialized into borsh or JSON.
    pub async fn view(&self, function: &str, args: Vec<u8>) -> Result<ViewResultDetails> {
        self.account.worker.view(self.id(), function, args).await
    }

    /// View the WASM code bytes of this contract.
    pub async fn view_code(&self) -> Result<Vec<u8>> {
        self.account.worker.view_code(self.id()).await
    }

    /// View a contract's state map of key value pairs.
    pub async fn view_state(&self, prefix: Option<&[u8]>) -> Result<HashMap<Vec<u8>, Vec<u8>>> {
        self.account.worker.view_state(self.id(), prefix).await
    }

    /// Views the current contract's details such as balance and storage usage.
    pub async fn view_account(&self) -> Result<AccountDetails> {
        self.account.worker.view_account(self.id()).await
    }

    /// Deletes the current contract, and returns the execution details of this
    /// transaction. The beneciary will receive the funds of the account deleted
    pub async fn delete_contract(self, beneficiary_id: &AccountId) -> Result<ExecutionFinalResult> {
        self.account.delete_account(beneficiary_id).await
    }

    /// Start a batch transaction, using the current contract's secret key as the
    /// signer, making calls into itself. Returns a [`Transaction`] object that
    /// we can use to add Actions to the batched transaction. Call `transact`
    /// to send the batched transaction to the network.
    pub fn batch(&self) -> Transaction {
        self.account.batch(self.id())
    }
}

/// Details of an Account or Contract. This is an non-exhaustive list of items
/// that the account stores in the blockchain state.
#[derive(Debug, Eq, PartialEq)]
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
