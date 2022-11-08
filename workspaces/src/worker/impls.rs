use crate::error::SandboxErrorCode;
use crate::network::{AllowDevAccountCreation, NetworkClient, NetworkInfo};
use crate::network::{Info, Sandbox};
use crate::result::{ExecutionFinalResult, Result, ViewResultDetails};
use crate::rpc::client::{Client, DEFAULT_CALL_DEPOSIT, DEFAULT_CALL_FN_GAS};
use crate::rpc::patch::ImportContractTransaction;
use crate::types::{AccountId, Gas, InMemorySigner};
use crate::worker::Worker;
use crate::{Account, Block, Contract};
use crate::{AccountDetails, Network};

use near_jsonrpc_client::methods::sandbox_fast_forward::RpcSandboxFastForwardRequest;
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::state_record::StateRecord;
use near_primitives::types::Balance;

use std::collections::HashMap;

impl<T: ?Sized> Clone for Worker<T> {
    fn clone(&self) -> Self {
        Self {
            workspace: self.workspace.clone(),
        }
    }
}

impl<T> AllowDevAccountCreation for Worker<T> where T: AllowDevAccountCreation {}

impl<T> NetworkInfo for Worker<T>
where
    T: NetworkInfo,
{
    fn info(&self) -> &Info {
        self.workspace.info()
    }
}

impl<T: ?Sized> Worker<T>
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
    ) -> Result<ExecutionFinalResult> {
        let outcome = self
            .client()
            .call(
                contract.signer(),
                contract.id(),
                function.into(),
                args,
                gas.unwrap_or(DEFAULT_CALL_FN_GAS),
                deposit.unwrap_or(DEFAULT_CALL_DEPOSIT),
            )
            .await?;

        Ok(ExecutionFinalResult::from_view(outcome))
    }

    /// Call into a contract's view function.
    pub async fn view(
        &self,
        contract_id: &AccountId,
        function: &str,
        args: Vec<u8>,
    ) -> Result<ViewResultDetails> {
        self.client()
            .view(contract_id.clone(), function.into(), args)
            .await
    }

    /// View the WASM code bytes of a contract on the network.
    pub async fn view_code(&self, contract_id: &AccountId) -> Result<Vec<u8>> {
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
    ) -> Result<HashMap<Vec<u8>, Vec<u8>>> {
        self.client()
            .view_state(contract_id.clone(), prefix, None)
            .await
    }

    /// View the latest block from the network
    pub async fn view_latest_block(&self) -> Result<Block> {
        self.client().view_block(None).await.map(Into::into)
    }

    /// Transfer tokens from one account to another. The signer is the account
    /// that will be used to to send from.
    pub async fn transfer_near(
        &self,
        signer: &InMemorySigner,
        receiver_id: &AccountId,
        amount_yocto: Balance,
    ) -> Result<ExecutionFinalResult> {
        self.client()
            .transfer_near(signer, receiver_id, amount_yocto)
            .await
            .map(ExecutionFinalResult::from_view)
            .map_err(crate::error::Error::from)
    }

    /// Deletes an account from the network. The beneficiary will receive the balance
    /// of the account deleted.
    pub async fn delete_account(
        &self,
        account_id: &AccountId,
        signer: &InMemorySigner,
        beneficiary_id: &AccountId,
    ) -> Result<ExecutionFinalResult> {
        self.client()
            .delete_account(signer, account_id, beneficiary_id)
            .await
            .map(ExecutionFinalResult::from_view)
            .map_err(crate::error::Error::from)
    }

    /// View account details of a specific account on the network.
    pub async fn view_account(&self, account_id: &AccountId) -> Result<AccountDetails> {
        self.client()
            .view_account(account_id.clone(), None)
            .await
            .map(Into::into)
    }
}

impl Worker<Sandbox> {
    pub fn root_account(&self) -> Result<Account> {
        let signer = self.workspace.root_signer()?;
        Ok(Account::new(signer, self.clone().coerce()))
    }

    /// Import a contract from the the given network, and return us a [`ImportContractTransaction`]
    /// which allows to specify further details, such as being able to import contract data and
    /// how far back in time we wanna grab the contract.
    pub fn import_contract<'a>(
        &self,
        id: &AccountId,
        worker: &'a Worker<impl Network>,
    ) -> ImportContractTransaction<'a> {
        ImportContractTransaction::new(id.to_owned(), worker.client(), self.clone().coerce())
    }

    /// Patch state into the sandbox network, given a key and value. This will allow us to set
    /// state that we have acquired in some manner. This allows us to test random cases that
    /// are hard to come up naturally as state evolves.
    pub async fn patch_state(
        &self,
        contract_id: &AccountId,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        let state = StateRecord::Data {
            account_id: contract_id.to_owned(),
            data_key: key.to_vec(),
            value: value.to_vec(),
        };
        let records = vec![state];

        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|e| SandboxErrorCode::PatchStateFailure.custom(e))?;

        Ok(())
    }

    /// Fast forward to a point in the future. The delta block height is supplied to tell the
    /// network to advanced a certain amount of blocks. This comes with the advantage only having
    /// to wait a fraction of the time it takes to produce the same number of blocks.
    ///
    /// Estimate as to how long it takes: if our delta_height crosses `X` epochs, then it would
    /// roughly take `X * 5` seconds for the fast forward request to be processed.
    pub async fn fast_forward(&self, delta_height: u64) -> Result<()> {
        self.client()
            // TODO: replace this with the `query` variant when RpcSandboxFastForwardRequest impls Debug
            .query_nolog(&RpcSandboxFastForwardRequest { delta_height })
            .await
            .map_err(|e| SandboxErrorCode::FastForwardFailure.custom(e))?;

        Ok(())
    }

    /// The port being used by RPC
    pub fn rpc_port(&self) -> u16 {
        self.workspace.rpc_port()
    }
}
