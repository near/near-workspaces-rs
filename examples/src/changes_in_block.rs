use serde_json::json;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    _ = contract
        .call("set_status")
        .args_json(json!({
            "message": "hello_world",
        }))
        .transact()
        .await?
        .into_result()?;

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let res = worker.changes_in_block().await?;

    // Example output:
    //
    // StateChangesInBlockByType RpcStateChangesInBlockByTypeResponse {
    //     block_hash: CixdibXkD1ifLmmVXNhEiRGRH6eB9171Q2UhCP2NazJz,
    //     changes: [
    //         AccountTouched {
    //             account_id: AccountId(
    //                 "dev-20230913102437-62490697138398",
    //             ),
    //         },
    //     ],
    // }
    println!("StateChangesInBlockByType {res:#?}");
    Ok(())
}
