#![cfg(feature = "unstable")]
#![recursion_limit = "256"]
use near_workspaces::{
    cargo_near_build as near_build,
    error::{Error, ErrorKind},
};
use test_log::test;

#[test(tokio::test)]
async fn test_dev_deploy_project() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    // TODO: uncomment back when nearcore becomes compatible with rust 1.87
    // let wasm = near_workspaces::compile_project("./tests/test-contracts/status-message").await?;
    let wasm = {
        let build_opts = cargo_near_build::BuildOpts::builder()
            .no_locked(true)
            .manifest_path(
                cargo_near_build::camino::Utf8PathBuf::from(
                    "./tests/test-contracts/status-message",
                )
                .join("Cargo.toml"),
            )
            .override_toolchain("1.86.0")
            .build();
        let compile_artifact = near_build::build_with_cli(build_opts)
            .map_err(|e| Error::custom(ErrorKind::Other, e))?;
        let file = compile_artifact.canonicalize()?;
        std::fs::read(file)?
    };

    let contract = worker.dev_deploy(&wasm).await?;

    contract
        .call("set_status")
        .args_json(("foo",))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    let res = contract
        .call("get_status")
        .args_json((contract.id(),))
        .view()
        .await?;
    assert_eq!(res.json::<String>()?, "foo");

    Ok(())
}
