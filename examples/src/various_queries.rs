use serde_json::json;

/// This example will show various calls into viewing information from what's on the sandbox chain
/// to what was on the chain prior to modifying state.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    // Fetch the latest block produced from the network.
    let block = worker.view_block().await?;
    println!("Latest Block: {block:#?}");

    // Fetch the block from the genesis point of the sandbox network. This is not necessarily
    // the genesis of every network since they can re-genesis at a higher block height.
    let genesis_block = worker
        .view_block()
        .block_height(0)
        // can instead use .block_hash(CryptoHash) as well
        .await?;
    println!("Sandbox Geneis Block: {genesis_block:#?}");

    // Reference the chunk via the block hash we queried for earlier:
    let shard_id = 0;
    let chunk = worker
        .view_chunk()
        .block_hash_and_shard(*block.hash(), shard_id)
        .await?;
    println!("Latest Chunk: {chunk:#?}");

    let bob = worker.dev_create_account().await?;
    println!("\nCreated bob's account with id {:?}", bob.id());

    // Show all the access keys relating to bob:
    let access_keys = bob.view_access_keys().await?;
    println!("bob's access keys: {access_keys:?}");

    let status_msg = worker
        .dev_deploy(include_bytes!("../res/status_message.wasm"))
        .await?;

    // Let's have bob set the "Hello" message into the contract.
    let outcome = bob
        .call(status_msg.id(), "set_status")
        .args_json(json!({
            "message": "Hello"
        }))
        .transact()
        .await?
        .into_result()?;
    println!(
        "Bob burnt {} gas callling into `set_status('Hello')`",
        outcome.total_gas_burnt.as_gas()
    );

    // let's get a reference point to the chain at it's current state, so we can reference it back later
    // when we want older data from the chain.
    let block = worker.view_block().await?;

    // Override bob's message of "Hello" with "World".
    let outcome = bob
        .call(status_msg.id(), "set_status")
        .args_json(json!({
            "message": "World"
        }))
        .transact()
        .await?
        .into_result()?;
    println!(
        "Bob burnt {} gas callling into `set_status('World')`",
        outcome.total_gas_burnt.as_gas()
    );

    // Then view that it indeed has changed:
    let msg: String = status_msg
        .view("get_status")
        .args_json(json!({
            "account_id": bob.id(),
        }))
        .await?
        .json()?;
    println!("Bob's status message: '{}'", msg);

    // But since we have a reference point to before bob overrode his message, we can view the message
    // from then as well, by giving the reference into the view function call:
    let msg: String = status_msg
        .view("get_status")
        .args_json(json!({
            "account_id": bob.id(),
        }))
        .block_hash(*block.hash())
        .await?
        .json()?;
    println!("Bob's older status message: '{}'", msg);

    Ok(())
}
