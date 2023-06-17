use async_process::Command;
use cargo_metadata::{Error as MetadataError, Message, Metadata, MetadataCommand};
use tracing::debug;

use std::env;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Stdio;
use std::convert::TryInto;

use crate::error::ErrorKind;
use crate::Result;


use cargo_near::build::run;
use cargo_near::{BuildCommand, ColorPreference};
use cargo_near::util::CompilationArtifact;



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

async fn build_cargo_project<P: AsRef<Path> + Debug>(project_path: P) -> Result<CompilationArtifact>
{
    let  manifest_path = project_path.as_ref().join("Cargo.toml");
    let command= BuildCommand{
        release:true,
        embed_abi:true,
        doc:false,
        color:ColorPreference::Always,
        no_abi:true,
        out_dir:None,
        manifest_path:Some(manifest_path.try_into()
        .map_err(|e| ErrorKind::Io.custom(e))?
    )
    };
    let result=run(command).map_err(|e| ErrorKind::Io.custom(e))?;
    Ok(result)
}

/// Builds the cargo project located at `project_path` and returns the generated wasm file contents.
///
/// NOTE: This function does not check whether the resulting wasm file is a valid smart
/// contract or not.
pub async fn compile_project(project_path: &str) -> Result<Vec<u8>> 
{
    let project_path = fs::canonicalize(project_path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => ErrorKind::Io.message(format!(
            "Incorrect file supplied to compile_project('{}')",
            project_path
        )),
        _ => ErrorKind::Io.custom(e),
    })?;
    let compile_artifact = build_cargo_project(project_path).await?;
    let file = compile_artifact.path.canonicalize().map_err(|e| ErrorKind::Io.custom(e))?;
    Ok(tokio::fs::read(file).await.map_err(|e| ErrorKind::Io.custom(e))?)
  
}
