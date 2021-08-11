use std::process::{Command, Child};
use std::sync::Once;

use once_cell::sync::OnceCell;

static INIT_ONCE_SANDBOX: Once = Once::new();
static SANDBOX_CLEANUP: OnceCell<SandboxCleanup> = OnceCell::new();

fn sandbox_start() -> Child {
    // TODO: stdout/stderr to file instead
    println!("Starting up sandbox...");
    let child = if cfg!(target_os = "windows") {
        Command::new("near-sandbox")
            .args(&["run"])
            .spawn()
            .expect("failed to startup near-sandbox")
    } else {
        Command::new("near-sandbox")
            .arg("run")
            .spawn()
            .expect("failed to startup near-sandbox")
    };
    println!("Started sandbox: {:?}", child.id());
    child
}

// #[allow(dead_code)]
pub fn sandbox_setup() {
    INIT_ONCE_SANDBOX.call_once(|| {
        let child = sandbox_start();
        let child_id = child.id();
        SANDBOX_CLEANUP.set(SandboxCleanup { child_process: child })
            .expect(&format!("failed to set SandboxCleanup with child pid: {}", child_id));

        // Let the sandbox startup for a bit before running anything else.
        // TODO: ping the sandbox instead
        use std::{thread, time::Duration};
        thread::sleep(Duration::from_secs(3));
    });
}

#[derive(Debug)]
pub struct SandboxCleanup {
    child_process: Child,
}

impl core::ops::Drop for SandboxCleanup {
    fn drop(&mut self) {
        self.child_process.kill()
            .map_err(|e| format!("Could not cleanup sandbox due to: {:?}", e))
            .unwrap();
    }
}

