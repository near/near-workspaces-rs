use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;
use near_jsonrpc_client::methods::sandbox_fast_forward::RpcSandboxFastForwardRequest;
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::hash::CryptoHash;
use near_primitives::state_record::StateRecord;
use near_primitives::types::StorageUsage;
use near_primitives::views::AccountView;
use std::iter::IntoIterator;

use super::{AllowDevAccountCreation, NetworkClient, NetworkInfo, TopLevelAccountCreator};
use crate::network::server::SandboxServer;
use crate::network::Info;
use crate::result::CallExecution;
use crate::rpc::client::Client;
use crate::rpc::patch::ImportContractTransaction;
use crate::types::{AccountId, Balance, InMemorySigner, Nonce, SecretKey};
use crate::{Account, Contract, Network, Worker};

// Constant taken from nearcore crate to avoid dependency
pub(crate) const NEAR_BASE: Balance = 1_000_000_000_000_000_000_000_000;

const DEFAULT_DEPOSIT: Balance = 100 * NEAR_BASE;

/// Local sandboxed environment/network, which can be used to test without interacting with
/// networks that are online such as mainnet and testnet. Look at [`workspaces::sandbox`]
/// for how to spin up a sandboxed network and interact with it.
///
/// [`workspaces::sandbox`]: crate::sandbox
pub struct Sandbox {
    server: SandboxServer,
    client: Client,
    info: Info,
}

impl Sandbox {
    pub(crate) fn home_dir(port: u16) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("sandbox-{}", port));
        path
    }

    pub(crate) fn root_signer(&self) -> InMemorySigner {
        let mut path = Self::home_dir(self.server.rpc_port);
        path.push("validator_key.json");

        InMemorySigner::from_file(&path)
    }

    pub(crate) async fn new() -> anyhow::Result<Self> {
        let mut server = SandboxServer::default();
        server.start()?;
        let client = Client::new(server.rpc_addr());
        client.wait_for_rpc().await?;

        let info = Info {
            name: "sandbox".to_string(),
            root_id: AccountId::from_str("test.near").unwrap(),
            keystore_path: PathBuf::from(".near-credentials/sandbox/"),
            rpc_url: server.rpc_addr(),
        };

        Ok(Self {
            server,
            client,
            info,
        })
    }
}

impl AllowDevAccountCreation for Sandbox {}

#[async_trait]
impl TopLevelAccountCreator for Sandbox {
    async fn create_tla(
        &self,
        id: AccountId,
        sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account>> {
        let root_signer = self.root_signer();
        let outcome = self
            .client
            .create_account(&root_signer, &id, sk.public_key(), DEFAULT_DEPOSIT)
            .await?;

        let signer = InMemorySigner::from_secret_key(id.clone(), sk);
        Ok(CallExecution {
            result: Account::new(id, signer),
            details: outcome.into(),
        })
    }

    async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract>> {
        let root_signer = self.root_signer();
        let outcome = self
            .client
            .create_account_and_deploy(
                &root_signer,
                &id,
                sk.public_key(),
                DEFAULT_DEPOSIT,
                wasm.into(),
            )
            .await?;

        let signer = InMemorySigner::from_secret_key(id.clone(), sk);
        Ok(CallExecution {
            result: Contract::new(id, signer),
            details: outcome.into(),
        })
    }
}

impl NetworkClient for Sandbox {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Sandbox {
    fn info(&self) -> &Info {
        &self.info
    }
}

impl Sandbox {
    pub(crate) fn import_contract<'a, 'b>(
        &'b self,
        id: &AccountId,
        worker: &'a Worker<impl Network>,
    ) -> ImportContractTransaction<'a, 'b> {
        ImportContractTransaction::new(id.to_owned(), worker.client(), self.client())
    }

    pub(crate) fn patch_state(&self, account_id: AccountId) -> SandboxPatchStateBuilder {
        SandboxPatchStateBuilder::new(self, account_id)
    }

    pub(crate) fn patch_account(&self, account_id: AccountId) -> SandboxPatchStateAccountBuilder {
        SandboxPatchStateAccountBuilder::new(self, account_id)
    }

    pub(crate) fn patch_access_key(
        &self,
        account_id: AccountId,
        public_key: crate::types::PublicKey,
    ) -> SandboxPatchAcessKeyBuilder {
        SandboxPatchAcessKeyBuilder::new(self, account_id, public_key)
    }

    // shall we expose convenience patch methods here for consistent API?

    pub(crate) async fn fast_forward(&self, delta_height: u64) -> anyhow::Result<()> {
        // NOTE: RpcSandboxFastForwardResponse is an empty struct with no fields, so don't do anything with it:
        self.client()
            // TODO: replace this with the `query` variant when RpcSandboxFastForwardRequest impls Debug
            .query_nolog(&RpcSandboxFastForwardRequest { delta_height })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to fast forward: {:?}", err))?;

        Ok(())
    }
}

//todo: review naming
#[must_use = "don't forget to .apply() this `SandboxPatchStateBuilder`"]
pub struct SandboxPatchStateBuilder<'s> {
    sandbox: &'s Sandbox,
    account_id: AccountId,
    records: Vec<StateRecord>,
}

impl<'s> SandboxPatchStateBuilder<'s> {
    pub fn new(sandbox: &'s Sandbox, account_id: AccountId) -> Self {
        SandboxPatchStateBuilder {
            sandbox,
            account_id,
            records: Vec::with_capacity(4),
        }
    }

