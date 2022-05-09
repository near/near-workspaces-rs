use near_crypto::KeyType;
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::{account::AccessKey, state_record::StateRecord, types::Balance};

use crate::network::DEV_ACCOUNT_SEED;
use crate::rpc::client::Client;
use crate::types::{BlockHeight, BlockId, BlockReference, SecretKey};
use crate::{AccountId, Contract, CryptoHash, InMemorySigner};

/// A [`Transaction`]-like object that allows us to specify details about importing
/// a contract from a different network into our sandbox local network. This creates
/// a new [`Transaction`] to be committed to the sandbox network once `transact()`
/// has been called. This does not commit any new transactions from the network
/// this object is importing from.
pub struct ImportContractTransaction<'a, 'b> {
    account_id: AccountId,
    from_network: &'a Client,
    into_network: &'b Client,

    /// Whether to grab data down from the other contract or not
    import_data: bool,

    /// Initial balance of the account. If None, uses what is specified
    /// from the other account instead.
    initial_balance: Option<Balance>,

    block_ref: Option<BlockReference>,
}

impl<'a, 'b> ImportContractTransaction<'a, 'b> {
    pub(crate) fn new(
        account_id: AccountId,
        from_network: &'a Client,
        into_network: &'b Client,
    ) -> Self {
        ImportContractTransaction {
            account_id,
            from_network,
            into_network,
            import_data: false,
            initial_balance: None,
            block_ref: None,
        }
    }

    /// Specify at which block height to import the contract from. This is usable with
    /// any network this object is importing from, but be aware that only archival
    /// networks will have the full history while networks like mainnet or testnet
    /// only has the history from 5 or less epochs ago.
    pub fn block_height(self, block_height: BlockHeight) -> Self {
        self.block_reference(BlockId::Height(block_height).into())
    }

    /// Specify at which block hash to import the contract from. This is usable with
    /// any network this object is importing from, but be aware that only archival
    /// networks will have the full history while networks like mainnet or testnet
    /// only has the history from 5 or less epochs ago.
    pub fn block_hash(self, block_hash: CryptoHash) -> Self {
        self.block_reference(BlockId::Hash(block_hash).into())
    }

    /// Specify at which block reference to import the contract from. This is usable with
    /// any network this object is importing from, but be aware that only archival
    /// networks will have the full history while networks like mainnet or testnet
    /// only has the history from 5 or less epochs ago.
    pub fn block_reference(mut self, block_ref: BlockReference) -> Self {
        self.block_ref = Some(block_ref);
        self
    }

    /// Along with importing the contract code, this will import the state from the
    /// contract itself. This is useful for testing current network state or state
    /// at a specific block. Note that there is a limit of 50mb of state data that
    /// can be pulled down using the usual RPC service. To get beyond this, our own
    /// RPC node has to be spun up and used instead.
    pub fn with_data(mut self) -> Self {
        self.import_data = true;
        self
    }

    /// Specifies the balance of the contract. This will override the balance currently
    /// on the network this transaction is importing from.
    pub fn initial_balance(mut self, initial_balance: Balance) -> Self {
        self.initial_balance = Some(initial_balance);
        self
    }

    /// Process the trannsaction, and return the result of the execution.
    pub async fn transact(self) -> anyhow::Result<Contract> {
        let account_id = self.account_id;
        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
        let pk = sk.public_key();
        let signer = InMemorySigner::from_secret_key(account_id.clone(), sk);
        let block_ref: near_primitives::types::BlockReference =
            self.block_ref.unwrap_or_else(BlockReference::latest).into();

        let mut account_view = self
            .from_network
            .view_account(account_id.clone(), block_ref.clone())
            .await?;
        if let Some(initial_balance) = self.initial_balance {
            account_view.amount = initial_balance;
        }

        let mut records = vec![
            StateRecord::Account {
                account_id: account_id.clone(),
                account: account_view.clone().into(),
            },
            StateRecord::AccessKey {
                account_id: account_id.clone(),
                public_key: pk.clone().into(),
                access_key: AccessKey::full_access(),
            },
        ];

        if account_view.code_hash != near_primitives::hash::CryptoHash::default() {
            let code_view = self
                .from_network
                .view_code(account_id.clone(), block_ref.clone())
                .await?;
            records.push(StateRecord::Contract {
                account_id: account_id.clone(),
                code: code_view.code,
            });
        }

        if self.import_data {
            records.extend(
                self.from_network
                    .view_state(account_id.clone(), None, block_ref)
                    .await?
                    .into_iter()
                    .map(|(key, value)| StateRecord::Data {
                        account_id: account_id.clone(),
                        data_key: key,
                        value,
                    }),
            );
        }

        // NOTE: For some reason, patching anything with account/contract related items takes two patches
        // otherwise its super non-deterministic and mostly just fails to locate the account afterwards: ¯\_(ツ)_/¯
        self.into_network
            .query(&RpcSandboxPatchStateRequest {
                records: records.clone(),
            })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        self.into_network
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        Ok(Contract::new(account_id, signer))
    }
}
