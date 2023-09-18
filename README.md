<div align="center">

  <h1>NEAR Workspaces (Rust Edition)</h1>

  <p>
    <strong>Rust library for automating workflows and writing tests for NEAR smart contracts. This software is not final, and will likely change.</strong>
  </p>

  <p>
    <a href="https://crates.io/crates/workspaces"><img src="https://img.shields.io/crates/v/workspaces.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/workspaces"><img src="https://img.shields.io/crates/d/workspaces.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/workspaces"><img src="https://docs.rs/workspaces/badge.svg" alt="Reference Documentation" /></a>
  </p>
</div>

## Release notes

**Release notes and unreleased changes can be found in the [CHANGELOG](CHANGELOG.md)**

## Requirements

- Rust v1.69.0 and up.
- MacOS (x86 and M1) or Linux (x86) for sandbox tests.

### WASM compilation not supported

`workspaces-rs`, the library itself, does not currently compile to WASM. Best to put this dependency in `[dev-dependencies]` section of `Cargo.toml` if we were trying to run this library alongside something that already does compile to WASM, such as `near-sdk-rs`.

## Simple Testing Case

A simple test to get us going and familiar with `workspaces` framework. Here, we will be going through the NFT contract and how we can test it with `workspaces-rs`.

### Setup -- Imports

First, we need to declare some imports for convenience.

```rust
// macro allowing us to convert human readable units to workspace units.
use near_units::parse_near;

// macro allowing us to convert args into JSON bytes to be read by the contract.
use serde_json::json;
```

We will need to have our pre-compiled WASM contract ahead of time and know its path. Refer to the respective near-sdk-{rs, js} repos/language for where these paths are located.

In this showcase, we will be pointing to the example's NFT contract:

```rust
const NFT_WASM_FILEPATH: &str = "./examples/res/non_fungible_token.wasm";
```

