use std::fs::File;

use crate::error::{ErrorKind, SandboxErrorCode};
use crate::result::Result;

use async_process::Child;
use fs2::FileExt;
use portpicker::pick_unused_port;
use tempfile::TempDir;
use tracing::info;

use near_sandbox_utils as sandbox;

/// Acquire an unused port and lock it for the duration until the sandbox server has
/// been started.
fn acquire_unused_port() -> Result<(u16, File)> {
    loop {
        let port = pick_unused_port()
            .ok_or_else(|| SandboxErrorCode::InitFailure.message("no ports free"))?;
        let lockpath = std::env::temp_dir().join(format!("near-sandbox-port{}.lock", port));
        let lockfile = File::create(lockpath).map_err(|err| {
            ErrorKind::Io.full(format!("failed to create lockfile for port {}", port), err)
        })?;
        if lockfile.try_lock_exclusive().is_ok() {
            break Ok((port, lockfile));
        }
    }
}

async fn init_home_dir() -> Result<TempDir> {
    let home_dir = tempfile::tempdir().map_err(|e| ErrorKind::Io.custom(e))?;
    let output = sandbox::init(&home_dir)
        .map_err(|e| SandboxErrorCode::InitFailure.custom(e))?
        .output()
        .await
        .map_err(|e| SandboxErrorCode::InitFailure.custom(e))?;
    info!(target: "workspaces", "sandbox init: {:?}", output);

    Ok(home_dir)
}

pub struct SandboxServer {
    pub(crate) rpc_port: u16,
    pub(crate) net_port: u16,
    pub(crate) home_dir: TempDir,

    rpc_port_lock: Option<File>,
    net_port_lock: Option<File>,
    process: Option<Child>,
}

impl SandboxServer {
    pub(crate) async fn run_new() -> Result<Self> {
        // Supress logs for the sandbox binary by default:
        supress_sandbox_logs_if_required();

        // Try running the server with the follow provided rpc_ports and net_ports
        let (rpc_port, rpc_port_lock) = acquire_unused_port()?;
        let (net_port, net_port_lock) = acquire_unused_port()?;
        let home_dir = init_home_dir().await?;

        // Configure `$home_dir/config.json` to our liking. Sandbox requires extra settings
        // for the best user experience, and being able to offer patching large state payloads.
        crate::network::config::set_sandbox_configs(&home_dir)?;
        let child = sandbox::run(&home_dir, rpc_port, net_port)
            .map_err(|e| SandboxErrorCode::RunFailure.custom(e))?;

        info!(target: "workspaces", "Started up sandbox at localhost:{} with pid={:?}", rpc_port, child.id());

        Ok(Self {
            rpc_port,
            net_port,
            home_dir,
            rpc_port_lock: Some(rpc_port_lock),
            net_port_lock: Some(net_port_lock),
            process: Some(child),
        })
    }

    /// Unlock port lockfiles that were used to avoid port contention when starting up
    /// the sandbox node.
    pub(crate) fn unlock_lockfiles(&mut self) -> Result<()> {
        if let Some(rpc_port_lock) = self.rpc_port_lock.take() {
            rpc_port_lock.unlock().map_err(|e| {
                ErrorKind::Io.full(
                    format!("failed to unlock lockfile for rpc_port={}", self.rpc_port),
                    e,
                )
            })?;
        }
        if let Some(net_port_lock) = self.net_port_lock.take() {
            net_port_lock.unlock().map_err(|e| {
                ErrorKind::Io.full(
                    format!("failed to unlock lockfile for net_port={}", self.net_port),
                    e,
                )
            })?;
        }

        Ok(())
    }

    pub fn rpc_addr(&self) -> String {
        format!("http://localhost:{}", self.rpc_port)
    }
}

impl Drop for SandboxServer {
    fn drop(&mut self) {
        if self.process.is_none() {
            return;
        }

        let child = self.process.as_mut().unwrap();

        info!(
            target: "workspaces",
            "Cleaning up sandbox: port={}, pid={}",
            self.rpc_port,
            child.id()
        );

        child
            .kill()
            .map_err(|e| format!("Could not cleanup sandbox due to: {:?}", e))
            .unwrap();

        // Unlock the ports just in case they have not been preemptively done.
        self.unlock_lockfiles().unwrap();
    }
}

/// Turn off neard-sandbox logs by default. Users can turn them back on with
/// NEAR_ENABLE_SANDBOX_LOG=1 and specify further paramters with the custom
/// NEAR_SANDBOX_LOG for higher levels of specificity. NEAR_SANDBOX_LOG args
/// will be forward into RUST_LOG environment variable as to not conflict
/// with similar named log targets.
fn supress_sandbox_logs_if_required() {
    if let Ok(val) = std::env::var("NEAR_ENABLE_SANDBOX_LOG") {
        if val != "0" {
            return;
        }
    }

    // non-exhaustive list of targets to supress, since choosing a default LogLevel
    // does nothing in this case, since nearcore seems to be overriding it somehow:
    std::env::set_var("NEAR_SANDBOX_LOG", "near=error,stats=error,network=error");
}
