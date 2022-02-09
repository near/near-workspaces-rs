use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;

use super::{
    Account, AllowDevAccountCreation, AllowStatePatching, CallExecution, Contract, NetworkClient,
    NetworkInfo, TopLevelAccountCreator,
};

use crate::network::sandbox::{HasRpcPort, DEFAULT_DEPOSIT};
use crate::network::server::SandboxServer;
use crate::network::{Info, Sandbox};
use crate::rpc::client::Client;
use crate::types::{AccountId, InMemorySigner, SecretKey};

pub struct SandboxMulti {
    server: SandboxServer,
    client: Client,
    info: Info,
}

impl SandboxMulti {
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

impl AllowStatePatching for SandboxMulti {}

impl AllowDevAccountCreation for SandboxMulti {}

#[async_trait]
impl TopLevelAccountCreator for SandboxMulti {
    async fn create_tla(
        &self,
        id: AccountId,
        sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account>> {
        let root_signer = Sandbox::root_signer(self.rpc_port());
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
        let root_signer = Sandbox::root_signer(self.rpc_port());
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

impl NetworkClient for SandboxMulti {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for SandboxMulti {
    fn info(&self) -> &Info {
        &self.info
    }
}

impl HasRpcPort for SandboxMulti {
    fn rpc_port(&self) -> u16 {
        self.server.rpc_port
    }
}
