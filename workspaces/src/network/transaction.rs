use near_primitives::transaction::{
    Action, AddKeyAction, CreateAccountAction, DeleteAccountAction, DeleteKeyAction,
    DeployContractAction, FunctionCallAction, StakeAction, TransferAction,
};
use near_primitives::views::FinalExecutionOutcomeView;

use crate::rpc::client::{
    send_batch_tx_and_retry, Client, DEFAULT_CALL_DEPOSIT, DEFAULT_CALL_FN_GAS,
};
use crate::types::{AccessKey, AccountId, Balance, Gas, InMemorySigner, PublicKey};

use super::CallExecutionDetails;

#[derive(Debug, Clone)]
pub struct CallArgs {
    pub function: String,
    pub args: Vec<u8>,
    pub deposit: Balance,
    pub gas: Gas,
}

impl CallArgs {
    pub fn new(function: &str) -> Self {
        Self {
            function: function.into(),
            args: Vec::new(),
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
}

impl From<CallArgs> for Action {
    fn from(args: CallArgs) -> Self {
        Self::FunctionCall(FunctionCallAction {
            method_name: args.function,
            args: args.args,
            deposit: args.deposit,
            gas: args.gas,
        })
    }
}

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
    pub fn call(mut self, call_args: CallArgs) -> Self {
        self.actions.push(call_args.into());
        self
    }

    /// Create a new account with the account id being `receiver_id`.
    pub fn create_account(mut self) -> Self {
        self.actions.push(CreateAccountAction {}.into());
        self
    }

    /// Deletes the `receiver_id`'s account. The beneficiary specified by
    /// `beneficiary_id` will receive the funds of the account deleted.
    pub fn delete_account(mut self, beneficiary_id: AccountId) -> Self {
        self.actions
            .push(DeleteAccountAction { beneficiary_id }.into());
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
    pub fn deploy(mut self, code: Vec<u8>) -> Self {
        self.actions.push(DeployContractAction { code }.into());
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

    async fn transact_raw(self) -> anyhow::Result<FinalExecutionOutcomeView> {
        send_batch_tx_and_retry(self.client, &self.signer, &self.receiver_id, self.actions).await
    }

    /// Process the trannsaction, and return the result of the execution.
    pub async fn transact(self) -> anyhow::Result<CallExecutionDetails> {
        self.transact_raw().await.map(Into::into)
    }
}
