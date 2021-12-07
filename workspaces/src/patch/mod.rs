use near_crypto::{KeyType, Signer};
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::{types::Balance, state_record::StateRecord, account::AccessKey, hash::CryptoHash};

use crate::{AccountId, network::DEV_ACCOUNT_SEED, InMemorySigner, Contract, rpc::client::Client};


pub struct ImportContractBuilder<'a, 'b> {
    account_id: AccountId,
    from_network: &'a Client,
    into_network: &'b Client,

    /// Whether to grab data down from the other contract or not
    import_data: bool,

    /// Initial balance of the account. If None, uses what is specified
    /// from the other account instead.
    initial_balance: Option<Balance>,
}

impl<'a, 'b> ImportContractBuilder<'a, 'b> {
    pub(crate) fn new(account_id: AccountId, from_network: &'a Client, into_network: &'b Client,) -> Self {
        ImportContractBuilder {
            account_id,
            from_network,
            into_network,
            import_data: false,
            initial_balance: None,
        }
    }

    pub fn with_data(mut self) -> Self {
        self.import_data = true;
        self
    }

    pub fn with_initial_balance(mut self, initial_balance: Balance) -> Self {
        self.initial_balance = Some(initial_balance);
        self
    }

    pub async fn transact(self) -> anyhow::Result<Contract> {
        let signer = InMemorySigner::from_seed(self.account_id.clone(), KeyType::ED25519, DEV_ACCOUNT_SEED);

        let mut account_view = self.from_network.view_account(self.account_id.clone(), None).await?;
        if let Some(initial_balance) = self.initial_balance {
            println!("Setting initial balance to {}", initial_balance);
            account_view.amount = initial_balance;
        }

        let mut records = vec![
            StateRecord::Account {
                account_id: self.account_id.clone().into(),
                account: account_view.clone().into(),
            },
            StateRecord::AccessKey {
                account_id: self.account_id.clone().into(),
                public_key: signer.public_key(),
                access_key: AccessKey::full_access(),
            }
        ];

        if account_view.code_hash != CryptoHash::default() {
            let code_view = self.from_network.view_code(self.account_id.clone(), None).await?;
            records.push(StateRecord::Contract {
                account_id: self.account_id.clone().into(),
                code: code_view.code,
            });
        }

        if self.import_data {
            records.extend(
                self.from_network
                    .view_state_raw(self.account_id.clone(), None, None)
                    .await?
                    .into_iter()
                    .map(|(key, value)| StateRecord::Data {
                        account_id: self.account_id.clone().into(),
                        data_key: key.into(),
                        value,
                    })
            );
        }

        // NOTE: For some reason, patching anything with account/contract related items takes two patches
        // otherwise its super non-deterministic and mostly just fails to locate the account afterwards: ¯\_(ツ)_/¯
        self.into_network
            .query(&RpcSandboxPatchStateRequest { records: records.clone() })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        self.into_network
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        Ok(Contract::new(self.account_id, signer))
    }
}
