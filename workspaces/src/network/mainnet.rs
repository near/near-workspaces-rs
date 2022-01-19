use std::path::PathBuf;

use async_trait::async_trait;

use crate::network::Info;
use crate::network::{Account, CallExecution, NetworkClient, NetworkInfo, TopLevelAccountCreator};
use crate::rpc::client::Client;
use crate::types::{AccountId, SecretKey};
use crate::Contract;

const RPC_URL: &str = "https://rpc.mainnet.near.org";
const ARCHIVAL_URL: &str = "https://archival-rpc.mainnet.near.org";

pub struct Mainnet {
    client: Client,
    info: Info,
}

impl Mainnet {
    pub(crate) fn new() -> Self {
        Self {
            client: Client::new(RPC_URL.into()),
            info: Info {
                name: "mainnet".into(),
                root_id: "near".parse().unwrap(),
                keystore_path: PathBuf::from(".near-credentials/mainnet/"),
                rpc_url: RPC_URL.into(),
            },
        }
    }
}

#[async_trait]
impl TopLevelAccountCreator for Mainnet {
    async fn create_tla(
        &self,
        _id: AccountId,
        _sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account>> {
        panic!("Unsupported for now: https://github.com/near/workspaces-rs/issues/18");
    }

    async fn create_tla_and_deploy(
        &self,
        _id: AccountId,
        _sk: SecretKey,
        _wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract>> {
        panic!("Unsupported for now: https://github.com/near/workspaces-rs/issues/18");
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

pub struct MainnetArchival {
    client: Client,
    info: Info,
}

impl MainnetArchival {
    pub(crate) fn new() -> Self {
        Self {
            client: Client::new(ARCHIVAL_URL.into()),
            info: Info {
                name: "mainnet".into(),
                root_id: "near".parse().unwrap(),
                keystore_path: PathBuf::from(".near-credentials/mainnet/"),
                rpc_url: ARCHIVAL_URL.into(),
            },
        }
    }
}

#[async_trait]
impl TopLevelAccountCreator for MainnetArchival {
    async fn create_tla(
        &self,
        _id: AccountId,
        _sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account>> {
        panic!("Archival networks do not support creating accounts");
    }

    async fn create_tla_and_deploy(
        &self,
        _id: AccountId,
        _sk: SecretKey,
        _wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract>> {
        panic!("Archival networks do not support creating accounts");
    }
}

impl NetworkClient for MainnetArchival {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for MainnetArchival {
    fn info(&self) -> &Info {
        &self.info
    }
}
