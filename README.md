# NEAR Workspaces (Rust Edition)
A set of functions provided to automate workflows and write tests, such as the ability to deploy and run NEAR contracts, along with several other functions to aid in development and maintenance.

This software is in very early alpha (use at your own risk). Only local sandboxed environments and testnet are available currently to run this library on.

## Testing
```rust
#![cfg(test)]

use workspaces::prelude::*;

#[tokio::test]
async fn test_deploy_and_view() {
    let worker = workspaces::sandbox();

    let contract = worker.dev_deploy(std::fs::read("path/to/file.wasm"))
        .await
        .expect("could not dev-deploy contract");

    let result = worker.view(
        contract_id,
        "function_name".to_string(),
        json!({
            "some_arg": "some_value",
        })
        .to_string()
        .into_bytes(),
    )
    .await
    .expect("could not call into view function");

    assert_eq!(result, OUR_EXPECTED_RESULT);
}
```

## Examples
Some examples can be found `examples/src/*.rs` to run it standalone.

To run the NFT example, run:
```
cargo run --example nft
```

## Features

### Choosing a network

```rust
#[tokio::main]  # or whatever runtime we want
async fn main() {
    // Create a sandboxed environment.
    // NOTE: Each call will create a new sandboxed environment
    let worker = workspaces::sandbox();
    // or for testnet:
    let worker = workspaces::testnet();
}
```

### Helper Functions

Need to make a helper function regardless of whatever Network?

```rust
use workspaces::prelude::*;
use workspaces::{Contract, DevNetwork, Network, Worker};

// Helper function that calls into a contract we give it
async fn call_my_func(worker: Worker<impl Network>, contract: &Contract) -> anyhow::Result<Contract> {
    // Call into the function `contract_function` with args:
    worker.call(
        contract,
        "contract_function".into(),
        serde_json::json!({
            "message": msg,
        })
        .to_string()
        .into_bytes(),
        None,
    ).await
}

// Create a helper function that deploys a specific contract
// NOTE: `dev_deploy` is only available on `DevNetwork`s such sandbox and testnet.
async fn deploy_my_contract(worker: Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
    worker.dev_deploy(CONTRACT_FILE).await
}
```
