use std::{fs::File, io::Read, path::Path, str::FromStr};

use async_trait::async_trait;

use near_crypto::PublicKey;
use near_primitives::types::AccountId;

use crate::{NEAR_BASE, rpc::client::Client, runtime::local::SandboxServer};

use super::{Account, AllowDevAccountCreation, CallExecution, Network, NetworkActions, NetworkInfo, TopLevelAccountCreator};


pub struct Sandbox {
    server: SandboxServer,
    client: Client,
}

impl Default for Sandbox {
    fn default() -> Self {
        let mut server = SandboxServer::default();
        server.start().unwrap();

        let client = Client::new(server.rpc_addr());
        Self { server, client }
    }
}

impl AllowDevAccountCreation for Sandbox {}

#[async_trait]
impl TopLevelAccountCreator for Sandbox {
    async fn create_tla(&self, id: AccountId, pk: PublicKey) -> anyhow::Result<CallExecution<Account>> {
        let root_signer = crate::runtime::local::root_account();
        let outcome = self.client.create_account(&root_signer, id, pk, NEAR_BASE).await?;

        Ok(CallExecution {
            result: Account {},
            details: outcome.into(),
        })
    }

    async fn create_tla_and_deploy<P: AsRef<Path> + Send + Sync>(&self, id: AccountId, pk: PublicKey, wasm: P) -> anyhow::Result<CallExecution<Contract>> {
        // TODO: async_compat/async version of File
        let mut code = Vec::new();
        File::open(wasm)?.read_to_end(&mut code)?;

        let root_signer = crate::runtime::local::root_account();
        let outcome = self.client.create_account_and_deploy(&root_signer, id, pk, NEAR_BASE, code).await?;

        Ok(CallExecution {
            result: Contract { account: Account {} },
            details: outcome.into(),
        })
    }
}

impl NetworkActions for Sandbox {

}

impl NetworkInfo for Sandbox {
    fn name(&self) -> String {
        "Sandbox".into()
    }

    fn root_account_id(&self) -> AccountId {
        AccountId::from_str("test.near").unwrap()
    }

    fn keystore_path(&self) -> std::path::PathBuf {
        // std::path::PathBuf::from("")
        todo!()
    }

    fn rpc_url(&self) -> String {
        self.server.rpc_addr()
    }

    fn helper_url(&self) -> String {
        todo!()
    }
}

impl Network for Sandbox {}
