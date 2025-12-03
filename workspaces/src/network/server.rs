use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;

use crate::error::{ErrorKind, SandboxErrorCode};
use crate::result::Result;
use crate::types::SecretKey;

use near_account_id::AccountId;
use reqwest::Url;

use tracing::info;

use near_sandbox as sandbox;
use tokio::net::TcpListener;

/// Request an unused port from the OS.
pub async fn pick_unused_port() -> Result<u16> {
    // Port 0 means the OS gives us an unused port
    // Important to use localhost as using 0.0.0.0 leads to users getting brief firewall popups to
    // allow inbound connections on MacOS.
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| ErrorKind::Io.full("failed to bind to random port", err))?;
    let port = listener
        .local_addr()
        .map_err(|err| ErrorKind::Io.full("failed to get local address for random port", err))?
        .port();
    Ok(port)
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ValidatorKey {
    HomeDir(PathBuf),
    Known(AccountId, SecretKey),
}

pub struct SandboxServer {
    pub(crate) validator_key: ValidatorKey,
    rpc_addr: Url,
    sandbox_instance: Option<near_sandbox::Sandbox>,
}

impl SandboxServer {
    /// Connect a sandbox server that's already been running, provided we know the rpc_addr
    /// and home_dir pointing to the sandbox process.
    pub(crate) fn new(rpc_addr: String, validator_key: ValidatorKey) -> Result<Self> {
        let rpc_addr = Url::parse(&rpc_addr).map_err(|e| {
            SandboxErrorCode::InitFailure.full(format!("Invalid rpc_url={rpc_addr}"), e)
        })?;
        Ok(Self {
            validator_key,
            rpc_addr,
            sandbox_instance: None,
        })
    }

    /// Run a new SandboxServer, spawning the sandbox node in the process.
    #[allow(dead_code)]
    pub(crate) async fn run_new() -> Result<Self> {
        Self::run_new_with_version(sandbox::DEFAULT_NEAR_SANDBOX_VERSION).await
    }

    pub(crate) async fn run_new_with_version(version: &str) -> Result<Self> {
        // Suppress logs for the sandbox binary by default:
        suppress_sandbox_logs_if_required();

        let sandbox_config = near_sandbox::SandboxConfig {
            additional_accounts: vec![sandbox::GenesisAccount {
                account_id: "registrar".parse().unwrap(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let sandbox_instance =
            sandbox::Sandbox::start_sandbox_with_config_and_version(sandbox_config, version)
                .await
                .map_err(|e| SandboxErrorCode::RunFailure.custom(e))?;

        info!(target: "workspaces", "Started up sandbox at {}", sandbox_instance.rpc_addr);

        let rpc_addr: Url = sandbox_instance
            .rpc_addr
            .parse()
            .expect("static scheme and host name with variable u16 port numbers form valid urls");

        let dir = sandbox_instance.home_dir.path().to_path_buf();

        Ok(Self {
            validator_key: ValidatorKey::HomeDir(dir),
            rpc_addr,
            sandbox_instance: Some(sandbox_instance),
        })
    }

    pub fn rpc_port(&self) -> Option<u16> {
        self.rpc_addr.port()
    }

    pub fn rpc_addr(&self) -> String {
        self.rpc_addr.to_string()
    }
}

impl Drop for SandboxServer {
    fn drop(&mut self) {
        if self.sandbox_instance.is_some() {
            info!(
                target: "workspaces",
                "Cleaning up sandbox"
            );
        }
    }
}

/// Turn off neard-sandbox logs by default. Users can turn them back on with
/// NEAR_ENABLE_SANDBOX_LOG=1 and specify further parameters with the custom
/// NEAR_SANDBOX_LOG for higher levels of specificity. NEAR_SANDBOX_LOG args
/// will be forward into RUST_LOG environment variable as to not conflict
/// with similar named log targets.
fn suppress_sandbox_logs_if_required() {
    if let Ok(val) = std::env::var("NEAR_ENABLE_SANDBOX_LOG") {
        if val != "0" {
            return;
        }
    }

    // non-exhaustive list of targets to suppress, since choosing a default LogLevel
    // does nothing in this case, since nearcore seems to be overriding it somehow:
    std::env::set_var("NEAR_SANDBOX_LOG", "near=error,stats=error,network=error");
}