    pub fn data(mut self, key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) -> Self {
        let data = StateRecord::Data {
            account_id: self.account_id.clone(),
            data_key: key.into(),
            value: value.into(),
        };

        self.records.push(data);
        self
    }

    pub fn data_multiple(
        mut self,
        kvs: impl IntoIterator<Item = (impl Into<Vec<u8>>, impl Into<Vec<u8>>)>,
    ) -> Self {
        let Self {
            ref mut records,
            ref account_id,
            ..
        } = self;
        records.extend(kvs.into_iter().map(|(key, value)| StateRecord::Data {
            account_id: account_id.clone(),
            data_key: key.into(),
            value: value.into(),
        }));
        self
    }

    // pub fn access_key(mut self, public_key: &PublicKey, access_key: &AccessKey) -> Self {
    //     let access_key = StateRecord::AccessKey {
    //         account_id: self.account_id.clone(),
    //         public_key: public_key.0.clone(),
    //         access_key: access_key.clone(),
    //     };
    //     self.records.push(access_key);
    //     self
    // }

    pub async fn apply(self) -> anyhow::Result<()> {
        let records = self.records;
        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .sandbox
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        Ok(())
    }
}

#[must_use = "don't forget to .apply() this `SandboxPatchStateAccountBuilder`"]
pub struct SandboxPatchStateAccountBuilder<'s> {
    sandbox: &'s Sandbox,
    account_id: AccountId,
    amount: Option<Balance>,
    locked: Option<Balance>,
    code_hash: Option<CryptoHash>,
    storage_usage: Option<StorageUsage>,
}

impl<'s> SandboxPatchStateAccountBuilder<'s> {
    pub const fn new(sandbox: &'s Sandbox, account_id: AccountId) -> Self {
        Self {
            sandbox,
            account_id,
            amount: None,
            locked: None,
            code_hash: None,
            storage_usage: None,
        }
    }

    pub const fn amount(mut self, amount: Balance) -> Self {
        self.amount = Some(amount);
        self
    }

    pub const fn locked(mut self, locked: Balance) -> Self {
        self.locked = Some(locked);
        self
    }

    pub const fn code_hash(mut self, code_hash: CryptoHash) -> Self {
        self.code_hash = Some(code_hash);
        self
    }

    pub const fn storage_usage(mut self, storage_usage: StorageUsage) -> Self {
        self.storage_usage = Some(storage_usage);
        self
    }

    pub async fn apply(self) -> anyhow::Result<()> {
        let account_view = self
            .sandbox
            .client()
            .view_account(self.account_id.clone(), None);

        let AccountView {
            amount: previous_amount,
            locked: previous_locked,
            code_hash: previous_code_hash,
            storage_usage: previous_storage_usage,
            ..
        } = account_view
            .await
            .map_err(|err| anyhow::anyhow!("Failed to read account: {:?}", err))?;

        let account = StateRecord::Account {
            account_id: self.account_id.clone(),
            account: near_primitives::account::Account::new(
                self.amount.unwrap_or(previous_amount),
                self.locked.unwrap_or(previous_locked),
                self.code_hash.unwrap_or(previous_code_hash),
                self.storage_usage.unwrap_or(previous_storage_usage),
            ),
        };

        let records = vec![account];

        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .sandbox
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        Ok(())
    }
}

#[must_use = "don't forget to .apply() this `SandboxPatchStateAccountBuilder`"]
pub struct SandboxPatchAcessKeyBuilder<'s> {
    sandbox: &'s Sandbox,
    account_id: AccountId,
    public_key: crate::types::PublicKey,
    nonce: Nonce,
}

impl<'s> SandboxPatchAcessKeyBuilder<'s> {
    pub const fn new(
        sandbox: &'s Sandbox,
        account_id: AccountId,
        public_key: crate::types::PublicKey,
    ) -> Self {
        Self {
            sandbox,
            account_id,
            public_key,
            nonce: 0,
        }
    }

    pub const fn nonce(mut self, nonce: Nonce) -> Self {
        self.nonce = nonce;
        self
    }

    pub async fn full_access(self) -> anyhow::Result<()> {
        let mut access_key = near_primitives::account::AccessKey::full_access();
        access_key.nonce = self.nonce;
        let access_key = StateRecord::AccessKey {
            account_id: self.account_id,
            public_key: self.public_key.into(),
            access_key,
        };

        let records = vec![access_key];

        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .sandbox
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        Ok(())
    }

    pub async fn function_call(
        self,
        receiver_id: &AccountId,
        method_names: &[&str],
        allowance: Option<Balance>,
    ) -> anyhow::Result<()> {
        let mut access_key: near_primitives::account::AccessKey =
            crate::types::AccessKey::function_call_access(receiver_id, method_names, allowance)
                .into();
        access_key.nonce = self.nonce;
        let access_key = StateRecord::AccessKey {
            account_id: self.account_id,
            public_key: self.public_key.into(),
            access_key,
        };

        let records = vec![access_key];

        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .sandbox
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        Ok(())
    }
}
