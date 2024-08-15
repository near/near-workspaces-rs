use crate::error::ErrorKind;

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

    let cargo_opts = cargo_near::BuildOpts {
        no_release: false,
        no_embed_abi: false,
        no_locked: true,
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
    };

    let compile_artifact = cargo_near::build(cargo_opts).map_err(|e| ErrorKind::Io.custom(e))?;

    let file = compile_artifact
        .path
        .canonicalize()
        .map_err(|e| ErrorKind::Io.custom(e))?;
    tokio::fs::read(file)
        .await
        .map_err(|e| ErrorKind::Io.custom(e))
}
