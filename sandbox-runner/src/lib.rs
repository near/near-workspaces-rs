use std::process::{Child, Command};
use std::sync::Once;

static INIT_ONCE_SANDBOX: Once = Once::new();

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
        sandbox_start();

        // Let the sandbox startup for a bit before running anything else.
        // TODO: ping the sandbox instead
        use std::{thread, time::Duration};
        thread::sleep(Duration::from_secs(3));
    });
}
