use futures::future::join_all;
use serde_json::json;
use workspaces::{network::Sandbox, prelude::*, Account, AccountId, Worker};

const STATUS_MSG_CONTRACT: &[u8] = include_bytes!("../../examples/res/status_message.wasm");

async fn task(
    worker: Worker<Sandbox>,
    id: AccountId,
    msg: &str,
    account: &Account,
) -> anyhow::Result<()> {
    println!("CALLING IN: {:?} -> {}", std::thread::current().id(), msg);
    account
        .call(&worker, &id, "set_status")
        .args_json(json!({
            "message": msg,
        }))?
        .transact()
        .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_parallel() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(STATUS_MSG_CONTRACT).await?;
    let account = worker.dev_create_account().await?;

    join_all(
        ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]
            .iter()
            .map(|x| {
                let id = contract.id().clone();
                let worker = worker.clone();
                let a = account.clone();
                tokio::spawn(async move { task(worker, id, x, &a).await.unwrap() })
            }),
    )
    .await;

    // Check the final set message. This should be random each time this test function is called:
    let final_set_msg = account
        .call(&worker, contract.id(), "get_status")
        .args_json(json!({ "account_id": account.id() }))?
        .view()
        .await?
        .json::<String>()?;
    println!("Final set message: {:?}", final_set_msg);

    Ok(())
}
