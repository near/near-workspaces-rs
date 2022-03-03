use std::convert::TryInto;

use near_crypto::KeyType;
use near_primitives::views::AccountView;

use crate::rpc::client::{DEFAULT_CALL_DEPOSIT, DEFAULT_CALL_FN_GAS};
use crate::types::{AccountId, Balance, Gas, InMemorySigner, SecretKey};
use crate::{CryptoHash, Network, Worker};

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
    /// a Builder object that we will make use to populate the rest of the
    /// call details.
    pub fn call<'a, T: Network>(
        &self,
        worker: &'a Worker<T>,
        contract_id: &AccountId,
        function: &str,
    ) -> CallBuilder<'a, T> {
        CallBuilder::new(
            worker,
            contract_id.to_owned(),
            self.signer.clone(),
            function.into(),
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

    /// Create a new sub account. Returns a CreateAccountBuilder object that
    /// we can make use of to fill out the rest of the details. The sub account
    /// id will be in the form of: "{new_account_id}.{parent_account_id}"
    pub fn create_subaccount<'a, 'b, T: Network>(
        &self,
        worker: &'a Worker<T>,
        new_account_id: &'b str,
    ) -> CreateAccountBuilder<'a, 'b, T> {
        CreateAccountBuilder::new(
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
        function: &str,
    ) -> CallBuilder<'a, T> {
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
}

pub struct CallBuilder<'a, T> {
    worker: &'a Worker<T>,
    signer: InMemorySigner,
    contract_id: AccountId,

    function: String,
    args: Vec<u8>,
    deposit: Balance,
    gas: Gas,
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
            args: serde_json::json!({}).to_string().into_bytes(),
            deposit: DEFAULT_CALL_DEPOSIT,
            gas: DEFAULT_CALL_FN_GAS,
        }
    }

    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.args = args;
        self
    }

    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> anyhow::Result<Self> {
        self.args = serde_json::to_vec(&args)?;
        Ok(self)
    }

    pub fn args_borsh<U: borsh::BorshSerialize>(mut self, args: U) -> anyhow::Result<Self> {
        self.args = args.try_to_vec()?;
        Ok(self)
    }

    pub fn deposit(mut self, deposit: u128) -> Self {
        self.deposit = deposit;
        self
    }

    pub fn gas(mut self, gas: u64) -> Self {
        self.gas = gas;
        self
    }

    pub async fn transact(self) -> anyhow::Result<CallExecutionDetails> {
        self.worker
            .client()
            .call(
                &self.signer,
                &self.contract_id,
                self.function,
                self.args,
                self.gas,
                self.deposit,
            )
            .await
            .and_then(CallExecutionDetails::from_outcome)
    }

    pub async fn view(self) -> anyhow::Result<ViewResultDetails> {
        self.worker
            .client()
            .view(self.contract_id, self.function, self.args)
            .await
    }
}

pub struct CreateAccountBuilder<'a, 'b, T> {
    worker: &'a Worker<T>,
    signer: InMemorySigner,
    parent_id: AccountId,
    new_account_id: &'b str,

    initial_balance: Balance,
    secret_key: Option<SecretKey>,
}

impl<'a, 'b, T> CreateAccountBuilder<'a, 'b, T>
where
    T: Network,
{
    fn new(
        worker: &'a Worker<T>,
        signer: InMemorySigner,
        parent_id: AccountId,
        new_account_id: &'b str,
    ) -> Self {
        Self {
            worker,
            signer,
            parent_id,
            new_account_id,
            initial_balance: 100000000000000000000000,
            secret_key: None,
        }
    }

    pub fn initial_balance(mut self, initial_balance: Balance) -> Self {
        self.initial_balance = initial_balance;
        self
    }

    pub fn keys(mut self, secret_key: SecretKey) -> Self {
        self.secret_key = Some(secret_key);
        self
    }

    pub async fn transact(self) -> anyhow::Result<CallExecution<Account>> {
        let sk = self
            .secret_key
            .unwrap_or_else(|| SecretKey::from_seed(KeyType::ED25519, "subaccount.seed"));
        let id: AccountId = format!("{}.{}", self.new_account_id, self.parent_id).try_into()?;

        let outcome = self
            .worker
            .client()
            .create_account(&self.signer, &id, sk.public_key(), self.initial_balance)
            .await?;

        let signer = InMemorySigner::from_secret_key(id.clone(), sk);
        let account = Account::new(id, signer);

        Ok(CallExecution {
            result: account,
            details: outcome.into(),
        })
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
