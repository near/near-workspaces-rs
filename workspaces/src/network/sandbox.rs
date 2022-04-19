use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;
use near_jsonrpc_client::methods::sandbox_fast_forward::RpcSandboxFastForwardRequest;
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::account::AccessKey;
use near_primitives::hash::CryptoHash;
use near_primitives::state_record::StateRecord;
use near_primitives::types::StorageUsage;
use near_primitives::views::AccountView;

use super::{AllowDevAccountCreation, NetworkClient, NetworkInfo, TopLevelAccountCreator};
use crate::network::server::SandboxServer;
use crate::network::Info;
use crate::result::CallExecution;
use crate::rpc::client::Client;
use crate::rpc::patch::ImportContractTransaction;
use crate::types::{AccountId, Balance, InMemorySigner, PublicKey, SecretKey};
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

    pub(crate) async fn patch_state(
        &self,
        contract_id: &AccountId,
        key: &[u8],
        value: &[u8],
    ) -> anyhow::Result<()> {
        let state = StateRecord::Data {
            account_id: contract_id.to_owned(),
            data_key: key.to_vec(),
            value: value.to_vec(),
        };
        let records = vec![state];

        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        Ok(())
    }

    pub(crate) async fn patch_account(
        &self,
        account_id: &AccountId,
        amount: Option<Balance>,
        locked: Option<Balance>,
        code_hash: Option<CryptoHash>,
        storage_usage: Option<StorageUsage>,
    ) -> anyhow::Result<()> {
        let AccountView {
            amount: current_amount,
            locked: current_locked,
            code_hash: current_code_hash,
            storage_usage: current_storage_usage,
            ..
        } = self
            .client()
            .view_account(account_id.clone(), None)
            .await
            .map_err(|err| anyhow::anyhow!("Failed to read account: {:?}", err))?;

        let account = StateRecord::Account {
            account_id: account_id.clone(),
            account: near_primitives::account::Account::new(
                amount.unwrap_or(current_amount),
                locked.unwrap_or(current_locked),
                code_hash.unwrap_or(current_code_hash),
                storage_usage.unwrap_or(current_storage_usage),
            ),
        };
        let records = vec![account];

        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch account: {:?}", err))?;

        Ok(())
    }

    pub(crate) async fn patch_access_key(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
        access_key: &AccessKey,
    ) -> anyhow::Result<()> {
        let access_key = StateRecord::AccessKey {
            account_id: account_id.clone(),
            public_key: public_key.0.clone(),
            access_key: access_key.clone(),
        };
        let records = vec![access_key];

        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        Ok(())
    }

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
