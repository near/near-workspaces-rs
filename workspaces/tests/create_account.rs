use workspaces::prelude::*;

#[tokio::test]
async fn test_subaccount_creation() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let account = worker.dev_create().await?;

    let sub = account
        .create_subaccount(&worker, "subaccount".into())
        .transact()
        .await?
        .into_result()?;

    let expect_id = format!("subaccount.{}", account.id());
    let actual_id = sub.id().to_string();

    assert_eq!(actual_id, expect_id);

    Ok(())
}
