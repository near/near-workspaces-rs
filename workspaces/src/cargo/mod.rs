use crate::error::ErrorKind;

/// Builds the cargo project located at `project_path` and returns the generated wasm file contents.
///
/// NOTE: This function does not check whether the resulting wasm file is a valid smart
/// contract or not.
/// NOTE: This function builds the project using default features
pub async fn compile_project(project_path: &str) -> crate::Result<Vec<u8>> {
    let project_path = std::fs::canonicalize(project_path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ErrorKind::Io.message(format!(
            "Incorrect file supplied to compile_project('{project_path}')"
        )),
        _ => ErrorKind::Io.custom(e),
    })?;

    // `no_abi` has become flipped true -> false
    let cargo_opts = cargo_near_build::BuildOpts {
        no_locked: true,
        manifest_path: Some(
            cargo_near_build::camino::Utf8PathBuf::from_path_buf(project_path.join("Cargo.toml"))
                .map_err(|error_path| {
                ErrorKind::Io.custom(format!(
                    "Unable to construct UTF-8 path from: {}",
                    error_path.display()
                ))
            })?,
        ),
        ..Default::default()
    };

    let compile_artifact =
        cargo_near_build::build_with_cli(cargo_opts).map_err(|e| ErrorKind::Io.custom(e))?;

    let file = compile_artifact
        .canonicalize()
        .map_err(|e| ErrorKind::Io.custom(e))?;
    tokio::fs::read(file)
        .await
        .map_err(|e| ErrorKind::Io.custom(e))
}
