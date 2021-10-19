# NEAR Workspaces (Rust Edition)
A runtime provided to automate workflows and write tests. This runtime provides the ability to deploy and run NEAR contracts, along with several other functions to aid in development and maintenance.

Write once, run them on a controlled NEAR Sandbox local environment, and on NEAR Testnet and NEAR Mainnet (soon).

This software is in very early alpha (use at your own risk).

## Testing
```rust
#![cfg(test)]

use workspaces::*;

#[workspaces::test(sandbox)]
async fn test_deploy_and_view() {
    let (contract_id, signer) = dev_deploy("path/to/file.wasm")
        .await
        .expect("could not dev-deploy contract");

    let result = view(
        contract_id,
        "function_name".to_string(),
        r#""some_arg": "some_value"".into(),
    ).await.expect("could not call into view function");

    assert_eq!(result, OUR_EXPECTED_RESULT);
}


```

## Examples
Some examples can be found `examples/src/*.rs` to run it standalone.

To run the NFT example, run:
```
cargo run --package examples --example nft
```
