use crate::network::{Info, NetworkClient, NetworkInfo};
use crate::rpc::client::Client;
use std::path::PathBuf;

const RPC_URL: &str = "https://rpc.mainnet.near.org";
const ARCHIVAL_URL: &str = "https://archival-rpc.mainnet.near.org";

pub struct Mainnet {
    client: Client,
    info: Info,
}

impl Mainnet {
    pub(crate) async fn new() -> anyhow::Result<Self> {
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

    pub(crate) async fn archival() -> anyhow::Result<Self> {
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
    type Network = Self;
    
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Mainnet {
    fn info(&self) -> &Info {
        &self.info
    }
}
