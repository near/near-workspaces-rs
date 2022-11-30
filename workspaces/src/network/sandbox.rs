use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;
use near_jsonrpc_client::methods::sandbox_fast_forward::RpcSandboxFastForwardRequest;
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::state_record::StateRecord;

use super::{AllowDevAccountCreation, NetworkClient, NetworkInfo, TopLevelAccountCreator};
use crate::error::SandboxErrorCode;
use crate::network::server::SandboxServer;
use crate::network::Info;
use crate::result::{Execution, ExecutionFinalResult, Result};
use crate::rpc::client::Client;
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

    pub(crate) fn root_signer(&self) -> Result<InMemorySigner> {
        let mut path = Self::home_dir(self.server.rpc_port);
        path.push("validator_key.json");

        InMemorySigner::from_file(&path)
    }

    pub(crate) async fn new() -> Result<Self> {
        let mut server = SandboxServer::default();
        server.start().await?;
        let client = Client::new(&server.rpc_addr());
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

    /// Port being used by sandbox server for RPC requests.
    pub(crate) fn rpc_port(&self) -> u16 {
        self.server.rpc_port
    }
}

impl std::fmt::Debug for Sandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Sandbox")
            .field("root_id", &self.info.root_id)
            .field("rpc_url", &self.info.rpc_url)
            .field("rpc_port", &self.server.rpc_port)
            .field("net_port", &self.server.net_port)
            .finish()
    }
}

impl AllowDevAccountCreation for Sandbox {}

#[async_trait]
impl TopLevelAccountCreator for Sandbox {
    async fn create_tla(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
    ) -> Result<Execution<Account>> {
        let root_signer = self.root_signer()?;
        let outcome = self
            .client()
            .create_account(&root_signer, &id, sk.public_key(), DEFAULT_DEPOSIT)
            .await?;

        let signer = InMemorySigner::from_secret_key(id, sk);
        Ok(Execution {
            result: Account::new(signer, worker),
            details: ExecutionFinalResult::from_view(outcome),
        })
    }

    async fn create_tla_and_deploy(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>> {
        let root_signer = self.root_signer()?;
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

        let signer = InMemorySigner::from_secret_key(id, sk);
        Ok(Execution {
            result: Contract::new(signer, worker),
            details: ExecutionFinalResult::from_view(outcome),
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
    pub(crate) async fn patch_state(
        &self,
        contract_id: &AccountId,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
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
            .map_err(|e| SandboxErrorCode::PatchStateFailure.custom(e))?;

        Ok(())
    }

    pub(crate) async fn fast_forward(&self, delta_height: u64) -> Result<()> {
        // NOTE: RpcSandboxFastForwardResponse is an empty struct with no fields, so don't do anything with it:
        self.client()
            // TODO: replace this with the `query` variant when RpcSandboxFastForwardRequest impls Debug
            .query_nolog(&RpcSandboxFastForwardRequest { delta_height })
            .await
            .map_err(|e| SandboxErrorCode::FastForwardFailure.custom(e))?;

        Ok(())
    }
}
