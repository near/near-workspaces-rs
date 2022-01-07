use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;

use super::{
    Account, AllowDevAccountCreation, AllowStatePatching, CallExecution, Contract, NetworkClient,
    NetworkInfo, TopLevelAccountCreator,
};

use crate::network::server::SandboxServer;
use crate::network::Info;
use crate::rpc::client::Client;
use crate::types::{AccountId, Balance, InMemorySigner, SecretKey};

// Constant taken from nearcore crate to avoid dependency
pub(crate) const NEAR_BASE: Balance = 1_000_000_000_000_000_000_000_000;

const DEFAULT_DEPOSIT: Balance = 100 * NEAR_BASE;

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

    pub(crate) fn new() -> Self {
        let mut server = SandboxServer::default();
        server.start().unwrap();

        let client = Client::new(server.rpc_addr());
        let info = Info {
            name: "sandbox".to_string(),
            root_id: AccountId::from_str("test.near").unwrap(),
            keystore_path: PathBuf::from(".near-credentials/sandbox/"),
            rpc_url: server.rpc_addr(),
        };

        Self {
            server,
            client,
            info,
        }
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
        wasm: Vec<u8>,
    ) -> anyhow::Result<CallExecution<Contract>> {
        let root_signer = self.root_signer();
        let outcome = self
            .client
            .create_account_and_deploy(&root_signer, &id, sk.public_key(), DEFAULT_DEPOSIT, wasm)
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
