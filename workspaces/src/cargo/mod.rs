use std::convert::TryInto;

use cargo_near::{
    commands::build_command::{build, BuildCommand},
    types::utf8_path_buf::Utf8PathBufInner,
};

use crate::error::ErrorKind;

/// Builds the cargo project located at `project_path` and returns the generated wasm file contents.
///
/// NOTE: This function does not check whether the resulting wasm file is a valid smart
/// contract or not.
pub async fn compile_project(project_path: &str) -> crate::Result<Vec<u8>> {
    let project_path = std::fs::canonicalize(project_path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ErrorKind::Io.message(format!(
            "Incorrect file supplied to compile_project('{}')",
            project_path
        )),
        _ => ErrorKind::Io.custom(e),
    })?;

    let cargo_near_build_command = BuildCommand {
        release: true,
        embed_abi: true,
        doc: false,
        color: None,
        no_abi: true,
        out_dir: None,
        manifest_path: Some(Utf8PathBufInner(
            project_path
                .join("Cargo.toml")
                .try_into()
                .map_err(|e| ErrorKind::Io.custom(e))?,
        )),
    };

    let compile_artifact =
        build::run(cargo_near_build_command).map_err(|e| ErrorKind::Io.custom(e))?;

    let file = compile_artifact
        .path
        .canonicalize()
        .map_err(|e| ErrorKind::Io.custom(e))?;
    tokio::fs::read(file)
        .await
        .map_err(|e| ErrorKind::Io.custom(e))
}
