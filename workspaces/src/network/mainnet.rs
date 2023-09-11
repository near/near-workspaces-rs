use crate::network::{Info, NetworkClient, NetworkInfo};
use crate::result::{Execution, ExecutionFinalResult, Result};
use crate::rpc::client::Client;
use crate::types::{Balance, SecretKey};
use crate::{Account, Contract, InMemorySigner, Network, Worker};
use std::path::PathBuf;

use async_trait::async_trait;

use super::builder::{FromNetworkBuilder, NetworkBuilder};
use super::TopLevelAccountCreator;
use near_account_id::AccountId;

/// URL to the mainnet RPC node provided by near.org.
pub const RPC_URL: &str = "https://rpc.mainnet.near.org";

/// URL to the mainnet archival RPC node provided by near.org.
pub const ARCHIVAL_URL: &str = "https://archival-rpc.mainnet.near.org";

/// Deposit balance for creating a new account on mainnet.
const DEFAULT_DEPOSIT: Balance = 0;

/// Mainnet related configuration for interacting with mainnet. Look at
/// [`workspaces::mainnet`] and [`workspaces::mainnet_archival`] for how to
/// spin up a [`Worker`] that can be used to interact with mainnet. Note that
/// mainnet account creation is not currently supported, and these calls into
/// creating a mainnet worker is meant for retrieving data and/or making
/// queries only.
///
/// [`workspaces::mainnet`]: crate::mainnet
/// [`workspaces::mainnet_archival`]: crate::mainnet_archival
/// [`Worker`]: crate::Worker
pub struct Mainnet {
    client: Client,
    info: Info,
}

#[async_trait::async_trait]
impl FromNetworkBuilder for Mainnet {
    async fn from_builder<'a>(build: NetworkBuilder<'a, Self>) -> Result<Self> {
        let rpc_url = build.rpc_addr.unwrap_or_else(|| RPC_URL.into());
        let client = Client::new(&rpc_url);
        client.wait_for_rpc().await?;

        Ok(Self {
            client,
            info: Info {
                name: build.name.into(),
                root_id: "near".parse().unwrap(),
                keystore_path: PathBuf::from(".near-credentials/mainnet/"),
                rpc_url,
            },
        })
    }
}

impl std::fmt::Debug for Mainnet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mainnet")
            .field("root_id", &self.info.root_id)
            .field("rpc_url", &self.info.rpc_url)
            .finish()
    }
}

impl NetworkClient for Mainnet {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Mainnet {
    fn info(&self) -> &Info {
        &self.info
    }
}

/// The methods assume the account credentials are already present in the
/// keystore. If not, you can use the [`Account::create_subaccount`] which also
/// provide better configuration options.
#[async_trait]
impl TopLevelAccountCreator for Mainnet {
    async fn create_tla(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
    ) -> Result<Execution<Account>> {
        let root_signer =
            InMemorySigner::from_file(worker.workspace.info().keystore_path.as_path())?;

        let outcome = self
            .client()
            .create_account(&root_signer, &id, sk.public_key(), DEFAULT_DEPOSIT) //
            .await?;

        let signer = InMemorySigner::from_secret_key(id, sk);
        Ok(Execution {
            result: Account::new(signer, worker),
            details: ExecutionFinalResult::from_view(outcome),
        })
    }

    async fn create_tla_and_deploy(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>> {
        let root_signer =
            InMemorySigner::from_file(worker.workspace.info().keystore_path.as_path())?;

        let outcome = self
            .client()
            .create_account_and_deploy(
                &root_signer,
                &id,
                sk.public_key(),
                DEFAULT_DEPOSIT,
                wasm.into(),
            )
            .await?;

        let signer = InMemorySigner::from_secret_key(id, sk);
        Ok(Execution {
            result: Contract::new(signer, worker),
            details: ExecutionFinalResult::from_view(outcome),
        })
    }
}
