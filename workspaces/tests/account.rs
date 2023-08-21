#![recursion_limit = "256"]
use serde_json::{Map, Value};
use test_log::test;

use std::fs::File;
use std::path::Path;

#[test(tokio::test)]
async fn test_subaccount_creation() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let account = worker.dev_create_account().await?;

    let sub = account
        .create_subaccount("subaccount")
        .transact()
        .await?
        .into_result()?;

    let expect_id = format!("subaccount.{}", account.id());
    let actual_id = sub.id().to_string();

    assert_eq!(actual_id, expect_id);

    // Check if the stored credentials match with the subaccount created.
    let savedir = Path::new("../target/credentials");
    sub.store_credentials(savedir).await?;
    let creds = File::open(savedir.join(format!("{}.json", sub.id())))?;
    let contents: Map<String, Value> = serde_json::from_reader(creds)?;
    assert_eq!(
        contents.get("account_id"),
        Some(&Value::String(sub.id().to_string()))
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_delete_account() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    let (first_account, second_account) = (
        worker.dev_create_account().await?,
        worker.dev_create_account().await?,
    );

    _ = first_account.delete_account(second_account.id()).await?;

    // All sandbox accounts start with a balance of `1_000_000_000_000_000_000_000_000` tokens.
    // On account deletion, first_account balance is debited to second_account as beneficary.
    assert!(second_account.view_account().await?.balance > 1_900_000_000_000_000_000_000_000,);

    Ok(())
}
