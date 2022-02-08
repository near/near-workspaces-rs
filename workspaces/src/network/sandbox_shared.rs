use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;
use once_cell::sync::Lazy;

use super::{
    Account, AllowDevAccountCreation, AllowStatePatching, CallExecution, Contract, NetworkClient,
    NetworkInfo, TopLevelAccountCreator,
};

use crate::network::server::SandboxServer;
use crate::network::Info;
use crate::rpc::client::Client;
use crate::types::{AccountId, Balance, InMemorySigner, SecretKey};

static SHARED_SERVER: Lazy<SandboxServer> = Lazy::new(|| {
    let mut server = SandboxServer::default();
    server.start().unwrap();
    server
});

// Constant taken from nearcore crate to avoid dependency
pub(crate) const NEAR_BASE: Balance = 1_000_000_000_000_000_000_000_000;

const DEFAULT_DEPOSIT: Balance = 100 * NEAR_BASE;

pub struct SandboxShared {
    client: Client,
    info: Info,
}

impl SandboxShared {
    pub(crate) fn home_dir(port: u16) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("sandbox-{}", port));
        path
    }

    pub(crate) fn root_signer(&self) -> InMemorySigner {
        let mut path = Self::home_dir(SHARED_SERVER.rpc_port);
        path.push("validator_key.json");

        InMemorySigner::from_file(&path)
    }

    pub(crate) fn new() -> Self {
        let client = Client::new(SHARED_SERVER.rpc_addr());
        let info = Info {
            name: "sandbox-shared".to_string(),
            root_id: AccountId::from_str("test.near").unwrap(),
            keystore_path: PathBuf::from(".near-credentials/sandbox/"),
            rpc_url: SHARED_SERVER.rpc_addr(),
        };

        Self { client, info }
    }
}

impl AllowStatePatching for SandboxShared {}

impl AllowDevAccountCreation for SandboxShared {}

#[async_trait]
impl TopLevelAccountCreator for SandboxShared {
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

impl NetworkClient for SandboxShared {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for SandboxShared {
    fn info(&self) -> &Info {
        &self.info
    }
}
