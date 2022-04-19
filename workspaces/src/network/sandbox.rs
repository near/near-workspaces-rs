use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;
use near_jsonrpc_client::methods::sandbox_fast_forward::RpcSandboxFastForwardRequest;
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::state_record::StateRecord;
use std::iter::IntoIterator;

use super::{AllowDevAccountCreation, NetworkClient, NetworkInfo, TopLevelAccountCreator};
use crate::network::server::SandboxServer;
use crate::network::Info;
use crate::result::CallExecution;
use crate::rpc::client::Client;
use crate::rpc::patch::ImportContractTransaction;
use crate::types::{AccountId, Balance, InMemorySigner, SecretKey};
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

    pub(crate) fn patch_state(&self) -> SandboxPatchStateBuilder {
        SandboxPatchStateBuilder::new(self)
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

//todo: review naming
#[must_use = "don't forget to .send() this `PatchStateBuilder`"]
pub struct SandboxPatchStateBuilder<'s> {
    sandbox: &'s Sandbox,
    records: Vec<StateRecord>,
}

//todo: add more methods
impl<'s> SandboxPatchStateBuilder<'s> {
    pub fn new(sandbox: &'s Sandbox) -> Self {
        SandboxPatchStateBuilder {
            sandbox,
            records: Vec::with_capacity(4),
        }
    }

    pub fn data(
        mut self,
        contract_id: &AccountId, //todo: borrowed or owned? (or Into<> or smth)
        key: impl Into<Vec<u8>>,
        value: impl Into<Vec<u8>>,
    ) -> Self {
        let state = StateRecord::Data {
            account_id: contract_id.to_owned(),
            data_key: key.into(),
            value: value.into(),
        };

        self.records.push(state);
        self
    }

    pub fn data_many(
        mut self,
        contract_id: &AccountId, //todo: borrowed or owned? (or Into<> or smth)
        kvs: impl IntoIterator<Item = (impl Into<Vec<u8>>, impl Into<Vec<u8>>)>,
    ) -> Self {
        self.records
            .extend(kvs.into_iter().map(|(key, value)| StateRecord::Data {
                account_id: contract_id.to_owned(),
                data_key: key.into(),
                value: value.into(),
            }));
        self
    }

    pub async fn send(self) -> anyhow::Result<()> {
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
