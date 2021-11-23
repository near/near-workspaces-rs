use std::path::Path;

use async_trait::async_trait;
use near_crypto::{InMemorySigner, PublicKey};
use near_primitives::types::AccountId;

use crate::network::{
    Account, AllowDevAccountCreation, CallExecution, Contract, NetworkInfo, TopLevelAccountCreator,
};

use super::Worker;

unsafe impl<T> Send for Worker<T> where T: Send {}
unsafe impl<T> Sync for Worker<T> where T: Sync {}

impl<T> AllowDevAccountCreation for Worker<T> where T: AllowDevAccountCreation {}

#[async_trait]
impl<T> TopLevelAccountCreator for Worker<T>
where
    T: TopLevelAccountCreator + Send + Sync,
{
    async fn create_tla(
        &self,
        id: AccountId,
        pk: PublicKey,
    ) -> anyhow::Result<CallExecution<Account>> {
        self.workspace.create_tla(id, pk).await
    }

    async fn create_tla_and_deploy<P: AsRef<Path> + Send + Sync>(
        &self,
        id: AccountId,
        signer: &InMemorySigner,
        wasm: P,
    ) -> anyhow::Result<CallExecution<Contract>> {
        self.workspace.create_tla_and_deploy(id, signer, wasm).await
    }
}

impl<T> NetworkInfo for Worker<T>
where
    T: NetworkInfo,
{
    fn name(&self) -> String {
        self.workspace.name()
    }

    fn root_account_id(&self) -> AccountId {
        self.workspace.root_account_id()
    }

    fn keystore_path(&self) -> std::path::PathBuf {
        self.workspace.keystore_path()
    }

    fn rpc_url(&self) -> String {
        self.workspace.rpc_url()
    }

    fn helper_url(&self) -> String {
        self.workspace.helper_url()
    }
}
