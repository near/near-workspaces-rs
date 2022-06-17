//! All operation types that are generated/used when making transactions or view calls.

use crate::error::{Error, RpcError, SerializationError};
use crate::result::{CallExecution, CallExecutionDetails, ViewResultDetails};
use crate::rpc::client::{
    send_batch_tx_and_retry, Client, DEFAULT_CALL_DEPOSIT, DEFAULT_CALL_FN_GAS,
};
use crate::types::{
    AccessKey, AccountId, Balance, Gas, InMemorySigner, KeyType, PublicKey, SecretKey,
};
use crate::worker::Worker;
use crate::{Account, Network};

use near_account_id::ParseAccountError;
use near_primitives::transaction::{
    Action, AddKeyAction, CreateAccountAction, DeleteAccountAction, DeleteKeyAction,
    DeployContractAction, FunctionCallAction, StakeAction, TransferAction,
};
use near_primitives::views::FinalExecutionOutcomeView;
use std::convert::TryInto;

const MAX_GAS: Gas = 300_000_000_000_000;

/// A set of arguments we can provide to a transaction, containing
/// the function name, arguments, the amount of gas to use and deposit.
#[derive(Debug, Clone)]
pub struct Function<'a> {
    name: &'a str,
    args: Vec<u8>,
    deposit: Balance,
    gas: Gas,
}

impl<'a> Function<'a> {
    /// Initialize a new instance of [`Function`], tied to a specific function on a
    /// contract that lives directly on a contract we've specified in [`Transaction`].
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            args: vec![],
            deposit: DEFAULT_CALL_DEPOSIT,
            gas: DEFAULT_CALL_FN_GAS,
        }
    }

    /// Provide the arguments for the call. These args are serialized bytes from either
    /// a JSON or Borsh serializable set of arguments. To use the more specific versions
    /// with better quality of life, use `args_json` or `args_borsh`.
    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.args = args;
        self
    }

    /// Similiar to `args`, specify an argument that is JSON serializable and can be
    /// accepted by the equivalent contract. Recommend to use something like
    /// `serde_json::json!` macro to easily serialize the arguments.
    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> Result<Self, SerializationError> {
        self.args = serde_json::to_vec(&args)?;
        Ok(self)
    }

    /// Similiar to `args`, specify an argument that is borsh serializable and can be
    /// accepted by the equivalent contract.
    pub fn args_borsh<U: borsh::BorshSerialize>(
        mut self,
        args: U,
    ) -> Result<Self, SerializationError> {
        self.args = args.try_to_vec().map_err(SerializationError::BorshError)?;
        Ok(self)
    }

    /// Specify the amount of tokens to be deposited where `deposit` is the amount of
    /// tokens in yocto near.
    pub fn deposit(mut self, deposit: Balance) -> Self {
        self.deposit = deposit;
        self
    }

    /// Specify the amount of gas to be used.
    pub fn gas(mut self, gas: Gas) -> Self {
        self.gas = gas;
        self
    }

    /// Use the maximum amount of gas possible to perform this function call into the contract.
    pub fn max_gas(self) -> Self {
        self.gas(MAX_GAS)
    }
}

impl From<Function<'_>> for Action {
    fn from(function: Function<'_>) -> Self {
        Self::FunctionCall(FunctionCallAction {
            method_name: function.name.to_string(),
            args: function.args,
            deposit: function.deposit,
            gas: function.gas,
        })
    }
}

/// A builder-like object that will allow specifying various actions to be performed
/// in a single transaction. For details on each of the actions, find them in
/// [NEAR transactions](https://docs.near.org/docs/concepts/transaction).
pub struct Transaction<'a> {
    client: &'a Client,
    signer: InMemorySigner,
    receiver_id: AccountId,
    actions: Vec<Action>,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(client: &'a Client, signer: InMemorySigner, receiver_id: AccountId) -> Self {
        Self {
            client,
            signer,
            receiver_id,
            actions: Vec::new(),
        }
    }

    /// Adds a key to the `receiver_id`'s account, where the public key can be used
    /// later to delete the same key.
    pub fn add_key(mut self, pk: PublicKey, ak: AccessKey) -> Self {
        self.actions.push(
            AddKeyAction {
                public_key: pk.into(),
                access_key: ak.into(),
            }
            .into(),
        );
        self
    }

    /// Call into the `receiver_id`'s contract with the specific function arguments.
    pub fn call(mut self, function: Function) -> Self {
        self.actions.push(function.into());
        self
    }

    /// Create a new account with the account id being `receiver_id`.
    pub fn create_account(mut self) -> Self {
        self.actions.push(CreateAccountAction {}.into());
        self
    }

    /// Deletes the `receiver_id`'s account. The beneficiary specified by
    /// `beneficiary_id` will receive the funds of the account deleted.
    pub fn delete_account(mut self, beneficiary_id: &AccountId) -> Self {
        self.actions.push(
            DeleteAccountAction {
                beneficiary_id: beneficiary_id.clone(),
            }
            .into(),
        );
        self
    }

    /// Deletes a key from the `receiver_id`'s account, where the public key is
    /// associated with the access key to be deleted.
    pub fn delete_key(mut self, pk: PublicKey) -> Self {
        self.actions
            .push(DeleteKeyAction { public_key: pk.0 }.into());
        self
    }

    /// Deploy contract code or WASM bytes to the `receiver_id`'s account.
    pub fn deploy(mut self, code: &[u8]) -> Self {
        self.actions
            .push(DeployContractAction { code: code.into() }.into());
        self
    }

    /// An action which stakes the signer's tokens and setups a validator public key.
    pub fn stake(mut self, stake: Balance, pk: PublicKey) -> Self {
        self.actions.push(
            StakeAction {
                stake,
                public_key: pk.0,
            }
            .into(),
        );
        self
    }

    /// Transfer `deposit` amount from `signer`'s account into `receiver_id`'s account.
    pub fn transfer(mut self, deposit: Balance) -> Self {
        self.actions.push(TransferAction { deposit }.into());
        self
    }

    async fn transact_raw(self) -> Result<FinalExecutionOutcomeView, RpcError> {
        send_batch_tx_and_retry(self.client, &self.signer, &self.receiver_id, self.actions).await
    }

    /// Process the trannsaction, and return the result of the execution.
    pub async fn transact(self) -> crate::result::Result<CallExecutionDetails> {
        self.transact_raw()
            .await
            .map_err(crate::error::Error::from)
            .and_then(CallExecutionDetails::from_outcome)
    }
}

