use std::process::Child;

use portpicker::pick_unused_port;

use crate::network::Sandbox;

pub struct SandboxServer {
    pub(crate) rpc_port: u16,
    pub(crate) net_port: u16,
    process: Option<Child>,
}

impl SandboxServer {
    pub fn new(rpc_port: u16, net_port: u16) -> Self {
        Self {
            rpc_port,
            net_port,
            process: None,
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        println!("Starting up sandbox at localhost:{}", self.rpc_port);
        let home_dir = Sandbox::home_dir(self.rpc_port);

        // Remove dir if it already exists:
        let _ = std::fs::remove_dir_all(&home_dir);
        near_sandbox_utils::init(&home_dir)?.wait()?;

        let child = near_sandbox_utils::run(&home_dir, self.rpc_port, self.net_port)?;
        println!("Started sandbox: pid={:?}", child.id());
        self.process = Some(child);

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
        if self.process.is_none() {
            return;
        }

        let child = self.process.as_mut().unwrap();

        eprintln!(
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
