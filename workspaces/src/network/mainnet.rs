use crate::network::{Info, NetworkClient, NetworkInfo};
use crate::result::Result;
use crate::rpc::client::Client;
use std::path::PathBuf;

const RPC_URL: &str = "https://rpc.mainnet.near.org";
const ARCHIVAL_URL: &str = "https://archival-rpc.mainnet.near.org";

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

impl Mainnet {
    pub(crate) async fn new() -> Result<Self> {
        let client = Client::new(RPC_URL.into());
        client.wait_for_rpc().await?;

        Ok(Self {
            client,
            info: Info {
                name: "mainnet".into(),
                root_id: "near".parse().unwrap(),
                keystore_path: PathBuf::from(".near-credentials/mainnet/"),
                rpc_url: RPC_URL.into(),
            },
        })
    }

    pub(crate) async fn archival() -> Result<Self> {
        let client = Client::new(ARCHIVAL_URL.into());
        client.wait_for_rpc().await?;

        Ok(Self {
            client,
            info: Info {
                name: "mainnet-archival".into(),
                root_id: "near".parse().unwrap(),
                keystore_path: PathBuf::from(".near-credentials/mainnet/"),
                rpc_url: ARCHIVAL_URL.into(),
            },
        })
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
