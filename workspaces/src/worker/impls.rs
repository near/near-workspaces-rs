use crate::network::{
    AllowDevAccountCreation, NetworkClient, NetworkInfo, PatchAccessKeyTransaction,
    TopLevelAccountCreator,
};
use crate::network::{Info, Sandbox, PatchStateAccountTransaction, PatchStateTransaction};
use crate::result::{CallExecution, CallExecutionDetails, ViewResultDetails};
use crate::rpc::client::{Client, DEFAULT_CALL_DEPOSIT, DEFAULT_CALL_FN_GAS};
use crate::rpc::patch::ImportContractTransaction;
use crate::types::{AccountId, Gas, InMemorySigner, PublicKey, SecretKey};
use crate::worker::Worker;
use crate::{Account, Block, Contract};
use crate::{AccountDetails, Network};
use async_trait::async_trait;
use near_primitives::types::Balance;
use std::collections::HashMap;

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
        wasm: &[u8],
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

impl<T> Worker<T>
where
    T: NetworkClient,
{
    pub(crate) fn client(&self) -> &Client {
        self.workspace.client()
    }

    /// Call into a contract's change function.
    pub async fn call(
        &self,
        contract: &Contract,
        function: &str,
        args: Vec<u8>,
        gas: Option<Gas>,
        deposit: Option<Balance>,
    ) -> anyhow::Result<CallExecutionDetails> {
        self.client()
            .call(
                contract.signer(),
                contract.id(),
                function.into(),
                args,
                gas.unwrap_or(DEFAULT_CALL_FN_GAS),
                deposit.unwrap_or(DEFAULT_CALL_DEPOSIT),
            )
            .await
            .and_then(CallExecutionDetails::from_outcome)
    }

    /// Call into a contract's view function.
    pub async fn view(
        &self,
        contract_id: &AccountId,
        function: &str,
        args: Vec<u8>,
    ) -> anyhow::Result<ViewResultDetails> {
        self.client()
            .view(contract_id.clone(), function.into(), args)
            .await
    }

    /// View the WASM code bytes of a contract on the network.
    pub async fn view_code(&self, contract_id: &AccountId) -> anyhow::Result<Vec<u8>> {
        let code_view = self.client().view_code(contract_id.clone(), None).await?;
        Ok(code_view.code)
    }

    /// View the state of a account/contract on the network. This will return the internal
    /// state of the account in the form of a map of key-value pairs; where STATE contains
    /// info on a contract's internal data.
    pub async fn view_state(
        &self,
        contract_id: &AccountId,
        prefix: Option<&[u8]>,
    ) -> anyhow::Result<HashMap<Vec<u8>, Vec<u8>>> {
        self.client()
            .view_state(contract_id.clone(), prefix, None)
            .await
    }

    /// View the latest block from the network
    pub async fn view_latest_block(&self) -> anyhow::Result<Block> {
        self.client().view_block(None).await.map(Into::into)
    }

    /// Transfer tokens from one account to another. The signer is the account
    /// that will be used to to send from.
    pub async fn transfer_near(
        &self,
        signer: &InMemorySigner,
        receiver_id: &AccountId,
        amount_yocto: Balance,
    ) -> anyhow::Result<CallExecutionDetails> {
        self.client()
            .transfer_near(signer, receiver_id, amount_yocto)
            .await
            .and_then(CallExecutionDetails::from_outcome)
    }

    /// Deletes an account from the network. The beneficiary will receive the balance
    /// of the account deleted.
    pub async fn delete_account(
        &self,
        account_id: &AccountId,
        signer: &InMemorySigner,
        beneficiary_id: &AccountId,
    ) -> anyhow::Result<CallExecutionDetails> {
        self.client()
            .delete_account(signer, account_id, beneficiary_id)
            .await
            .and_then(CallExecutionDetails::from_outcome)
    }

    /// View account details of a specific account on the network.
    pub async fn view_account(&self, account_id: &AccountId) -> anyhow::Result<AccountDetails> {
        self.client()
            .view_account(account_id.clone(), None)
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

    /// Import a contract from the the given network, and return us a [`ImportContractTransaction`]
    /// which allows to specify further details, such as being able to import contract data and
    /// how far back in time we wanna grab the contract.
    pub fn import_contract<'a, 'b>(
        &'b self,
        id: &AccountId,
        worker: &'a Worker<impl Network>,
    ) -> ImportContractTransaction<'a, 'b> {
        self.workspace.import_contract(id, worker)
    }

    /// Patch state into the sandbox network, using builder pattern. This will allow us to set
    /// state that we have acquired in some manner. This allows us to test random cases that
    /// are hard to come up naturally as state evolves.
    pub fn patch_state_builder(&self, account_id: &AccountId) -> PatchStateTransaction {
        self.workspace.patch_state(account_id.clone())
    }

    /// Patch account state using builder pattern
    pub fn patch_account(&self, account_id: &AccountId) -> PatchStateAccountTransaction {
        self.workspace.patch_account(account_id.clone())
    }

    /// Patch state into the sandbox network, given a key and value. This will allow us to set
    /// state that we have acquired in some manner. This allows us to test random cases that
    /// are hard to come up naturally as state evolves.
    pub async fn patch_state(
        &self,
        contract_id: &AccountId,
        key: &[u8],
        value: &[u8],
    ) -> anyhow::Result<()> {
        self.workspace
            .patch_state(contract_id.clone())
            .data(key, value)
            .transact()
            .await?;
        Ok(())
    }

    /// Patch state into the sandbox network. Same as `patch_state` but accepts a sequence of key value pairs
    pub async fn patch_state_multiple<'s>(
        &'s self,
        account_id: &AccountId,
        kvs: impl IntoIterator<Item = (&'s [u8], &'s [u8])>,
    ) -> anyhow::Result<()> {
        self.workspace
            .patch_state(account_id.clone())
            .data_multiple(kvs)
            .transact()
            .await?;
        Ok(())
    }

    pub fn patch_access_key(
        &self,
        account_id: &AccountId,
        public_key: &PublicKey,
    ) -> PatchAccessKeyTransaction {
        self.workspace
            .patch_access_key(account_id.clone(), public_key.clone())
    }

    /// Fast forward to a point in the future. The delta block height is supplied to tell the
    /// network to advanced a certain amount of blocks. This comes with the advantage only having
    /// to wait a fraction of the time it takes to produce the same number of blocks.
    ///
    /// Estimate as to how long it takes: if our delta_height crosses `X` epochs, then it would
    /// roughly take `X * 5` seconds for the fast forward request to be processed.
    pub async fn fast_forward(&self, delta_height: u64) -> anyhow::Result<()> {
        self.workspace.fast_forward(delta_height).await
    }
}
