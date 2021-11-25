mod account;
mod result;
mod sandbox;
mod server;
mod testnet;

use std::path::Path;

use async_trait::async_trait;

use near_crypto::{InMemorySigner, KeyType, Signer};
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::borsh::BorshSerialize;
use near_primitives::state_record::StateRecord;
use near_primitives::types::AccountId;

use crate::rpc::client::Client;

pub use crate::network::account::{Account, Contract};
pub use crate::network::sandbox::Sandbox;
pub use crate::network::testnet::Testnet;

pub use self::result::{CallExecution, CallExecutionDetails};

const DEV_ACCOUNT_SEED: &str = "testificate";

pub trait NetworkClient {
    fn client(&self) -> &Client;
}

pub trait NetworkInfo {
    /// Name of the network itself
    fn name(&self) -> String;

    /// Root Account ID of the network. Mainnet has `near`, testnet has `testnet`.
    fn root_account_id(&self) -> AccountId;

    /// Path to the keystore directory
    fn keystore_path(&self) -> std::path::PathBuf;

    // TODO: change return type to Url instead of String
    /// Rpc endpoint to point our client to
    fn rpc_url(&self) -> String;

    // TODO: not everything has a helper url. maybe make this optional or remove it into a seprate trait
    /// The helper URL to create top level account out of for certain networks.
    fn helper_url(&self) -> String;
}

pub trait NetworkActions {}

#[async_trait]
pub trait TopLevelAccountCreator {
    async fn create_tla(
        &self,
        id: AccountId,
        signer: InMemorySigner,
    ) -> anyhow::Result<CallExecution<Account>>;

    async fn create_tla_and_deploy<P: AsRef<Path> + Send + Sync>(
        &self,
        id: AccountId,
        signer: InMemorySigner,
        wasm: P,
    ) -> anyhow::Result<CallExecution<Contract>>;
}

// NOTE: Not all networks/runtimes will have the ability to be able to do dev_deploy.
// This trait acts as segmented boundary for only specific networks such as sandbox and testnet.
pub trait AllowDevAccountCreation {}

#[async_trait]
pub trait DevAccountDeployer {
    fn dev_generate(&self) -> (AccountId, InMemorySigner);
    async fn dev_create(&self) -> anyhow::Result<Account>;
    async fn dev_deploy<P: AsRef<Path> + Send + Sync>(&self, wasm: P) -> anyhow::Result<Contract>;
}

#[async_trait]
impl<T> DevAccountDeployer for T
where
    T: TopLevelAccountCreator + NetworkInfo + AllowDevAccountCreation + Send + Sync,
{
    fn dev_generate(&self) -> (AccountId, InMemorySigner) {
        let account_id = crate::rpc::tool::random_account_id();
        let signer =
            InMemorySigner::from_seed(account_id.clone(), KeyType::ED25519, DEV_ACCOUNT_SEED);

        let mut savepath = self.keystore_path();

        // TODO: potentially make this into the async version:
        std::fs::create_dir_all(savepath.clone()).unwrap();

        savepath = savepath.join(account_id.to_string());
        savepath.set_extension("json");
        signer.write_to_file(&savepath);

        (account_id, signer)
    }

    async fn dev_create(&self) -> anyhow::Result<Account> {
        let (account_id, signer) = self.dev_generate();
        let account = self.create_tla(account_id.clone(), signer).await?;
        account.into()
    }

    async fn dev_deploy<P: AsRef<Path> + Send + Sync>(&self, wasm: P) -> anyhow::Result<Contract> {
        let (account_id, signer) = self.dev_generate();
        let contract = self
            .create_tla_and_deploy(account_id.clone(), signer, wasm)
            .await?;
        contract.into()
    }
}

pub trait AllowStatePatching {}

#[async_trait]
pub trait StatePatcher {
    async fn patch_state<U>(
        &self,
        contract_id: AccountId,
        key: String,
        value: &U,
    ) -> anyhow::Result<()>
    where
        U: BorshSerialize + Send + Sync;
}

#[async_trait]
impl<T> StatePatcher for T
where
    T: AllowStatePatching + NetworkClient + Send + Sync,
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
        let value = U::try_to_vec(value).unwrap();
        let state = StateRecord::Data {
            account_id: contract_id,
            data_key: key.into(),
            value,
        };
        let records = vec![state];

        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        Ok(())
    }
}

pub trait Network:
    TopLevelAccountCreator + NetworkActions + NetworkInfo + NetworkClient + Send + Sync
{
}

impl<T> Network for T where
    T: TopLevelAccountCreator + NetworkActions + NetworkInfo + NetworkClient + Send + Sync
{
}

/// DevNetwork is a Network that can call into `dev_create` and `dev_deploy` to create developer accounts.
pub trait DevNetwork: AllowDevAccountCreation + Network {}

// Implemented by default if we have `AllowDevAccountCreation`
impl<T> DevNetwork for T where T: AllowDevAccountCreation + Network {}
