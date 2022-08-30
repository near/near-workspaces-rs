#![cfg(feature = "unstable")]
#![recursion_limit = "256"]
use test_log::test;

#[test(tokio::test)]
async fn test_dev_deploy_project() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = workspaces::compile_project("./tests/test-contracts/status-message").await?;
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
