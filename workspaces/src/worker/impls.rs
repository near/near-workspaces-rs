use std::collections::HashMap;

use async_trait::async_trait;
use near_primitives::types::{Balance, StoreKey};

use crate::network::{
    Account, AllowDevAccountCreation, CallExecution, CallExecutionDetails, Contract, NetworkClient,
    NetworkInfo, StatePatcher, TopLevelAccountCreator, ViewResultDetails,
};
use crate::network::{Info, Sandbox};
use crate::rpc::client::Client;
use crate::rpc::patch::ImportContractBuilder;
use crate::types::{AccountId, InMemorySigner, SecretKey};
use crate::worker::Worker;
use crate::Network;

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
        sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account>> {
        self.workspace.create_tla(id, sk).await
    }

    async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: Vec<u8>,
    ) -> anyhow::Result<CallExecution<Contract>> {
        self.workspace.create_tla_and_deploy(id, sk, wasm).await
    }
}

impl<T> NetworkInfo for Worker<T>
where
    T: NetworkInfo,
{
    fn info(&self) -> &Info {
        self.workspace.info()
    }
}

#[async_trait]
impl<T> StatePatcher for Worker<T>
where
    T: StatePatcher + Send + Sync,
{
    async fn patch_state(
        &self,
        contract_id: AccountId,
        key: String,
        value: Vec<u8>,
    ) -> anyhow::Result<()> {
        self.workspace.patch_state(contract_id, key, value).await
    }

    fn import_contract<'a, 'b>(
        &'b self,
        id: AccountId,
        worker: &'a Worker<impl Network>,
    ) -> ImportContractBuilder<'a, 'b> {
        self.workspace.import_contract(id, worker)
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
        args: Vec<u8>,
    ) -> anyhow::Result<ViewResultDetails> {
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

impl Worker<Sandbox> {
    pub fn root_account(&self) -> Account {
        let account_id = self.info().root_id.clone();
        let signer = self.workspace.root_signer();
        Account::new(account_id, signer)
    }
}
