use crate::network::Sandbox;
use near_sandbox_utils::SandboxHandle;
use portpicker::pick_unused_port;
use tracing::info;

pub struct SandboxServer {
    pub(crate) rpc_port: u16,
    pub(crate) net_port: u16,
    sandbox_handle: Option<SandboxHandle>,
}

impl SandboxServer {
    pub fn new(rpc_port: u16, net_port: u16) -> Self {
        Self {
            rpc_port,
            net_port,
            sandbox_handle: None,
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        info!(target: "workspaces", "Starting up sandbox at localhost:{}", self.rpc_port);
        let home_dir = Sandbox::home_dir(self.rpc_port);

        // Remove dir if it already exists:
        let _ = std::fs::remove_dir_all(&home_dir);
        near_sandbox_utils::init(&home_dir)?.wait()?;

        let sandbox_handle = near_sandbox_utils::run(&home_dir, self.rpc_port, self.net_port)?;
        info!(target: "workspaces", "Started sandbox: pid={:?}", sandbox_handle.sandbox_process.id());
        self.sandbox_handle = Some(sandbox_handle);

        Ok(())
    }

    pub fn rpc_addr(&self) -> String {
        format!("http://localhost:{}", self.rpc_port)
    }
}

impl Default for SandboxServer {
    fn default() -> Self {
        let rpc_port = pick_unused_port().expect("no ports free");
        let net_port = pick_unused_port().expect("no ports free");
        Self::new(rpc_port, net_port)
    }
}

impl Drop for SandboxServer {
    fn drop(&mut self) {
        if self.sandbox_handle.is_none() {
            return;
        }

        let child = &mut self.sandbox_handle.as_mut().unwrap().sandbox_process;

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
    }
}
