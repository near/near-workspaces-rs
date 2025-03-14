// This example shows how to use the `near_abi_client` Generation Based API.
// We are generating client code using the schema for the ABI and `workspaces-rs` to call into the contract.
// More information about usage can be found here: <https://github.com/near/near-abi-client-rs/blob/main/README.md>
//
// A good scenario for usage might be when you are interacting with a contract or multiple contracts at an automated level
// and you want to have a type-safe way of interacting with them.

/// The generated api requires setup in the `build.rs` file to generate the client code.
#[path = "gen/adder.rs"]
mod generation_adder;

const ADDER_WASM_FILEPATH: &str = "./examples/res/adder.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(ADDER_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    // The client is initialized with the contract.
    let abi_client = generation_adder::AbiClient { contract };

    // Here we can call the method, now typed with arguments and return types.
    let res = abi_client.add(vec![1, 2], vec![3, 4]).await?;

    assert_eq!(res, [4, 6]);
    Ok(())
}
