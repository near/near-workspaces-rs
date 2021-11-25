use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use async_trait::async_trait;

use near_crypto::{InMemorySigner, Signer};
use near_primitives::types::{AccountId, Balance};

use crate::rpc::client::Client;
use crate::runtime::local::SandboxServer;
use crate::NEAR_BASE;

use super::{
    Account, AllowDevAccountCreation, AllowStatePatching, CallExecution, Contract, NetworkActions,
    NetworkClient, NetworkInfo, TopLevelAccountCreator,
};

const DEFAULT_DEPOSIT: Balance = 100 * NEAR_BASE;

pub struct Sandbox {
    server: SandboxServer,
    client: Client,
}

impl Sandbox {
    fn home_dir(port: u16) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("sandbox-{}", port));
        path
    }

    fn root_signer(&self) -> InMemorySigner {
        let mut path = Self::home_dir(self.server.rpc_port);
        path.push("validator_key.json");

        InMemorySigner::from_file(&path)
    }

    pub(crate) fn new() -> Self {
        let mut server = SandboxServer::default();
        server.start().unwrap();

        let client = Client::new(server.rpc_addr());
        Self { server, client }
    }
}

impl AllowStatePatching for Sandbox {}

impl AllowDevAccountCreation for Sandbox {}

#[async_trait]
impl TopLevelAccountCreator for Sandbox {
    async fn create_tla(
        &self,
        id: AccountId,
        signer: InMemorySigner,
    ) -> anyhow::Result<CallExecution<Account>> {
        let root_signer = self.root_signer();
        let outcome = self
            .client
            .create_account(
                &root_signer,
                id.clone(),
                signer.public_key(),
                DEFAULT_DEPOSIT,
            )
            .await?;

        Ok(CallExecution {
            result: Account::new(id, signer),
            details: outcome.into(),
        })
    }

    async fn create_tla_and_deploy<P: AsRef<Path> + Send + Sync>(
        &self,
        id: AccountId,
        signer: InMemorySigner,
        wasm: P,
    ) -> anyhow::Result<CallExecution<Contract>> {
        let root_signer = self.root_signer();
        // TODO: async_compat/async version of File
        let mut code = Vec::new();
        File::open(wasm)?.read_to_end(&mut code)?;

        let outcome = self
            .client
            .create_account_and_deploy(
                &root_signer,
                id.clone(),
                signer.public_key(),
                DEFAULT_DEPOSIT,
                code,
            )
            .await?;

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

impl NetworkActions for Sandbox {}

impl NetworkInfo for Sandbox {
    fn name(&self) -> String {
        "sandbox".into()
    }

    fn root_account_id(&self) -> AccountId {
        AccountId::from_str("test.near").unwrap()
    }

    fn keystore_path(&self) -> PathBuf {
        PathBuf::from(".near-credentials/sandbox/")
    }

    fn rpc_url(&self) -> String {
        self.server.rpc_addr()
    }

    fn helper_url(&self) -> String {
        todo!()
    }
}
