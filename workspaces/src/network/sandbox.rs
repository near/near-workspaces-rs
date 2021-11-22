use std::path::Path;

use async_trait::async_trait;

use near_crypto::PublicKey;
use near_primitives::types::AccountId;

use crate::{NEAR_BASE, rpc::client::Client, runtime::local::SandboxServer};

use super::{Account, AllowDevAccountCreation, CallExecution, TopLevelAccountCreator};


struct Sandbox {
    server: SandboxServer,
    client: Client,
}

impl Sandbox {
}

impl AllowDevAccountCreation for Sandbox {}

#[async_trait]
impl TopLevelAccountCreator for Sandbox {
    async fn create_tla(&self, id: AccountId, pk: PublicKey) -> anyhow::Result<CallExecution<Account>> {
        todo!()
    }

    async fn create_tla_and_deploy<P: AsRef<Path> + Send + Sync>(&self, id: AccountId, pk: PublicKey, wasm: P) -> anyhow::Result<CallExecution<Account>> {
        todo!()
    }
}
