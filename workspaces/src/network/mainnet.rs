use crate::network::Info;
use crate::network::{NetworkClient, NetworkInfo};
use crate::rpc::client::Client;
use std::path::PathBuf;

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

    pub(crate) fn archival() -> Self {
        Self {
            client: Client::new(ARCHIVAL_URL.into()),
            info: Info {
                name: "mainnet-archival".into(),
                root_id: "near".parse().unwrap(),
                keystore_path: PathBuf::from(".near-credentials/mainnet/"),
                rpc_url: ARCHIVAL_URL.into(),
            },
        }
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
