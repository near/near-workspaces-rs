use crate::error::ErrorKind;

use cargo_near::commands::build_command::{BuildCommand, BuildContext, CliBuildCommand};

/// Builds the cargo project located at `project_path` and returns the generated wasm file contents.
///
/// NOTE: This function does not check whether the resulting wasm file is a valid smart
/// contract or not.
/// NOTE: This function builds the project using default features
pub async fn compile_project(project_path: &str) -> crate::Result<Vec<u8>> {
    let project_path = std::fs::canonicalize(project_path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ErrorKind::Io.message(format!(
            "Incorrect file supplied to compile_project('{}')",
            project_path
        )),
        _ => ErrorKind::Io.custom(e),
    })?;

    // Hack to create a structure as no_docker is private in BuildCommand
    let cargo_near_build_command: BuildCommand = CliBuildCommand {
        no_release: false,
        no_docker: true,
        no_locked: false,
        no_embed_abi: false,
        no_doc: true,
        color: None,
        no_abi: true,
        out_dir: None,
        manifest_path: Some(
            cargo_near::types::utf8_path_buf::Utf8PathBuf::from_path_buf(
                project_path.join("Cargo.toml"),
            )
            .map_err(|error_path| {
                ErrorKind::Io.custom(format!(
                    "Unable to construct UTF-8 path from: {}",
                    error_path.display()
                ))
            })?,
        ),
        features: None,
        no_default_features: false,
    }
    .into();

    let compile_artifact = cargo_near_build_command
        .run(BuildContext::Build)
        .map_err(|e| ErrorKind::Io.custom(e))?;

    let file = compile_artifact
        .path
        .canonicalize()
        .map_err(|e| ErrorKind::Io.custom(e))?;
    tokio::fs::read(file)
        .await
        .map_err(|e| ErrorKind::Io.custom(e))
}
