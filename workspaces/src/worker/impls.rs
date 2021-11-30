use std::collections::HashMap;
use std::path::Path;

use async_trait::async_trait;
use near_crypto::InMemorySigner;
use near_primitives::borsh::BorshSerialize;
use near_primitives::types::{AccountId, Balance, FunctionArgs, StoreKey};

use crate::network::{
    Account, AllowDevAccountCreation, CallExecution, CallExecutionDetails, Contract, NetworkClient,
    NetworkInfo, StatePatcher, TopLevelAccountCreator,
};
use crate::rpc::client::Client;
use crate::worker::Worker;

unsafe impl<T> Send for Worker<T> where T: Send {}
unsafe impl<T> Sync for Worker<T> where T: Sync {}

impl<T> Clone for Worker<T> {
    fn clone(&self) -> Self {
        Self {
            workspace: self.workspace.clone(),
        }
    }
}

impl<T> AllowDevAccountCreation for Worker<T> where T: AllowDevAccountCreation {}

#[async_trait]
impl<T> TopLevelAccountCreator for Worker<T>
where
    T: TopLevelAccountCreator + Send + Sync,
{
    async fn create_tla(
        &self,
        id: AccountId,
        signer: InMemorySigner,
    ) -> anyhow::Result<CallExecution<Account>> {
        self.workspace.create_tla(id, signer).await
    }

    async fn create_tla_and_deploy<P: AsRef<Path> + Send + Sync>(
        &self,
        id: AccountId,
        signer: InMemorySigner,
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

#[async_trait]
impl<T> StatePatcher for Worker<T>
where
    T: StatePatcher + Send + Sync,
{
    async fn patch_state<U>(
        &self,
        contract_id: AccountId,
        key: String,
        value: &U,
    ) -> anyhow::Result<()>
    where
        U: BorshSerialize + Send + Sync,
    {
        self.workspace.patch_state(contract_id, key, value).await
    }
}

impl<T> Worker<T>
where
    T: NetworkClient,
{
    pub(crate) fn client(&self) -> &Client {
        self.workspace.client()
    }

    pub async fn call(
        &self,
        contract: &Contract,
        method: String,
        args: Vec<u8>,
        deposit: Option<Balance>,
    ) -> anyhow::Result<CallExecutionDetails> {
        self.client()
            .call(
                contract.signer(),
                contract.id().clone(),
                method,
                args,
                None,
                deposit,
            )
            .await
            .map(Into::into)
    }

    pub async fn view(
        &self,
        contract_id: AccountId,
        method_name: String,
        args: FunctionArgs,
    ) -> anyhow::Result<serde_json::Value> {
        self.client().view(contract_id, method_name, args).await
    }

    pub async fn view_state(
        &self,
        contract_id: AccountId,
        prefix: Option<StoreKey>,
    ) -> anyhow::Result<HashMap<String, Vec<u8>>> {
        self.client().view_state(contract_id, prefix).await
    }

    pub async fn transfer_near(
        &self,
        signer: &InMemorySigner,
        receiver_id: AccountId,
        amount_yocto: Balance,
    ) -> anyhow::Result<CallExecutionDetails> {
        self.client()
            .transfer_near(signer, receiver_id, amount_yocto)
            .await
            .map(Into::into)
    }

    pub async fn delete_account(
        &self,
        account_id: AccountId,
        signer: &InMemorySigner,
        beneficiary_id: AccountId,
    ) -> anyhow::Result<CallExecutionDetails> {
        self.client()
            .delete_account(signer, account_id, beneficiary_id)
            .await
            .map(Into::into)
    }
}
