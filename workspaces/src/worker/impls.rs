use crate::network::{AllowDevAccountCreation, NetworkClient, NetworkInfo};
use crate::network::{Info, Sandbox};
use crate::operations::{CallTransaction, Function};
use crate::result::{ExecutionFinalResult, Result};
use crate::rpc::client::Client;
use crate::rpc::patch::{ImportContractTransaction, PatchTransaction};
use crate::rpc::query::{
    GasPrice, Query, QueryChunk, ViewAccessKey, ViewAccessKeyList, ViewAccount, ViewBlock,
    ViewCode, ViewFunction, ViewState,
};
use crate::types::{AccountId, InMemorySigner, PublicKey};
use crate::worker::Worker;
use crate::{Account, Network};

use near_primitives::types::Balance;

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

    /// Call into a contract's view function. Returns a [`Query`] which allows us
    /// to specify further details like the arguments of the view call, or at what
    /// point in the chain we want to view.
    pub fn view(&self, contract_id: &AccountId, function: &str) -> Query<'_, ViewFunction> {
        self.view_by_function(contract_id, Function::new(function))
    }

    pub(crate) fn view_by_function(
        &self,
        contract_id: &AccountId,
        function: Function,
    ) -> Query<'_, ViewFunction> {
        Query::new(
            self.client(),
            ViewFunction {
                account_id: contract_id.clone(),
                function,
            },
        )
    }

    /// View the WASM code bytes of a contract on the network.
    pub fn view_code(&self, contract_id: &AccountId) -> Query<'_, ViewCode> {
        Query::new(
            self.client(),
            ViewCode {
                account_id: contract_id.clone(),
            },
        )
    }

    /// View the state of a account/contract on the network. This will return the internal
    /// state of the account in the form of a map of key-value pairs; where STATE contains
    /// info on a contract's internal data.
    pub fn view_state(&self, contract_id: &AccountId) -> Query<'_, ViewState> {
        Query::view_state(self.client(), contract_id)
    }

    /// View the block from the network. Supply additional parameters such as [`block_height`]
    /// or [`block_hash`] to get the block.
    ///
    /// [`block_height`]: Query::block_height
    /// [`block_hash`]: Query::block_hash
    pub fn view_block(&self) -> Query<'_, ViewBlock> {
        Query::new(self.client(), ViewBlock)
    }

    /// View the chunk from the network once awaited. Supply additional parameters such as
    /// [`block_hash_and_shard`], [`block_height_and_shard`] or [`chunk_hash`] to get the
    /// chunk at a specific reference point. If none of those are supplied, the default
    /// reference point will be used, which will be the latest block_hash with a shard_id
    /// of 0.
    ///
    /// [`block_hash_and_shard`]: QueryChunk::block_hash_and_shard
    /// [`block_height_and_shard`]: QueryChunk::block_height_and_shard
    /// [`chunk_hash`]: QueryChunk::chunk_hash
    pub fn view_chunk(&self) -> QueryChunk<'_> {
        QueryChunk::new(self.client())
    }

    /// Views the [`AccessKey`] of the account specified by [`AccountId`] associated with
    /// the [`PublicKey`]
    ///
    /// [`AccessKey`]: crate::types::AccessKey
    pub fn view_access_key(&self, id: &AccountId, pk: &PublicKey) -> Query<'_, ViewAccessKey> {
        Query::new(
            self.client(),
            ViewAccessKey {
                account_id: id.clone(),
                public_key: pk.clone(),
            },
        )
    }

    /// Views all the [`AccessKey`]s of the account specified by [`AccountId`]. This will
    /// return a list of [`AccessKey`]s along with the associated [`PublicKey`].
    ///
    /// [`AccessKey`]: crate::types::AccessKey
    pub fn view_access_keys(&self, id: &AccountId) -> Query<'_, ViewAccessKeyList> {
        Query::new(
            self.client(),
            ViewAccessKeyList {
                account_id: id.clone(),
            },
        )
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
    pub fn view_account(&self, account_id: &AccountId) -> Query<'_, ViewAccount> {
        Query::new(
            self.client(),
            ViewAccount {
                account_id: account_id.clone(),
            },
        )
    }

    pub fn gas_price(&self) -> Query<'_, GasPrice> {
        Query::new(self.client(), GasPrice)
    }
}

impl<T> Worker<T>
where
    T: Network + 'static,
{
    /// Call into a contract's change function. Returns a [`CallTransaction`] object
    /// that we will make use to populate the rest of the call details. The [`signer`]
    /// will be used to sign the transaction.
    ///
    /// [`signer`]: crate::types::InMemorySigner
    pub fn call(
        &self,
        signer: &InMemorySigner,
        contract_id: &AccountId,
        function: &str,
    ) -> CallTransaction {
        CallTransaction::new(
            self.clone().coerce(),
            contract_id.to_owned(),
            signer.clone(),
            function,
        )
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
        id: &'a AccountId,
        worker: &Worker<impl Network + 'static>,
    ) -> ImportContractTransaction<'a> {
        ImportContractTransaction::new(id, worker.clone().coerce(), self.clone())
    }

    /// Start patching the state of the account specified by the [`AccountId`]. This will create
    /// a [`PatchTransaction`] that will allow us to patch access keys, code, and contract state.
    /// This is similar to functions like [`Account::batch`] where we can perform multiple actions
    /// in one transaction.
    pub fn patch(&self, account_id: &AccountId) -> PatchTransaction {
        PatchTransaction::new(self, account_id.clone())
    }

    /// Patch state into the sandbox network, given a prefix key and value. This will allow us
    /// to set contract state that we have acquired in some manner, where we are able to test
    /// random cases that are hard to come up naturally as state evolves.
    pub async fn patch_state(
        &self,
        contract_id: &AccountId,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        self.workspace.patch_state(contract_id, key, value).await
    }

    /// Fast forward to a point in the future. The delta block height is supplied to tell the
    /// network to advanced a certain amount of blocks. This comes with the advantage only having
    /// to wait a fraction of the time it takes to produce the same number of blocks.
    ///
    /// Estimate as to how long it takes: if our delta_height crosses `X` epochs, then it would
    /// roughly take `X * 5` seconds for the fast forward request to be processed.
    pub async fn fast_forward(&self, delta_height: u64) -> Result<()> {
        self.workspace.fast_forward(delta_height).await
    }

    /// The port being used by RPC
    pub fn rpc_port(&self) -> u16 {
        self.workspace.rpc_port()
    }
}
