use anyhow::anyhow;
use async_process::Command;
use cargo_metadata::Message;
use std::env;
use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::process::Stdio;
use tracing::debug;

async fn build_cargo_project<P: AsRef<Path> + Debug>(
    project_path: P,
) -> anyhow::Result<Vec<Message>> {
    let mut cmd = match env::var_os("CARGO") {
        Some(cargo) => Command::new(cargo),
        None => Command::new("cargo"),
    };
    let output = cmd
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--message-format=json",
            "--offline"
        ])
        .current_dir(&project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;
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

/// Builds the cargo project located at `project_path` and returns the generated wasm file contents.
///
/// NOTE: This function does not check whether the resulting wasm file is a valid smart
/// contract or not.
pub async fn compile_project(project_path: &str) -> anyhow::Result<Vec<u8>> {
    let messages = build_cargo_project(fs::canonicalize(project_path)?).await?;
    // We find the last compiler artifact message which should contain information about the
    // resulting .wasm file
    let compile_artifact = messages
        .iter()
        .filter_map(|m| match m {
            cargo_metadata::Message::CompilerArtifact(artifact) => Some(artifact),
            _ => None,
        })
        .last()
        .ok_or(anyhow!(
            "Cargo failed to produce any compilation artifacts. \
                 Please check that your project contains a NEAR smart contract."
        ))?;
    // The project could have generated many auxiliary files, we are only interested in .wasm files
    let wasm_files = compile_artifact
        .filenames
        .iter()
        .filter(|f| f.as_str().ends_with(".wasm"))
        .collect::<Vec<_>>();
    match wasm_files.as_slice() {
        [] => Err(anyhow!(
            "Compilation resulted in no '.wasm' target files. \
                 Please check that your project contains a NEAR smart contract."
        )),
        [file] => Ok(tokio::fs::read(file.canonicalize()?).await?),
        _ => Err(anyhow!(
            "Compilation resulted in more than one '.wasm' target file: {:?}",
            wasm_files
        )),
    }
}
