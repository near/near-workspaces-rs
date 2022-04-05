use crate::network::NetworkClient;
use crate::network::{Mainnet, Sandbox, Testnet};
use crate::result::{CallExecutionDetails, ViewResultDetails};
use crate::rpc::client::{DEFAULT_CALL_DEPOSIT, DEFAULT_CALL_FN_GAS};
use crate::types::{AccountId, Gas, InMemorySigner};
use crate::AccountDetails;
use crate::{Block, Contract};
use near_primitives::types::{Balance, StoreKey};
use std::collections::HashMap;

macro_rules! impl_helper_functions {
    ($network:ty) => {
        impl $network {
            /// Call into a contract's change function.
            pub async fn call(
                &self,
                contract: &Contract<$network>,
                function: &str,
                args: Vec<u8>,
                gas: Option<Gas>,
                deposit: Option<Balance>,
            ) -> anyhow::Result<CallExecutionDetails> {
                self.client()
                    .call(
                        contract.signer(),
                        contract.id(),
                        function.into(),
                        args,
                        gas.unwrap_or(DEFAULT_CALL_FN_GAS),
                        deposit.unwrap_or(DEFAULT_CALL_DEPOSIT),
                    )
                    .await
                    .and_then(CallExecutionDetails::from_outcome)
            }

            /// Call into a contract's view function.
            pub async fn view(
                &self,
                contract_id: &AccountId,
                function: &str,
                args: Vec<u8>,
            ) -> anyhow::Result<ViewResultDetails> {
                self.client()
                    .view(&contract_id, function.into(), args)
                    .await
            }

            /// View the WASM code bytes of a contract on the network.
            pub async fn view_code(&self, contract_id: &AccountId) -> anyhow::Result<Vec<u8>> {
                let code_view = self.client().view_code(&contract_id, None).await?;
                Ok(code_view.code)
            }

            /// View the state of a account/contract on the network. This will return the internal
            /// state of the account in the form of a map of key-value pairs; where STATE contains
            /// info on a contract's internal data.
            pub async fn view_state(
                &self,
                contract_id: &AccountId,
                prefix: Option<StoreKey>,
            ) -> anyhow::Result<HashMap<String, Vec<u8>>> {
                self.client().view_state(contract_id.clone(), prefix).await
            }

            /// View the latest block from the network
            pub async fn view_latest_block(&self) -> anyhow::Result<Block> {
                self.client().view_block(None).await.map(Into::into)
            }

            /// Transfer tokens from one account to another. The signer is the account
            /// that will be used to to send from.
            pub async fn transfer_near(
                &self,
                signer: &InMemorySigner,
                receiver_id: &AccountId,
                amount_yocto: Balance,
            ) -> anyhow::Result<CallExecutionDetails> {
                self.client()
                    .transfer_near(signer, receiver_id, amount_yocto)
                    .await
                    .and_then(CallExecutionDetails::from_outcome)
            }

            /// Deletes an account from the network. The beneficiary will receive the balance
            /// of the account deleted.
            pub async fn delete_account(
                &self,
                account_id: &AccountId,
                signer: &InMemorySigner,
                beneficiary_id: &AccountId,
            ) -> anyhow::Result<CallExecutionDetails> {
                self.client()
                    .delete_account(signer, account_id, beneficiary_id)
                    .await
                    .and_then(CallExecutionDetails::from_outcome)
            }

            /// View account details of a specific account on the network.
            pub async fn view_account(
                &self,
                account_id: &AccountId,
            ) -> anyhow::Result<AccountDetails> {
                self.client()
                    .view_account(&account_id, None)
                    .await
                    .map(Into::into)
            }
        }
    };
}

impl_helper_functions!(Sandbox);
impl_helper_functions!(Testnet);
impl_helper_functions!(Mainnet);