/// Similiar to a [`Transaction`], but more specific to making a call into a contract.
/// Note, only one call can be made per `CallTransaction`.
pub struct CallTransaction<'a, 'b, T> {
    worker: &'a Worker<T>,
    signer: InMemorySigner,
    contract_id: AccountId,
    function: Function<'b>,
}

impl<'a, 'b, T: Network> CallTransaction<'a, 'b, T> {
    pub(crate) fn new(
        worker: &'a Worker<T>,
        contract_id: AccountId,
        signer: InMemorySigner,
        function: &'b str,
    ) -> Self {
        Self {
            worker,
            signer,
            contract_id,
            function: Function::new(function),
        }
    }

    /// Provide the arguments for the call. These args are serialized bytes from either
    /// a JSON or Borsh serializable set of arguments. To use the more specific versions
    /// with better quality of life, use `args_json` or `args_borsh`.
    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.function = self.function.args(args);
        self
    }

    /// Similiar to `args`, specify an argument that is JSON serializable and can be
    /// accepted by the equivalent contract. Recommend to use something like
    /// `serde_json::json!` macro to easily serialize the arguments.
    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> Result<Self, SerializationError> {
        self.function = self.function.args_json(args)?;
        Ok(self)
    }

    /// Similiar to `args`, specify an argument that is borsh serializable and can be
    /// accepted by the equivalent contract.
    pub fn args_borsh<U: borsh::BorshSerialize>(
        mut self,
        args: U,
    ) -> Result<Self, SerializationError> {
        self.function = self.function.args_borsh(args)?;
        Ok(self)
    }

    /// Specify the amount of tokens to be deposited where `deposit` is the amount of
    /// tokens in yocto near.
    pub fn deposit(mut self, deposit: u128) -> Self {
        self.function = self.function.deposit(deposit);
        self
    }

    /// Specify the amount of gas to be used where `gas` is the amount of gas in yocto near.
    pub fn gas(mut self, gas: u64) -> Self {
        self.function = self.function.gas(gas);
        self
    }

    /// Use the maximum amount of gas possible to perform this transaction.
    pub fn max_gas(self) -> Self {
        self.gas(MAX_GAS)
    }

    /// Finally, send the transaction to the network. This will consume the `CallTransaction`
    /// object and return us the execution details, along with any errors if the transaction
    /// failed in any process along the way.
    pub async fn transact(self) -> crate::result::Result<CallExecutionDetails> {
        self.worker
            .client()
            .call(
                &self.signer,
                &self.contract_id,
                self.function.name.to_string(),
                self.function.args,
                self.function.gas,
                self.function.deposit,
            )
            .await
            .map_err(crate::error::Error::from)
            .and_then(CallExecutionDetails::from_outcome)
    }

    /// Instead of transacting the transaction, call into the specified view function.
    pub async fn view(self) -> crate::result::Result<ViewResultDetails> {
        self.worker
            .client()
            .view(
                self.contract_id,
                self.function.name.to_string(),
                self.function.args,
            )
            .await
            .map_err(crate::error::Error::from)
    }
}

/// Similiar to a [`Transaction`], but more specific to creating an account.
/// This transaction will create a new account with the specified `receiver_id`
pub struct CreateAccountTransaction<'a, 'b, T> {
    worker: &'a Worker<T>,
    signer: InMemorySigner,
    parent_id: AccountId,
    new_account_id: &'b str,

    initial_balance: Balance,
    secret_key: Option<SecretKey>,
}

impl<'a, 'b, T> CreateAccountTransaction<'a, 'b, T>
where
    T: Network,
{
    pub(crate) fn new(
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

    /// Specifies the initial balance of the new account. Amount directly taken out
    /// from the caller/signer of this transaction.
    pub fn initial_balance(mut self, initial_balance: Balance) -> Self {
        self.initial_balance = initial_balance;
        self
    }

    /// Set the secret key of the new account.
    pub fn keys(mut self, secret_key: SecretKey) -> Self {
        self.secret_key = Some(secret_key);
        self
    }

    /// Send the transaction to the network. This will consume the `CreateAccountTransaction`
    /// and give us back the details of the execution and finally the new [`Account`] object.
    pub async fn transact(self) -> crate::result::Result<CallExecution<Account>> {
        let sk = self
            .secret_key
            .unwrap_or_else(|| SecretKey::from_seed(KeyType::ED25519, "subaccount.seed"));
        let id: AccountId = format!("{}.{}", self.new_account_id, self.parent_id)
            .try_into()
            .map_err(|e: ParseAccountError| Error::AccountError(e.to_string()))?;

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