NOTE: there is an unstable feature that will allow us to compile our projects during testing time as well. Take a look at the feature section [Compiling Contracts During Test Time](#compiling-contracts-during-test-time)

### Setup -- Setting up Sandbox and Deploying NFT Contract

This includes launching our sandbox, loading our wasm file and deploying that wasm file to the sandbox environment.

```rust

#[tokio::test]
async fn test_nft_contract() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;
```

Where

- `anyhow` - A crate that deals with error handling, making it more robust for developers.
- `worker` - Our gateway towards interacting with our sandbox environment.
- `contract`- The deployed contract on sandbox the developer interacts with.

### Initialize Contract & Test Output

Then we'll go directly into making a call into the contract, and initialize the NFT contract's metadata:

```rust
    let outcome = contract
        .call("new_default_meta")
        .args_json(json!({
            "owner_id": contract.id(),
        }))
        .transact()  // note: we use the contract's keys here to sign the transaction
        .await?;

    // outcome contains data like logs, receipts and transaction outcomes.
    println!("new_default_meta outcome: {:#?}", outcome);
```

Afterwards, let's mint an NFT via `nft_mint`. This showcases some extra arguments we can supply, such as deposit and gas:

```rust
    let deposit = 10000000000000000000000;
    let outcome = contract
        .call("nft_mint")
        .args_json(json!({
            "token_id": "0",
            "token_owner_id": contract.id(),
            "token_metadata": {
                "title": "Olympus Mons",
                "dscription": "Tallest mountain in charted solar system",
                "copies": 1,
            },
        }))
        .deposit(deposit)
        // nft_mint might consume more than default gas, so supply our own gas value:
        .gas(near_units::parse_gas("300 T"))
        .transact()
        .await?;

    println!("nft_mint outcome: {:#?}", outcome);
```

Then later on, we can view our minted NFT's metadata via our `view` call into `nft_metadata`:

```rust
    let result: serde_json::Value = contract
        .call("nft_metadata")
        .view()
        .await?
        .json()?;

    println!("--------------\n{}", result);
    println!("Dev Account ID: {}", contract.id());
    Ok(())
}
```

### Updating Contract Afterwards

Note that if our contract code changes, `workspaces-rs` does nothing about it since we are utilizing `deploy`/`dev_deploy` to merely send the contract bytes to the network. So if it does change, we will have to recompile the contract as usual, and point `deploy`/`dev_deploy` again to the right WASM files. However, there is a workspaces feature that will recompile contract changes for us: refer to the experimental/unstable [`compile_project`](#compiling-contracts-during-test-time) function for telling workspaces to compile a _Rust_ project for us.

## Examples

More standalone examples can be found in `examples/src/*.rs`.

To run the above NFT example, execute:

```
cargo run --example nft
```

## Features

### Choosing a network

```rust
#[tokio::main]  // or whatever runtime we want
async fn main() -> anyhow::Result<()> {
    // Create a sandboxed environment.
    // NOTE: Each call will create a new sandboxed environment
    let worker = workspaces::sandbox().await?;
    // or for testnet:
    let worker = workspaces::testnet().await?;
}
```

### Helper Functions

Need to make a helper functions utilizing contracts? Just import it and pass it around:

```rust
use workspaces::Contract;

// Helper function that calls into a contract we give it
async fn call_my_func(contract: &Contract) -> anyhow::Result<()> {
    // Call into the function `contract_function` with args:
    contract.call("contract_function")
        .args_json(serde_json::json!({
            "message": msg,
        })
        .transact()
        .await?;
    Ok(())
}
```

Or to pass around workers regardless of networks:

```rust
use workspaces::{DevNetwork, Worker};

const CONTRACT_BYTES: &[u8] = include_bytes!("./relative/path/to/file.wasm");

// Create a helper function that deploys a specific contract
// NOTE: `dev_deploy` is only available on `DevNetwork`s such as sandbox and testnet.
async fn deploy_my_contract(worker: Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
    worker.dev_deploy(CONTRACT_BYTES).await
}
```

### View Account Details

We can check the balance of our accounts like so:

```rs
#[test(tokio::test)]
async fn test_contract_transfer() -> anyhow::Result<()> {
    let transfer_amount = near_units::parse_near!("0.1");
    let worker = workspaces::sandbox().await?;

    let contract = worker
        .dev_deploy(include_bytes!("../target/res/your_project_name.wasm"))
        .await?;
    contract.call("new")
        .max_gas()
        .transact()
        .await?;

    let alice = worker.dev_create_account().await?;
    let bob = worker.dev_create_account().await?;
    let bob_original_balance = bob.view_account().await?.balance;

    alice.call(contract.id(), "function_that_transfers")
        .args_json(json!({ "destination_account": bob.id() }))
        .max_gas()
        .deposit(transfer_amount)
        .transact()
        .await?;
    assert_eq!(
        bob.view_account().await?.balance,
        bob_original_balance + transfer_amount
    );

    Ok(())
}
```

For viewing other chain related details, look at the docs for [Worker](https://docs.rs/workspaces/0.4.1/workspaces/struct.Worker.html), [Account](https://docs.rs/workspaces/0.4.1/workspaces/struct.Account.html) and [Contract](https://docs.rs/workspaces/0.4.1/workspaces/struct.Contract.html)

### Spooning - Pulling Existing State and Contracts from Mainnet/Testnet

This example will showcase spooning state from a testnet contract into our local sandbox environment.

We will first start with the usual imports:

```rust
use near_units::{parse_gas, parse_near};
use workspaces::network::Sandbox;
use workspaces::{Account, AccountId, BlockHeight, Contract, Worker};
```

Then specify the contract name from testnet we want to be pulling:

```rust
const CONTRACT_ACCOUNT: &str = "contract_account_name_on_testnet.testnet";
```

Let's also specify a specific block ID referencing back to a specific time. Just in case our contract or the one we're referencing has been changed or updated:

```rust
const BLOCK_HEIGHT: BlockHeight = 12345;
```

Create a function called `pull_contract` which will pull the contract's `.wasm` file from the chain and deploy it onto our local sandbox. We'll have to re-initialize it with all the data to run tests.

```rust
async fn pull_contract(owner: &Account, worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    let testnet = workspaces::testnet_archival().await?;
    let contract_id: AccountId = CONTRACT_ACCOUNT.parse()?;
```

This next line will actually pull down the relevant contract from testnet and set an initial balance on it with 1000 NEAR.

Following that we will have to init the contract again with our own metadata. This is because the contract's data is to big for the RPC service to pull down, who's limits are set to 50kb.

```rust

    let contract = worker
        .import_contract(&contract_id, &testnet)
        .initial_balance(parse_near!("1000 N"))
        .block_height(BLOCK_HEIGHT)
        .transact()
        .await?;

    owner
        .call(contract.id(), "init_method_name")
        .args_json(serde_json::json!({
            "arg1": value1,
            "arg2": value2,
        }))
        .transact()
        .await?;

    Ok(contract)
}
```

### Time Traveling

`workspaces` testing offers support for forwarding the state of the blockchain to the future. This means contracts which require time sensitive data do not need to sit and wait the same amount of time for blocks on the sandbox to be produced. We can simply just call `worker.fast_forward` to get us further in time.
Note: This is not to be confused with speeding up the current in-flight transactions; the state being forwarded in this case refers to time-related state (the block height, timestamp and epoch).

```rust
#[tokio::test]
async fn test_contract() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(WASM_BYTES).await?;

    let blocks_to_advance = 10000;
    worker.fast_forward(blocks_to_advance).await?;

    // Now, "do_something_with_time" will be in the future and can act on future time-related state.
    contract.call("do_something_with_time")
        .transact()
        .await?;
}
```

For a full example, take a look at [examples/src/fast_forward.rs](https://github.com/near/workspaces-rs/blob/main/examples/src/fast_forward.rs).

### Compiling Contracts During Test Time

Note, this is an unstable feature and will very likely change. To enable it, add the `unstable` feature flag to `workspaces` dependency in `Cargo.toml`:

```toml
[dependencies]
workspaces = { version = "...", features = ["unstable"] }
```

Then, in our tests right before we call into `deploy` or `dev_deploy`, we can compile our projects:

```rust
#[tokio::test]
async fn test_contract() -> anyhow::Result<()> {
    let wasm = workspaces::compile_project("path/to/contract-rs-project").await?;

    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(&wasm).await?;
    ...
}
```

For a full example, take a look at [workspaces/tests/deploy_project.rs](https://github.com/near/workspaces-rs/blob/main/workspaces/tests/deploy_project.rs).

### Other Features

Other features can be directly found in the `examples/` folder, with some documentation outlining how they can be used.

### Environment Variables

These environment variables will be useful if there was ever a snag hit:

- `NEAR_RPC_TIMEOUT_SECS`: The default is 10 seconds, but this is the amount of time before timing out waiting for a RPC service when talking to the sandbox or any other network such as testnet.
- `NEAR_SANDBOX_BIN_PATH`: Set this to our own prebuilt `neard-sandbox` bin path if we want to use a non-default version of the sandbox or configure nearcore with our own custom features that we want to test in workspaces.
<<<<<<< HEAD
- `NEAR_SANDBOX_MAX_PAYLOAD_SIZE`: Sets the max payload size for sending transaction commits to sandbox. The default is 1gb and is necessary for patching large states.
- `NEAR_SANDBOX_MAX_FILES`: Set the max amount of files that can be opened at a time in the sandbox. If none is specified, the default size of 4096 will be used. The actual near chain will use over 10,000 in practice, but for testing this should be much lower since we do not have a constantly running blockchain unless our tests take up that much time.
- `NEAR_RPC_API_KEY`: This is the API key necessary for communicating with RPC nodes. This is useful when interacting with services such as Pagoda Console or a service that can access RPC metrics.
