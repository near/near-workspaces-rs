mod impls;
mod testnet;
mod sandbox;

use std::path::Path;

use async_trait::async_trait;

use near_crypto::PublicKey;
use near_primitives::{types::AccountId, views::FinalExecutionStatus};

use crate::rpc::client::Client;

// TODO: currently a marker trait
pub trait Server {
}

pub struct Account {}

unsafe impl std::marker::Sync for Account {}
unsafe impl std::marker::Send for Account {}

pub struct Contract {
    account: Account,
}

unsafe impl std::marker::Sync for Contract {}
unsafe impl std::marker::Send for Contract {}

pub trait NetworkClient {
    fn client(&self) -> &Client;
}

pub trait NetworkInfo {
    /// Name of the network itself
    fn name(&self) -> String;
    fn root_account_id(&self) -> AccountId;

    /// Path to the keystore directory
    fn keystore_path(&self) -> std::path::PathBuf;

    fn rpc_url(&self) -> String;
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
            FinalExecutionStatus::NotStarted => Err(anyhow::anyhow!("Transaction not started")),
            FinalExecutionStatus::Started => Err(anyhow::anyhow!("Transaction still being processed.")),
        }
    }
}

#[async_trait]
pub trait TopLevelAccountCreator {
    async fn create_tla(&self, id: AccountId, pk: PublicKey) -> anyhow::Result<CallExecution<Account>>;
    async fn create_tla_and_deploy<P: AsRef<Path> + Send + Sync>(&self, id: AccountId, pk: PublicKey, wasm: P) -> anyhow::Result<CallExecution<Account>>;
}

// NOTE: Not all networks/runtimes will have the ability to be able to do dev_deploy.
// This trait acts as segmented boundary for only specific networks such as sandbox and testnet.
pub trait AllowDevAccountCreation {}

#[async_trait]
pub trait DevAccountDeployer {
    async fn dev_create(&self) -> anyhow::Result<Account>;
    async fn dev_deploy<P: AsRef<Path> + Send + Sync>(&self, wasm: P) -> anyhow::Result<Contract>;
}

#[async_trait]
impl<T> DevAccountDeployer for T
where
    T: TopLevelAccountCreator + AllowDevAccountCreation,
    Self: Send + Sync,
{
    async fn dev_create(&self) -> anyhow::Result<Account> {
        let (account_id, signer) = crate::dev_generate();
        let account = self.create_tla(account_id.clone(), signer.public_key).await?;
        account.into()
    }

    async fn dev_deploy<P: AsRef<Path> + Send + Sync>(&self, wasm: P) -> anyhow::Result<Contract> {
        Ok(Contract { account: Account {}})
    }
}

#[async_trait]
pub trait StatePatcher {
    async fn patch_state(&self) -> anyhow::Result<()>;
}

pub trait Network: TopLevelAccountCreator + NetworkActions + NetworkInfo {}
