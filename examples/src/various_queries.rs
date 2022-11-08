#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    // Fetch the latest block produced from the network.
    let block = worker.view_block().await?;
    println!("Block: {block:?}");

    // Fetch the block from the genesis point of the sandbox network. This is not necessarily
    // the genesis of every network since they can re-genesis at a higher block height.
    let genesis_block = worker
        .view_block()
        .block_height(0)
        // can instead use .block_hash(CryptoHash) as well
        .await?;
    println!("Sandbox Geneis Block: {genesis_block:?}");

    // Reference the chunk via the block hash we queried for earlier:
    let shard_id = 0;
    let chunk = worker
        .view_chunk()
        .block_hash_and_shard(*block.hash(), shard_id)
        .await?;
    println!("Chunk: {chunk:?}");

    Ok(())
}
