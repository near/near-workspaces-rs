use futures::future::join_all;
use serde_json::json;
use workspaces::{network::Sandbox, prelude::*, Account, AccountId, Worker};

const STATUS_MSG_CONTRACT: &[u8] = include_bytes!("../../examples/res/status_message.wasm");

async fn task(
    worker: Worker<Sandbox>,
    id: AccountId,
    msg: &str,
    add: u64,
    account: &Account,
) -> anyhow::Result<()> {
    println!("CALLING IN: {:?} -> {}", std::thread::current().id(), msg);
    account
        .call(&worker, &id, "set_status")
        .args_json(json!({
            "message": msg,
        }))?
        // .nonce(add)
        .transact()
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_parallel_transactions() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(STATUS_MSG_CONTRACT).await?;
    let account = worker.dev_create_account().await?;

    for x in ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"] {
        let id = contract.id().clone();
        let worker = worker.clone();
        let a = account.clone();
        tokio::spawn(async move { task(worker, id, x, 0, &a).await.unwrap() }).await?;
    }

    let aa = account
        .call(&worker, contract.id(), "get_status")
        .args_json(json!({ "account_id": account.id() }))?
        .view()
        .await?
        .json::<String>()?;
    let bb = account
        .call(&worker, contract.id(), "get_status")
        .args_json(json!({ "account_id": account.id() }))?
        .view()
        .await?
        .json::<String>()?;
    println!("a: {}", aa);
    println!("b: {}", bb);

    Ok(())
}

#[test]
fn test_parallel() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("my-custom-name")
        .thread_stack_size(3 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap();

    let _x: anyhow::Result<()> = rt.block_on(async {
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
                    tokio::spawn(async move { task(worker, id, x, 0, &a).await.unwrap() })
                }),
        )
        .await;

        let aa = account
            .call(&worker, contract.id(), "get_status")
            .args_json(json!({ "account_id": account.id() }))?
            .view()
            .await?
            .json::<String>()?;

        println!("msg: {:?}", aa);

        Ok(())
    });

    Ok(())
}
