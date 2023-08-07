/// This contract has only one method `noop` which does nothing and returns nothing.
const NOOP_CONTRACT_WASM_FILEPATH: &str = "./examples/res/noop_contract.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(NOOP_CONTRACT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    if let Err(e) = contract.call("noop").transact().await?.json::<()>() {
        println!("ExecutionOutcome from noop: {e:?}");
    }

    Ok(())
}
