use super::{
    Account, AllowDevAccountCreation, AllowStatePatching, CallExecution, Contract, NetworkClient,
    NetworkInfo, TopLevelAccountCreator,
};
use crate::network::Info;
use crate::rpc::client::Client;
use crate::types::{AccountId, Balance, InMemorySigner, SecretKey};
use async_trait::async_trait;
use std::path::PathBuf;

// Constant taken from nearcore crate to avoid dependency
pub(crate) const NEAR_BASE: Balance = 1_000_000_000_000_000_000_000_000;

pub(crate) const DEFAULT_DEPOSIT: Balance = 100 * NEAR_BASE;

#[cfg(not(feature = "sandbox-multi"))]
pub struct Sandbox(crate::network::SandboxShared);

#[cfg(feature = "sandbox-multi")]
pub struct Sandbox(crate::network::SandboxMulti);

impl Sandbox {
    pub(crate) fn home_dir(port: u16) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("sandbox-{}", port));
        path
    }

    pub(crate) fn root_signer(port: u16) -> InMemorySigner {
        let mut path = Self::home_dir(port);
        path.push("validator_key.json");

        InMemorySigner::from_file(&path)
    }

    #[cfg(not(feature = "sandbox-multi"))]
    pub(crate) fn new() -> Self {
        Self(crate::network::SandboxShared::new())
    }

    #[cfg(feature = "sandbox-multi")]
    pub(crate) fn new() -> Self {
        Self(crate::network::SandboxMulti::new())
    }
}

impl AllowStatePatching for Sandbox {}

impl AllowDevAccountCreation for Sandbox {}

#[async_trait]
impl TopLevelAccountCreator for Sandbox {
    async fn create_tla(
        &self,
        id: AccountId,
        sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account>> {
        self.0.create_tla(id, sk).await
    }

    async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract>> {
        self.0.create_tla_and_deploy(id, sk, wasm).await
    }
}

impl NetworkClient for Sandbox {
    fn client(&self) -> &Client {
        self.0.client()
    }
}

impl NetworkInfo for Sandbox {
    fn info(&self) -> &Info {
        self.0.info()
    }
}

pub(crate) trait HasRpcPort {
    fn rpc_port(&self) -> u16;
}

impl HasRpcPort for Sandbox {
    fn rpc_port(&self) -> u16 {
        self.0.rpc_port()
    }
}
