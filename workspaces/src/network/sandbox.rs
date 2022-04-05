use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;
use near_jsonrpc_client::methods::sandbox_fast_forward::RpcSandboxFastForwardRequest;
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::state_record::StateRecord;

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

pub struct Sandbox {
    server: SandboxServer,
    client: Client,
    info: Info,
}

impl Worker<Sandbox> {
    fn root_signer(&self) -> InMemorySigner {
        self.workspace.root_signer()
    }
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
impl TopLevelAccountCreator for Worker<Sandbox> {
    async fn create_tla(
        &self,
        id: AccountId,
        sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account<Sandbox>>> {
        let root_signer = self.root_signer();
        let outcome = self
            .client()
            .create_account(&root_signer, &id, sk.public_key(), DEFAULT_DEPOSIT)
            .await?;

        let signer = InMemorySigner::from_secret_key(id.clone(), sk);
        Ok(CallExecution {
            result: Account::new(self.clone(), id, signer),
            details: outcome.into(),
        })
    }

    async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract<Sandbox>>> {
        let root_signer = self.root_signer();
        let outcome = self
            .client()
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
            result: Contract::new(self.clone(), id, signer),
            details: outcome.into(),
        })
    }
}

impl NetworkClient for Sandbox {
    type Network = Self;

    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Sandbox {
    fn info(&self) -> &Info {
        &self.info
    }
}

impl Worker<Sandbox> {
    pub fn root_account(&self) -> Account<Sandbox> {
        let account_id = self.info().root_id.clone();
        let signer = self.workspace.root_signer();
        Account::new(self.clone(), account_id, signer)
    }

    /// Import a contract from the the given network, and return us a [`ImportContractTransaction`]
    /// which allows to specify further details, such as being able to import contract data and
    /// how far back in time we wanna grab the contract.
    pub fn import_contract<'a, 'b>(
        &'b self,
        id: &AccountId,
        worker: &'a Worker<impl Network>,
    ) -> ImportContractTransaction<'a, 'b, Sandbox> {
        ImportContractTransaction::new(id.to_owned(), worker.client(), self)
    }

    /// Patch state into the sandbox network, given a key and value. This will allow us to set
    /// state that we have acquired in some manner. This allows us to test random cases that
    /// are hard to come up naturally as state evolves.
    pub async fn patch_state(
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

    /// Fast forward to a point in the future. The delta block height is supplied to tell the
    /// network to advanced a certain amount of blocks. This comes with the advantage only having
    /// to wait a fraction of the time it takes to produce the same number of blocks.
    ///
    /// Estimate as to how long it takes: if our delta_height crosses `X` epochs, then it would
    /// roughly take `X * 5` seconds for the fast forward request to be processed.
    pub async fn fast_forward(&self, delta_height: u64) -> anyhow::Result<()> {
        // NOTE: RpcSandboxFastForwardResponse is an empty struct with no fields, so don't do anything with it:
        self.client()
            // TODO: replace this with the `query` variant when RpcSandboxFastForwardRequest impls Debug
            .query_nolog(&RpcSandboxFastForwardRequest { delta_height })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to fast forward: {:?}", err))?;

        Ok(())
    }
}
