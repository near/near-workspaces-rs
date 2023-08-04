#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    // NOTE: this API is under the "experimental" flag.
    let validators = worker.validators_ordered(None).await?;

    println!("Validators {validators:?}");
    Ok(())
}
