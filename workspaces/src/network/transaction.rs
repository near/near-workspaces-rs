use near_primitives::transaction::{
    Action, CreateAccountAction, DeleteAccountAction, DeleteKeyAction, DeployContractAction,
    FunctionCallAction, StakeAction, TransferAction,
};
use near_primitives::views::FinalExecutionOutcomeView;

use crate::rpc::client::{
    send_batch_tx_and_retry, Client, DEFAULT_CALL_DEPOSIT, DEFAULT_CALL_FN_GAS,
};
use crate::types::{AccountId, Balance, Gas, InMemorySigner, PublicKey};

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

    pub fn args(&mut self, args: Vec<u8>) -> &mut Self {
        self.args = args;
        self
    }

    pub fn args_json<U: serde::Serialize>(&mut self, args: U) -> anyhow::Result<&mut Self> {
        self.args = serde_json::to_vec(&args)?;
        Ok(self)
    }

    pub fn args_borsh<U: borsh::BorshSerialize>(&mut self, args: U) -> anyhow::Result<&mut Self> {
        self.args = args.try_to_vec()?;
        Ok(self)
    }

    pub fn deposit(&mut self, deposit: u128) -> &mut Self {
        self.deposit = deposit;
        self
    }

    pub fn gas(&mut self, gas: u64) -> &mut Self {
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

pub trait IntoCallArgs {
    fn into_call_args(self) -> CallArgs;
}

impl IntoCallArgs for &str {
    fn into_call_args(self) -> CallArgs {
        CallArgs::new(self)
    }
}

impl IntoCallArgs for CallArgs {
    fn into_call_args(self) -> CallArgs {
        self
    }
}

pub struct Transaction<'a> {
    client: &'a Client,
    signer: InMemorySigner,
    receiver_id: AccountId,
    actions: Vec<Action>,
}

impl<'a> Transaction<'a> {
    pub fn new(client: &'a Client, signer: InMemorySigner, receiver_id: AccountId) -> Self {
        Self {
            client,
            signer,
            receiver_id,
            actions: Vec::new(),
        }
    }

    // TODO(chore): expose our own AccessKey type
    // pub fn add_key(mut self, pk: PublicKey, access_key: AccessKey) -> Self {
    //     self.actions.push(AddKeyAction { public_key: pk.into(), access_key }.into());
    //     self
    // }

    pub fn call(mut self, call_args: impl IntoCallArgs) -> Self {
        self.actions.push(call_args.into_call_args().into());
        self
    }

    pub fn create_account(mut self) -> Self {
        self.actions.push(CreateAccountAction {}.into());
        self
    }

    pub fn delete_account(mut self, beneficiary_id: AccountId) -> Self {
        self.actions
            .push(DeleteAccountAction { beneficiary_id }.into());
        self
    }

    pub fn delete_key(mut self, pk: PublicKey) -> Self {
        self.actions
            .push(DeleteKeyAction { public_key: pk.0 }.into());
        self
    }

    pub fn deploy(mut self, code: Vec<u8>) -> Self {
        self.actions.push(DeployContractAction { code }.into());
        self
    }

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

    pub fn transfer(mut self, deposit: Balance) -> Self {
        self.actions.push(TransferAction { deposit }.into());
        self
    }

    pub async fn transact(self) -> anyhow::Result<FinalExecutionOutcomeView> {
        send_batch_tx_and_retry(&self.client, &self.signer, &self.receiver_id, self.actions).await
    }
}
