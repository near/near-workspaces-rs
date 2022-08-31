use serde_json::json;

const STATUS_MSG_CONTRACT: &[u8] = include_bytes!("../../examples/res/status_message.wasm");

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_parallel() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(STATUS_MSG_CONTRACT).await?;
    let account = worker.dev_create_account().await?;

    let parallel_tasks = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]
        .iter()
        .map(|msg| {
            let id = contract.id().clone();
            let account = account.clone();
            tokio::spawn(async move {
                account
                    .call(&id, "set_status")
                    .args_json(json!({
                        "message": msg,
                    }))
                    .transact()
                    .await?
                    .into_result()?;
                anyhow::Result::<()>::Ok(())
            })
        });
    futures::future::join_all(parallel_tasks).await;

    // Check the final set message. This should be random each time this test function is called:
    let final_set_msg = account
        .call(contract.id(), "get_status")
        .args_json(json!({ "account_id": account.id() }))
        .view()
        .await?
        .json::<String>()?;
    println!("Final set message: {:?}", final_set_msg);

    Ok(())
}
