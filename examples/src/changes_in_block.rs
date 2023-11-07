use near_primitives::types::BlockReference;
use serde_json::json;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.root_account()?.deploy(&wasm).await?.into_result()?;

    let outcome = contract
        .call("set_status")
        .args_json(json!({
            "message": "hello_world",
        }))
        .transact()
        .await?;

    let block_ref = {
        let hash = near_primitives::hash::CryptoHash(outcome.outcome().block_hash.0);
        BlockReference::BlockId(near_primitives::types::BlockId::Hash(hash))
    };

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let res = worker.changes_in_block(block_ref).await?;

    // Example output:
    //
    // StateChangesInBlockByType RpcStateChangesInBlockByTypeResponse {
    //     block_hash: 7ifRdyBsJMXVyp8zw8uGdBMaRShiXuD6yghrp66jqrst,
    //     changes: [
    //         AccountTouched {
    //             account_id: AccountId(
    //                 "dev-20230822100117-44171728969098",
    //             ),
    //         },
    //         AccessKeyTouched {
    //             account_id: AccountId(
    //                 "dev-20230822100117-44171728969098",
    //             ),
    //         },
    //         DataTouched {
    //             account_id: AccountId(
    //                 "dev-20230822100117-44171728969098",
    //             ),
    //         },
    //     ],
    // }
    println!("StateChangesInBlockByType {res:#?}");
    Ok(())
}
