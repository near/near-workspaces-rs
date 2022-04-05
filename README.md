<div align="center">

  <h1>NEAR Workspaces (Rust Edition)</h1>

  <p>
    <strong>Rust library for automating workflows and writing tests for NEAR smart contracts. This software is in early alpha (use at your own risk)</strong>
  </p>

  <p>
    <a href="https://crates.io/crates/workspaces"><img src="https://img.shields.io/crates/v/workspaces.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/workspaces"><img src="https://img.shields.io/crates/d/workspaces.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/workspaces"><img src="https://docs.rs/workspaces/badge.svg" alt="Reference Documentation" /></a>
  </p>
</div>

## Requirements
- rust v1.56 and up
- MacOS (x86), M1 (through rosetta) or Linux (x86) for sandbox tests. Testnet is available regardless

### M1 MacOS Setup
To be able to use this library on an M1 Mac, we would need to setup rosetta plus our cross compile target:
```
softwareupdate --install-rosetta
rustup default stable-x86_64-apple-darwin
```

## Testing
A simple test to get us going and familiar with the features:

```rust
#![cfg(test)]

use workspaces::prelude::*;

#[tokio::test]
async fn test_deploy_and_view() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    let contract = worker.dev_deploy(include_bytes!("path/to/file.wasm"))
        .await
        .expect("could not dev-deploy contract");

    let result: String = contract.call(&worker, "function_name")
        .args_json(serde_json::json!({
            "some_arg": "some_value",
        }))?
        .view()
        .await?
        .json()?;

    assert_eq!(result, "OUR_EXPECTED_RESULT");
    Ok(())
}
```

## Examples
Some examples can be found in `examples/src/*.rs` to run it standalone.

To run the NFT example, execute:
```
cargo run --example nft
```

## Features

### Choosing a network

```rust
#[tokio::main]  # or whatever runtime we want
async fn main() -> anyhow::Result<()> {
    // Create a sandboxed environment.
    // NOTE: Each call will create a new sandboxed environment
    let worker = workspaces::sandbox().await?;
    // or for testnet:
    let worker = workspaces::testnet().await?;
}
```

### Helper Functions

Need to make a helper function regardless of whatever Network?

```rust
use workspaces::prelude::*;
use workspaces::{Contract, DevNetwork, Network, Worker};

// Helper function that calls into a contract we give it
async fn call_my_func(worker: Worker<impl Network>, contract: &Contract) -> anyhow::Result<()> {
    // Call into the function `contract_function` with args:
    contract.call(&worker, "contract_function")
        .args_json(serde_json::json!({
            "message": msg,
        })?
        .transact()
        .await?;
    Ok(())
}

// Create a helper function that deploys a specific contract
// NOTE: `dev_deploy` is only available on `DevNetwork`s such sandbox and testnet.
async fn deploy_my_contract(worker: Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
    worker.dev_deploy(&std::fs::read(CONTRACT_FILE)?).await
}
```
