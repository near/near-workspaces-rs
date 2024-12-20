use crate::network::Info;
use crate::result::{Execution, Result};
use crate::rpc::client::Client;
use crate::types::{AccountId, KeyType, SecretKey};
use crate::{Account, Contract, Worker};
use async_trait::async_trait;

pub(crate) const DEV_ACCOUNT_SEED: &str = "testificate";

pub trait NetworkClient {
    fn client(&self) -> &Client;
}

pub trait NetworkInfo {
    fn info(&self) -> &Info;
}

/// Trait provides the ability to create a sponsored account.
///
/// A sponsored account is a subaccount of the network's root account.
/// The `subaccount_prefix` is a prefix for the subaccount ID.
/// For example, if this parameter is `"subaccount"` then
/// the full ID for testnet will be `"subaccount.testnet"`.
///
/// It is expected that the `subaccount_prefix` does not contain a `.`.
#[async_trait]
pub trait SponsoredAccountCreator {
    async fn create_sponsored_account(
        &self,
        worker: Worker<dyn Network>,
        subaccount_prefix: AccountId,
        sk: SecretKey,
    ) -> Result<Execution<Account>>;

    async fn create_sponsored_account_and_deploy(
        &self,
        worker: Worker<dyn Network>,
        subaccount_prefix: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>>;
}

#[async_trait]
pub trait TopLevelAccountCreator {
    async fn create_tla(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
    ) -> Result<Execution<Account>>;

    async fn create_tla_and_deploy(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>>;
}

impl<T> Worker<T>
where
    T: Network + TopLevelAccountCreator + 'static,
{
    pub async fn create_tla(&self, id: AccountId, sk: SecretKey) -> Result<Execution<Account>> {
        let res = self
            .workspace
            .create_tla(self.clone().coerce(), id, sk)
            .await?;

        for callback in self.tx_callbacks.iter() {
            callback(res.details.total_gas_burnt)?;
        }

        Ok(res)
    }

    pub async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>> {
        let res = self
            .workspace
            .create_tla_and_deploy(self.clone().coerce(), id, sk, wasm)
            .await?;

        for callback in self.tx_callbacks.iter() {
            callback(res.details.total_gas_burnt)?;
        }

        Ok(res)
    }

    pub async fn generate_tla_credentials(&self) -> (AccountId, SecretKey) {
        let id = crate::rpc::tool::random_account_id();
        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
        (id, sk)
    }

    /// Creates a top level developement account.
    /// On sandbox network it has a balance of 100 Near.
    /// If you need more Near for your tests in sandbox consider using `root_account()` method:
    ///
    /// # Examples
    /// ```
    /// use near_workspaces::{result::Result, Account, network::Sandbox, Worker};
    /// fn get_account_with_lots_of_near(worker: &Worker<Sandbox>) -> Result<Account> {
    ///     worker.root_account()
    /// }
    /// ```
    ///
    pub async fn dev_create_tla(&self) -> Result<Account> {
        let (id, sk) = self.generate_tla_credentials().await;
        let account = self.create_tla(id.clone(), sk).await?;
        Ok(account.into_result()?)
    }

    pub async fn dev_deploy_tla(&self, wasm: &[u8]) -> Result<Contract> {
        let (id, sk) = self.generate_tla_credentials().await;
        let contract = self.create_tla_and_deploy(id.clone(), sk, wasm).await?;
        Ok(contract.into_result()?)
    }
}

impl<T> Worker<T>
where
    T: DevNetwork + 'static,
{
    pub async fn create_sponsored_account(
        &self,
        subaccount_prefix: AccountId,
        sk: SecretKey,
    ) -> Result<Execution<Account>> {
        if subaccount_prefix.as_str().contains('.') {
            return Err(crate::error::ErrorKind::Io
                .custom("Subaccount prefix for sponsored account cannot contain '.'"));
        }
        let res = self
            .workspace
            .create_sponsored_account(self.clone().coerce(), subaccount_prefix, sk)
            .await?;

        for callback in self.tx_callbacks.iter() {
            callback(res.details.total_gas_burnt)?;
        }

        Ok(res)
    }

    pub async fn create_sponsored_account_and_deploy(
        &self,
        subaccount_prefix: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>> {
        if subaccount_prefix.as_str().contains('.') {
            return Err(crate::error::ErrorKind::Io
                .custom("Subaccount prefix for sponsored account cannot contain '.'"));
        }
        let res = self
            .workspace
            .create_sponsored_account_and_deploy(self.clone().coerce(), subaccount_prefix, sk, wasm)
            .await?;

        for callback in self.tx_callbacks.iter() {
            callback(res.details.total_gas_burnt)?;
        }

        Ok(res)
    }

    pub async fn generate_sponsored_credentials(&self) -> (AccountId, SecretKey) {
        let id = crate::rpc::tool::random_account_id();
        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
        (id, sk)
    }

    /// Creates a sub-account of the network root account with
    /// random account ID and secret key. By default, balance is around 10 Near.
    pub async fn dev_create_account(&self) -> Result<Account> {
        let (id, sk) = self.generate_sponsored_credentials().await;
        let account = self.create_sponsored_account(id.clone(), sk).await?;
        Ok(account.into_result()?)
    }

    pub async fn dev_deploy(&self, wasm: &[u8]) -> Result<Contract> {
        let (id, sk) = self.generate_sponsored_credentials().await;
        let contract = self
            .create_sponsored_account_and_deploy(id.clone(), sk, wasm)
            .await?;
        Ok(contract.into_result()?)
    }
}

/// Network trait specifies the functionality of a network type such as mainnet, testnet or any
/// other networks that are not specified in this library.
pub trait Network: NetworkInfo + NetworkClient + Send + Sync {}

impl<T> Network for T where T: NetworkInfo + NetworkClient + Send + Sync {}

/// DevNetwork is a Network that can call into `dev_create` and `dev_deploy` to create developer accounts.
pub trait DevNetwork: Network + SponsoredAccountCreator + 'static {}

impl<T> DevNetwork for T where T: Network + SponsoredAccountCreator + 'static {}
