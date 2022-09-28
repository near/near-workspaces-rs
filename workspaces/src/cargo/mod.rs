use async_process::Command;
use cargo_metadata::{Error as MetadataError, Message, Metadata, MetadataCommand};
use tracing::debug;

use std::env;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Stdio;

use crate::error::ErrorKind;
use crate::Result;

fn cargo_bin() -> Command {
    match env::var_os("CARGO") {
        Some(cargo) => Command::new(cargo),
        None => Command::new("cargo"),
    }
}

/// Fetch current project's metadata (i.e. project invoking this method, not the one that we are
/// trying to compile).
fn root_cargo_metadata() -> Result<Metadata> {
    MetadataCommand::new().exec().map_err(|e| match e {
        // comes from cargo metadata command error message, so IO should be appropriate
        MetadataError::CargoMetadata { stderr } => ErrorKind::Io.message(stderr),
        MetadataError::Io(err) => ErrorKind::Io.custom(err),
        MetadataError::Utf8(err) => ErrorKind::DataConversion.custom(err),
        MetadataError::ErrUtf8(err) => ErrorKind::DataConversion.custom(err),
        MetadataError::Json(err) => ErrorKind::DataConversion.custom(err),
        err @ MetadataError::NoJson => ErrorKind::DataConversion.message(err.to_string()),
    })
}

async fn build_cargo_project<P: AsRef<Path> + Debug>(project_path: P) -> Result<Vec<Message>> {
    let metadata = root_cargo_metadata()?;
    let output = cargo_bin()
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--message-format=json",
            "--target-dir",
            metadata.target_directory.as_str(),
        ])
        .current_dir(&project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| ErrorKind::Io.custom(e))?;

    debug!(
        target: "workspaces",
        "Building project '{:?}' resulted in status {:?}",
        &project_path, output.status
    );

    if output.status.success() {
        let reader = std::io::BufReader::new(output.stdout.as_slice());
        Ok(Message::parse_stream(reader).map(|m| m.unwrap()).collect())
    } else {
        Err(ErrorKind::Io.message(format!(
            "Failed to build project '{:?}'.\n\
            Stderr:\n\
            {}\n\
            Stdout:\n\
            {}",
            project_path,
            String::from_utf8(output.stderr).map_err(|e| ErrorKind::DataConversion.custom(e))?,
            String::from_utf8(output.stdout).map_err(|e| ErrorKind::DataConversion.custom(e))?,
        )))
    }
}

/// Builds the cargo project located at `project_path` and returns the generated wasm file contents.
///
/// NOTE: This function does not check whether the resulting wasm file is a valid smart
/// contract or not.
pub async fn compile_project(project_path: &str) -> Result<Vec<u8>> {
    let project_path = fs::canonicalize(project_path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => ErrorKind::Io.message(format!(
            "Incorrect file supplied to compile_project('{}')",
            project_path
        )),
        _ => ErrorKind::Io.custom(e),
    })?;
    let messages = build_cargo_project(project_path).await?;

    // We find the last compiler artifact message which should contain information about the
    // resulting .wasm file
    let compile_artifact = messages
        .iter()
        .filter_map(|m| match m {
            cargo_metadata::Message::CompilerArtifact(artifact) => Some(artifact),
            _ => None,
        })
        .last()
        .ok_or_else(|| {
            ErrorKind::Io.message(
                "Cargo failed to produce any compilation artifacts. \
                 Please check that your project contains a NEAR smart contract.",
            )
        })?;
    // The project could have generated many auxiliary files, we are only interested in .wasm files
    let wasm_files = compile_artifact
        .filenames
        .iter()
        .filter(|f| f.as_str().ends_with(".wasm"))
        .collect::<Vec<_>>();
    match wasm_files.as_slice() {
        [] => Err(ErrorKind::Io.message(
            "Compilation resulted in no '.wasm' target files. \
                 Please check that your project contains a NEAR smart contract.",
        )),
        [file] => {
            let file = file.canonicalize().map_err(|e| ErrorKind::Io.custom(e))?;
            Ok(tokio::fs::read(file)
                .await
                .map_err(|e| ErrorKind::Io.custom(e))?)
        }
        _ => Err(ErrorKind::Io.message(format!(
            "Compilation resulted in more than one '.wasm' target file: {:?}",
            wasm_files
        ))),
    }
}
