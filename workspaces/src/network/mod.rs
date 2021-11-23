mod account;
mod sandbox;
mod testnet;

use std::path::Path;

use async_trait::async_trait;

use near_crypto::{InMemorySigner, KeyType, PublicKey, Signer};
use near_primitives::{types::AccountId, views::FinalExecutionStatus};

use crate::rpc::client::Client;

pub use crate::network::sandbox::Sandbox;
pub use crate::network::account::{Account, Contract};

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


pub trait NetworkActions {
}

// TODO: add CallExecution* Types into their own file
/// Struct to hold a type we want to return along w/ the execution result view.
/// This view has extra info about the execution, such as gas usage and whether
/// the transaction failed to be processed on the chain.
pub struct CallExecution<T> {
    pub result: T,
    pub details: crate::CallExecutionResult,
}

impl<T> CallExecution<T> {
    fn unwrap(self) -> T {
        match self.details.status {
            FinalExecutionStatus::SuccessValue(_) => self.result,
            _ => panic!("Call failed"),
        }
    }
}

impl<T> Into<anyhow::Result<T>> for CallExecution<T> {
    fn into(self) -> anyhow::Result<T> {
        match self.details.status {
            FinalExecutionStatus::SuccessValue(_) => Ok(self.result),
            FinalExecutionStatus::Failure(err) => Err(anyhow::anyhow!(err)),
            FinalExecutionStatus::NotStarted => Err(anyhow::anyhow!("Transaction not started.")),
            FinalExecutionStatus::Started => {
                Err(anyhow::anyhow!("Transaction still being processed."))
            }
        }
    }
}

#[async_trait]
pub trait TopLevelAccountCreator {
    async fn create_tla(&self, id: AccountId, pk: PublicKey) -> anyhow::Result<CallExecution<Account>>;
    async fn create_tla_and_deploy<P: AsRef<Path> + Send + Sync>(&self, id: AccountId, signer: &InMemorySigner, wasm: P) -> anyhow::Result<CallExecution<Contract>>;
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
    T: TopLevelAccountCreator + NetworkInfo + AllowDevAccountCreation,
    Self: Send + Sync,
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
        let account = self
            .create_tla(account_id.clone(), signer.public_key)
            .await?;
        account.into()
    }

    async fn dev_deploy<P: AsRef<Path> + Send + Sync>(&self, wasm: P) -> anyhow::Result<Contract> {
        let (account_id, signer) = self.dev_generate();
        let contract = self
            .create_tla_and_deploy(account_id.clone(), &signer, wasm)
            .await?;
        contract.into()
    }
}

#[async_trait]
pub trait StatePatcher {
    async fn patch_state(&self) -> anyhow::Result<()>;
}

pub trait Network: TopLevelAccountCreator + NetworkActions + NetworkInfo + Send + Sync {}

impl<T> Network for T where T: TopLevelAccountCreator + NetworkActions + NetworkInfo + Send + Sync {}

/// DevNetwork is a Network that can call into `dev_create` and `dev_deploy` to create developer accounts.
pub trait DevNetwork: AllowDevAccountCreation + Network {}

// Implemented by default if we have `AllowDevAccountCreation`
impl<T> DevNetwork for T where T: AllowDevAccountCreation + Network {}
