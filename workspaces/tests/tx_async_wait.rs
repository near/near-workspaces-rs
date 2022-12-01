#![cfg(feature = "unstable")]

#[tokio::test]
async fn test_async_wait_finality() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = workspaces::compile_project("./tests/test-contracts/promise-chain").await?;
    let contract = worker.dev_deploy(&wasm).await?;
    let account = worker.dev_create_account().await?;

    let result = account
        .call(contract.id(), "a")
        .max_gas()
        .transact_async()
        .await?;
    assert_eq!(result.await?.json::<String>()?, "Test string");

    Ok(())
}
