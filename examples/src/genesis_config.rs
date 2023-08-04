#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let genesis_config = worker.genesis_config().await?;

    // dump the genesis config info to stdout
    println!("{:?}", genesis_config);
    Ok(())
}
