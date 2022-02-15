use super::{
    Account, AllowDevAccountCreation, AllowStatePatching, CallExecution, Contract, NetworkClient,
    NetworkInfo, TopLevelAccountCreator,
};
use crate::network::server::SandboxServer;
use crate::network::Info;
use crate::rpc::client::Client;
use crate::types::{AccountId, Balance, InMemorySigner, SecretKey};
use async_mutex::Mutex;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::future::Future;
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(not(feature = "sandbox-parallel"))]
static SHARED_SERVER: Lazy<SandboxServer> = Lazy::new(|| {
    let mut server = SandboxServer::default();
    server.start().unwrap();
    server
});

// Using a shared sandbox instance is thread-safe as long as all threads use it with their own
// account. This means, however, is that the creation of these accounts should be sequential in
// order to avoid duplicated nonces.
#[cfg(not(feature = "sandbox-parallel"))]
static TLA_ACCOUNT_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

// Constant taken from nearcore crate to avoid dependency
pub(crate) const NEAR_BASE: Balance = 1_000_000_000_000_000_000_000_000;

pub(crate) const DEFAULT_DEPOSIT: Balance = 100 * NEAR_BASE;

#[cfg(not(feature = "sandbox-parallel"))]
pub struct Sandbox {
    client: Client,
    info: Info,
}

#[cfg(feature = "sandbox-parallel")]
pub struct Sandbox {
    server: SandboxServer,
    client: Client,
    info: Info,
}

impl Sandbox {
    pub(crate) fn home_dir(port: u16) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("sandbox-{}", port));
        path
    }

    #[cfg(not(feature = "sandbox-parallel"))]
    pub(crate) fn root_signer(&self) -> InMemorySigner {
        let mut path = Self::home_dir(SHARED_SERVER.rpc_port);
        path.push("validator_key.json");

        InMemorySigner::from_file(&path)
    }

    #[cfg(feature = "sandbox-parallel")]
    pub(crate) fn root_signer(&self) -> InMemorySigner {
        let mut path = Self::home_dir(self.server.rpc_port);
        path.push("validator_key.json");

        InMemorySigner::from_file(&path)
    }

    #[cfg(not(feature = "sandbox-parallel"))]
    pub(crate) fn new() -> Self {
        let client = Client::new(SHARED_SERVER.rpc_addr());
        let info = Info {
            name: "sandbox-shared".to_string(),
            root_id: AccountId::from_str("test.near").unwrap(),
            keystore_path: PathBuf::from(".near-credentials/sandbox/"),
            rpc_url: SHARED_SERVER.rpc_addr(),
        };

        Self { client, info }
    }

    #[cfg(feature = "sandbox-parallel")]
    pub(crate) fn new() -> Self {
        let mut server = SandboxServer::default();
        server.start().unwrap();

        let client = Client::new(server.rpc_addr());
        let info = Info {
            name: "sandbox".to_string(),
            root_id: AccountId::from_str("test.near").unwrap(),
            keystore_path: PathBuf::from(".near-credentials/sandbox/"),
            rpc_url: server.rpc_addr(),
        };

        Self {
            server,
            client,
            info,
        }
    }
}

impl AllowStatePatching for Sandbox {}

impl AllowDevAccountCreation for Sandbox {}

async fn tla_guarded<F: FnOnce() -> Fut, Fut: Future<Output = T>, T>(f: F) -> T {
    #[cfg(not(feature = "sandbox-parallel"))]
    let guard = TLA_ACCOUNT_MUTEX.lock().await;
    let result = f().await;
    #[cfg(not(feature = "sandbox-parallel"))]
    drop(guard);
    result
}

#[async_trait]
impl TopLevelAccountCreator for Sandbox {
    async fn create_tla(
        &self,
        id: AccountId,
        sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account>> {
        let root_signer = self.root_signer();

        let outcome = tla_guarded(|| {
            self.client
                .create_account(&root_signer, &id, sk.public_key(), DEFAULT_DEPOSIT)
        })
        .await?;

        let signer = InMemorySigner::from_secret_key(id.clone(), sk);
        Ok(CallExecution {
            result: Account::new(id, signer),
            details: outcome.into(),
        })
    }

    async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract>> {
        let root_signer = self.root_signer();
        let outcome = tla_guarded(|| {
            self.client.create_account_and_deploy(
                &root_signer,
                &id,
                sk.public_key(),
                DEFAULT_DEPOSIT,
                wasm.into(),
            )
        })
        .await?;

        let signer = InMemorySigner::from_secret_key(id.clone(), sk);
        Ok(CallExecution {
            result: Contract::new(id, signer),
            details: outcome.into(),
        })
    }
}

impl NetworkClient for Sandbox {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Sandbox {
    fn info(&self) -> &Info {
        &self.info
    }
}
