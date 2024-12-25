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

/// Trait provides the ability to create a sponsored subaccount of network's root account.
///
/// Network's root account is identified by value returned by [`Self::root_account_id`] method.
/// The `subaccount_prefix` is a prefix for the subaccount ID.
/// For example, if this parameter is `"subaccount"` then
/// the full ID for testnet will be `"subaccount.testnet"`.
///
/// It is expected that the `subaccount_prefix` does not contain a `.`.
#[async_trait]
pub trait RootAccountSubaccountCreator {
    /// for sandbox value of [`Worker::<Sandbox>::root_account`]
    /// and for testnet value of [`Worker::<Testnet>::root_account_id`]
    /// are consistent with id that this method returns
    fn root_account_id(&self) -> Result<AccountId>;

    async fn create_root_account_subaccount(
        &self,
        worker: Worker<dyn Network>,
        subaccount_prefix: AccountId,
        sk: SecretKey,
    ) -> Result<Execution<Account>>;

    async fn create_root_account_subaccount_and_deploy(
        &self,
        worker: Worker<dyn Network>,
        subaccount_prefix: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>>;
}

/// tla - stands for "top level account"
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

impl<T> Worker<T> {
    pub fn generate_dev_account_credentials(&self) -> (AccountId, SecretKey) {
        let id = crate::rpc::tool::random_account_id();
        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
        (id, sk)
    }
}

impl<T> Worker<T>
where
    T: Network + TopLevelAccountCreator + 'static,
{
    /// Creates account `id` as top level account
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

    /// Creates account `id` as top level account and deploys wasm code to it
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

    /// Creates a top level developement account.
    /// On sandbox network it has a balance of 100 Near.
    /// If you need more Near for your tests in sandbox consider using [`Worker::<Sandbox>::root_account`] method.
    pub async fn dev_create_tla(&self) -> Result<Account> {
        let (id, sk) = self.generate_dev_account_credentials();
        let account = self.create_tla(id, sk).await?;
        Ok(account.into_result()?)
    }

    /// Creates a top level developement account and deploys wasm code to it.
    pub async fn dev_deploy_tla(&self, wasm: &[u8]) -> Result<Contract> {
        let (id, sk) = self.generate_dev_account_credentials();
        let contract = self.create_tla_and_deploy(id, sk, wasm).await?;
        Ok(contract.into_result()?)
    }
}

impl<T> Worker<T>
where
    T: DevNetwork + 'static,
{
    pub async fn create_root_account_subaccount(
        &self,
        subaccount_prefix: AccountId,
        sk: SecretKey,
    ) -> Result<Execution<Account>> {
        if subaccount_prefix.as_str().contains('.') {
            return Err(crate::error::ErrorKind::Io
                .custom("Subaccount prefix for subaccount created cannot contain '.'"));
        }
        let res = self
            .workspace
            .create_root_account_subaccount(self.clone().coerce(), subaccount_prefix, sk)
            .await?;

        for callback in self.tx_callbacks.iter() {
            callback(res.details.total_gas_burnt)?;
        }

        Ok(res)
    }

    pub async fn create_root_account_subaccount_and_deploy(
        &self,
        subaccount_prefix: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>> {
        if subaccount_prefix.as_str().contains('.') {
            return Err(crate::error::ErrorKind::Io
                .custom("Subaccount prefix for subaccount created cannot contain '.'"));
        }
        let res = self
            .workspace
            .create_root_account_subaccount_and_deploy(
                self.clone().coerce(),
                subaccount_prefix,
                sk,
                wasm,
            )
            .await?;

        for callback in self.tx_callbacks.iter() {
            callback(res.details.total_gas_burnt)?;
        }

        Ok(res)
    }

    /// Creates a subaccount of the network's [root account](RootAccountSubaccountCreator::root_account_id) with
    /// random account ID and secret key. By default, balance is around 10 Near for testnet
    /// and 100 NEAR for sandbox.
    pub async fn dev_create_account(&self) -> Result<Account> {
        let (id, sk) = self.generate_dev_account_credentials();
        let account = self.create_root_account_subaccount(id, sk).await?;
        Ok(account.into_result()?)
    }

    /// Creates a subaccount of the network's [root account](RootAccountSubaccountCreator::root_account_id) with
    /// random account ID and secret key and deploys provided wasm code into it.
    pub async fn dev_deploy(&self, wasm: &[u8]) -> Result<Contract> {
        let (id, sk) = self.generate_dev_account_credentials();
        let contract = self
            .create_root_account_subaccount_and_deploy(id, sk, wasm)
            .await?;
        Ok(contract.into_result()?)
    }
}

/// Network trait specifies the functionality of a network type such as mainnet, testnet or any
/// other networks that are not specified in this library.
pub trait Network: NetworkInfo + NetworkClient + Send + Sync {}

impl<T> Network for T where T: NetworkInfo + NetworkClient + Send + Sync {}

/// DevNetwork is a Network that can call into [`Worker::dev_create_account`] and [`Worker::dev_deploy`] to create developer accounts.
pub trait DevNetwork: Network + RootAccountSubaccountCreator + 'static {}

impl<T> DevNetwork for T where T: Network + RootAccountSubaccountCreator + 'static {}
