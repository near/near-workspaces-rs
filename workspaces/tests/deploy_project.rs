#![cfg(feature = "unstable")]
#![recursion_limit = "256"]
use test_log::test;
use workspaces::prelude::*;

#[test(tokio::test)]
async fn test_dev_deploy_project() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = workspaces::compile_project("./tests/test-contracts/status-message").await?;
    let contract = worker.dev_deploy(&wasm).await?;

    let _res = contract
        .call("set_status")
        .args_json(("foo",))
        .max_gas()
        .transact()
        .await?;

    let res = contract
        .call("get_status")
        .args_json((contract.id(),))
        .view()
        .await?;
    assert_eq!(res.json::<String>()?, "foo");

    Ok(())
}
