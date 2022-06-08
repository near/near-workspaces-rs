use crate::network::{Info, NetworkClient, NetworkInfo};
use crate::rpc::client::Client;
use std::path::PathBuf;

const RPC_URL: &str = "https://rpc.betanet.near.org";

/// Betanet related configuration for interacting with betanet. Look at
/// [`workspaces::betanet`] for how to spin up a [`Worker`] that can be
/// used to interact with betanet. Note that betanet account creation
/// is not currently supported, and these calls into creating a betanet
/// worker is meant for retrieving data and/or making queries only.
/// Also, note that betanet can be unstable and does not provide an
/// archival endpoint similiar to that of mainnet.
///
/// [`workspaces::betanet`]: crate::betanet
/// [`workspaces::betanet_archival`]: crate::betanet_archival
/// [`Worker`]: crate::Worker
pub struct Betanet {
    client: Client,
    info: Info,
}

impl Betanet {
    pub(crate) async fn new() -> crate::result::Result<Self> {
        let client = Client::new(RPC_URL.into());
        client.wait_for_rpc().await?;

        Ok(Self {
            client,
            info: Info {
                name: "betanet".into(),
                root_id: "near".parse().unwrap(),
                keystore_path: PathBuf::from(".near-credentials/betanet/"),
                rpc_url: RPC_URL.into(),
            },
        })
    }
}

impl NetworkClient for Betanet {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Betanet {
    fn info(&self) -> &Info {
        &self.info
    }
}
