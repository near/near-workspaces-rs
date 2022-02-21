use anyhow::anyhow;
use cargo_metadata::Message;
use std::fmt::Debug;
use std::path::Path;
use std::process::{Command, Stdio};
use tracing::debug;

pub fn build_cargo_project<P: AsRef<Path> + Debug>(
    project_path: P,
) -> anyhow::Result<Vec<Message>> {
    let cmd = Command::new("cargo")
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--message-format=json",
        ])
        .current_dir(&project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = cmd.wait_with_output()?;
    debug!(
        target: "workspaces",
        "Building project '{:?}' resulted in status {:?}",
        &project_path, output.status
    );
    if output.status.success() {
        let reader = std::io::BufReader::new(output.stdout.as_slice());
        Ok(cargo_metadata::Message::parse_stream(reader)
            .map(|m| m.unwrap())
            .collect())
    } else {
        Err(anyhow!(
            "Failed to build project '{:?}'.\n\
            Stderr:\n\
            {}\n\
            Stdout:\n\
            {}",
            project_path,
            String::from_utf8(output.stderr)?,
            String::from_utf8(output.stdout)?,
        ))
    }
}
