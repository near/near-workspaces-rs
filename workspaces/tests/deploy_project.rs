#![recursion_limit = "256"]
use test_log::test;
use workspaces::prelude::*;

#[test(tokio::test)]
async fn test_dev_deploy_project() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let wasm = workspaces::compile_project("./tests/test-contracts/status-message").await?;
    let contract = worker.dev_deploy(&wasm).await?;

    let _res = contract
        .call(&worker, "set_status")
        .args_json(("foo",))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    let res = contract
        .call(&worker, "get_status")
        .args_json((contract.id(),))?
        .view()
        .await?;
    assert_eq!(res.json::<String>()?, "foo");

    Ok(())
}
