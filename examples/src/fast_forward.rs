/// Our simple contract. Has a function to called `current_env_data` to just grab
/// the current block_timestamp and epoch_height. Will be used to showcase what
/// our contracts can see pre-and-post fast forwarding.
const SIMPLE_WASM_FILEPATH: &str = "./examples/res/simple_contract.wasm";

/// This example will call into `fast_forward` to show us that our contracts are
/// are being fast forward in regards to the timestamp, block height and epoch height.
/// This saves us the time from having to wait a while for the same amount of blocks
/// to be produced, which could take hours with the default genesis configuration.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker
        .dev_deploy(&std::fs::read(SIMPLE_WASM_FILEPATH)?)
        .await?;

    let (timestamp, epoch_height): (u64, u64) =
        contract.call("current_env_data").view().await?.json()?;
    println!("timestamp = {}, epoch_height = {}", timestamp, epoch_height);

    let block_info = worker.view_latest_block().await?;
    println!("BlockInfo pre-fast_forward {:?}", block_info);

    // Call into fast_forward. This will take a bit of time to invoke, but is
    // faster than manually waiting for the same amounts of blocks to be produced
    worker.fast_forward(10000).await?;

    let (timestamp, epoch_height): (u64, u64) =
        contract.call("current_env_data").view().await?.json()?;
    println!("timestamp = {}, epoch_height = {}", timestamp, epoch_height);

    let block_info = worker.view_latest_block().await?;
    println!("BlockInfo post-fast_forward {:?}", block_info);

    Ok(())
}
