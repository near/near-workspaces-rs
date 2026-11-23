<div align="center">

  <h1>NEAR Workspaces (Rust Edition)</h1>

  <p>
    <strong>Deprecated. Test NEAR smart contracts with <a href="https://github.com/r-near/near-kit-rs">near-kit</a> and <a href="https://github.com/near/near-sandbox-rs">near-sandbox</a> instead.</strong>
  </p>

  <p>
    <a href="https://crates.io/crates/near-workspaces"><img src="https://img.shields.io/crates/v/near-workspaces.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/near-workspaces"><img src="https://img.shields.io/crates/d/near-workspaces.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/near-workspaces"><img src="https://docs.rs/near-workspaces/badge.svg" alt="Reference Documentation" /></a>
  </p>
</div>

> [!WARNING]
> **near-workspaces is deprecated, and this repository will be archived.** It gets no new features, fixes or support for new nearcore releases.
> Write new tests with [near-kit](https://github.com/r-near/near-kit-rs) and [near-sandbox](https://github.com/near/near-sandbox-rs), and move existing ones over with the [migration guide](#migrating-to-near-kit-and-near-sandbox) below.
> Published versions stay on crates.io, so projects that depend on them keep building.

## Migrating to near-kit and near-sandbox

near-workspaces did two jobs: it ran a local NEAR node, and it was the client that talked to the node. Two crates do these jobs now:

- [near-sandbox](https://github.com/near/near-sandbox-rs) starts the node. It downloads the sandbox binary, runs it, and can patch the node's state or fast-forward it.
- [near-kit](https://github.com/r-near/near-kit-rs) is the client. It signs and sends transactions, calls view functions and reads the chain, on the sandbox as on testnet or mainnet.

A third crate, [cargo-near-build](https://crates.io/crates/cargo-near-build), builds contracts from tests, as `compile_project` did.

What changes the most:

- **No dev helpers.** `dev_deploy`, `dev_create_account` and `max_gas` are gone. Creating an account and deploying a contract to it is an ordinary transaction, so you choose the account name, its balance and its keys.
- **No `Account` and `Contract` types.** An account you sign with is a `SigningNear`: a client with a signer. A contract is an `AccountId` you send transactions to.
- **The `Sandbox` value owns the node.** The node stops when the value is dropped, so keep it until the test ends.
- **Gas is explicit.** A function call attaches 30 Tgas unless you set `.gas(..)`. near-workspaces attached 10 Tgas.

The code in this guide is tested with near-kit 0.33.1, near-sandbox 0.4.1 and cargo-near-build 0.11.5 on nearcore 2.14.1. near-kit is not 1.0 yet, so a minor release can change its API; its [upgrade notes](https://github.com/r-near/near-kit-rs/blob/main/MIGRATION.md) list every change.

### Contents

- [Dependencies](#dependencies)
- [A complete test](#a-complete-test)
- [Starting a sandbox](#starting-a-sandbox)
- [Accounts](#accounts)
- [Deploying contracts](#deploying-contracts)
- [Building contracts in tests](#building-contracts-in-tests)
- [Calling contracts](#calling-contracts)
- [View functions](#view-functions)
- [Transaction results](#transaction-results)
- [Batch transactions](#batch-transactions)
- [Async transactions](#async-transactions)
- [Account details and other queries](#account-details-and-other-queries)
- [Time travel](#time-travel)
- [Patching state](#patching-state)
- [Importing accounts from testnet and mainnet](#importing-accounts-from-testnet-and-mainnet)
- [Testnet, mainnet and custom networks](#testnet-mainnet-and-custom-networks)
- [Helper functions](#helper-functions)
- [Typed contract clients](#typed-contract-clients)
- [Sandbox configuration and environment variables](#sandbox-configuration-and-environment-variables)
- [Quick reference](#quick-reference)

### Dependencies

Before:

```toml
[dev-dependencies]
near-workspaces = { version = "0.24", features = ["unstable"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
serde_json = "1"
```

After:

```toml
[dev-dependencies]
near-kit = "0.33"
near-sandbox = "0.4"
# Only to build contracts from tests (`compile_project` with the `unstable` feature)
cargo-near-build = "0.11"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
serde_json = "1"
```

- near-kit needs Rust 1.95 or newer.
- near-sandbox downloads the sandbox binary the first time a test starts a node and keeps it in its build directory under `target/`, so `cargo clean` or a new build of near-sandbox downloads it again. With the `global_install` feature it keeps the binary in `~/.near` instead.
- `near_kit::AccountId`, `NearToken` and `Gas` are the types from `near-account-id`, `near-token` and `near-gas`, the same ones near-sdk and near-sandbox use, and `near_kit::json_types` holds near-sdk's `U128`, `U64` and `Base64VecU8`. With a recent near-sdk, values pass between them with no conversion.

### A complete test

The NFT test from the earlier version of this README, before and after. The sections below explain each step.

Before:

```rust
use near_workspaces::types::{Gas, NearToken};
use serde_json::json;

const NFT_WASM_FILEPATH: &str = "./examples/res/non_fungible_token.wasm";

#[tokio::test]
async fn test_nft_contract() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let tx_details = contract
        .call("new_default_meta")
        .args_json(json!({
            "owner_id": contract.id(),
        }))
        .transact() // signed with the contract account's key
        .await?;

    // tx_details contains data like logs, receipts and transaction outcomes.
    println!("new_default_meta tx_details: {tx_details:#?}");

    let tx_details = contract
        .call("nft_mint")
        .args_json(json!({
            "token_id": "0",
            "token_owner_id": contract.id(),
            "token_metadata": {
                "title": "Olympus Mons",
                "description": "Tallest mountain in charted solar system",
                "copies": 1,
            },
        }))
        .deposit(NearToken::from_millinear(10))
        .gas(Gas::from_tgas(100))
        .transact()
        .await?;

    println!("nft_mint tx_details: {tx_details:#?}");

    let result: serde_json::Value = contract.view("nft_metadata").await?.json()?;

    println!("--------------\n{result}");
    println!("Dev Account ID: {}", contract.id());

    Ok(())
}
```

After:

```rust
use near_kit::protocol::Action;
use near_kit::signer::{InMemorySigner, SecretKey};
use near_kit::transaction::FunctionCall;
use near_kit::{Gas, Near, NearToken};
use near_sandbox::Sandbox;
use near_sandbox::config::{DEFAULT_GENESIS_ACCOUNT, DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY};
use serde_json::json;

const NFT_WASM_FILEPATH: &str = "./examples/res/non_fungible_token.wasm";

#[tokio::test]
async fn test_nft_contract() -> anyhow::Result<()> {
    // The node stops when this value is dropped, so keep it until the test ends
    let node = Sandbox::start_sandbox().await?;
    let near = Near::custom(&node.rpc_addr, "localnet").build();

    // The genesis account `sandbox` pays for the setup, as the root account did
    let sandbox = near.with_signer(InMemorySigner::new(
        DEFAULT_GENESIS_ACCOUNT,
        DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?,
    ));

    // What `dev_deploy` did: create an account with its own key, fund it and deploy the code
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let nft_account_id = sandbox.account_id().sub_account("nft")?;
    let nft_key = SecretKey::generate_ed25519();

    sandbox
        .transaction(&nft_account_id)
        .add_action(Action::create_account())
        .add_action(Action::transfer(NearToken::from_near(100)))
        .add_action(Action::add_full_access_key(nft_key.public_key()))
        .add_action(Action::deploy_contract(wasm))
        .await?
        .result()?;

    // Signs as `nft.sandbox`, like the `Contract` that `dev_deploy` returned
    let nft = near.with_signer(InMemorySigner::new(&nft_account_id, nft_key));

    let tx_details = nft
        .transaction(&nft_account_id)
        .add_action(FunctionCall::new("new_default_meta").args(json!({
            "owner_id": nft_account_id,
        })))
        .await?;

    // tx_details contains data like logs, receipts and transaction outcomes.
    println!("new_default_meta tx_details: {tx_details:#?}");

    let tx_details = nft
        .transaction(&nft_account_id)
        .add_action(
            FunctionCall::new("nft_mint")
                .args(json!({
                    "token_id": "0",
                    "token_owner_id": nft_account_id,
                    "token_metadata": {
                        "title": "Olympus Mons",
                        "description": "Tallest mountain in charted solar system",
                        "copies": 1,
                    },
                }))
                .deposit(NearToken::from_millinear(10))
                .gas(Gas::from_tgas(100)),
        )
        .await?;

    println!("nft_mint tx_details: {tx_details:#?}");

    let result: serde_json::Value = near.view(&nft_account_id, "nft_metadata").await?;

    println!("--------------\n{result}");
    println!("Contract account ID: {nft_account_id}");

    Ok(())
}
```

The setup is longer because creating the contract account is now spelled out. A helper keeps each test short: see [Helper functions](#helper-functions).

### Starting a sandbox

Before:

```rust
let worker = near_workspaces::sandbox().await?;
// Or a specific nearcore version
let worker = near_workspaces::sandbox_with_version("2.14.1").await?;
```

After:

```rust
use near_kit::Near;
use near_sandbox::Sandbox;

let node = Sandbox::start_sandbox().await?;
// Or a specific nearcore version
let node = Sandbox::start_sandbox_with_version("2.14.1").await?;

// The client: views, queries and, with a signer, transactions
let near = Near::custom(&node.rpc_addr, "localnet").build();
```

- The `Sandbox` owns the node process and its data directory. Dropping it stops the node and deletes the data. `let _node = ...` keeps it until the end of the scope, but `let _ = ...` drops it at once. A setup helper should return it along with the clients.
- The second argument of `Near::custom` is a name for the network, which `near.chain_id()` returns. near-kit doesn't check it against the node, whose own chain ID is random (`test-chain-...`), so this guide calls the network `localnet`.
- Each `Sandbox` is a separate node on its own ports, so tests run in parallel as before, with `cargo test` or `cargo nextest`. To share one node among the tests of a test binary, see near-sandbox's [`singleton_sandbox` example](https://github.com/near/near-sandbox-rs/blob/main/examples/singleton_sandbox.rs).

The rest of this guide assumes a `node` and a `near` client created this way.

### Accounts

#### The `sandbox` account

near-sandbox adds an account named `sandbox` to genesis, with 10,000 NEAR and a key that near-sandbox exports as a constant. It pays for the setup, as the root account did. In near-workspaces the root account was the validator, `test.near`, and dev accounts were named `dev-<time>-<number>.test.near`. Here they become sub-accounts of `sandbox`, such as `alice.sandbox`.

Before:

```rust
let root = worker.root_account()?;
```

After:

```rust
use near_kit::signer::InMemorySigner;
use near_sandbox::config::{DEFAULT_GENESIS_ACCOUNT, DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY};

let sandbox = near.with_signer(InMemorySigner::new(
    DEFAULT_GENESIS_ACCOUNT,
    DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?,
));
```

#### Dev accounts and sub-accounts

Before:

```rust
let alice = worker.dev_create_account().await?;

let bob = root
    .create_subaccount("bob")
    .initial_balance(NearToken::from_near(10))
    .transact()
    .await?
    .into_result()?;
```

After: a transaction from the parent account creates the account, funds it and adds a key.

```rust
use near_kit::NearToken;
use near_kit::protocol::Action;
use near_kit::signer::{InMemorySigner, SecretKey};

let alice_account_id = sandbox.account_id().sub_account("alice")?; // alice.sandbox
let alice_key = SecretKey::generate_ed25519();

sandbox
    .transaction(&alice_account_id)
    .add_action(Action::create_account())
    .add_action(Action::transfer(NearToken::from_near(100)))
    .add_action(Action::add_full_access_key(alice_key.public_key()))
    .await?
    .result()?;

let alice = sandbox.with_signer(InMemorySigner::new(alice_account_id, alice_key));
```

`dev_create_account` picked a random `dev-...` name and gave the account 100 NEAR. Here you choose both. Names only need to be unique within one sandbox.

#### Accounts in genesis

Accounts listed in the sandbox config exist when the node starts, so a test needs no transaction for them. near-workspaces had no equivalent.

```rust
use near_kit::account;
use near_sandbox::{GenesisAccount, SandboxConfig};

let alice_account = GenesisAccount::default_with_name(account!("alice"));

let node = Sandbox::start_sandbox_with_config(SandboxConfig {
    additional_accounts: vec![alice_account.clone()],
    ..Default::default()
})
.await?;
let near = Near::custom(&node.rpc_addr, "localnet").build();

let alice = near.with_signer(InMemorySigner::new(
    alice_account.account_id,
    alice_account.private_key.parse()?,
));
```

`default_with_name` gives the account 10,000 NEAR and the `sandbox` account's key. For other values, fill in `GenesisAccount { account_id, public_key, private_key, balance }` yourself.

#### Top-level accounts

A transaction can only create sub-accounts of its signer. For any other name, use a state patch.

Before:

```rust
let carol = worker
    .create_tla("carol".parse()?, secret_key)
    .await?
    .into_result()?;
```

After:

```rust
let carol_key = SecretKey::generate_ed25519();

node.create_account(account!("carol"))
    .initial_balance(NearToken::from_near(100))
    .public_key(carol_key.public_key().to_string())
    .send()
    .await?;

let carol = near.with_signer(InMemorySigner::new(account!("carol"), carol_key));
```

Without `.public_key(..)`, the account gets the `sandbox` account's key.

#### Signing as another account

There is no `Account`/`Contract` split and no `as_account()`. To sign as another account, give a client that account's key: `near.with_signer(InMemorySigner::new(account_id, secret_key))`, or `alice.with_signer(..)` from an existing client. The clients share one connection, so make as many as you need. Keep the `SecretKey` if a test needs it later, as you would have used `account.secret_key()`.

### Deploying contracts

Before:

```rust
let wasm = std::fs::read("res/contract.wasm")?;
let contract = worker.dev_deploy(&wasm).await?;
```

After:

```rust
use near_kit::transaction::FunctionCall;
use serde_json::json;

let wasm = std::fs::read("res/contract.wasm")?;
let contract_id = sandbox.account_id().sub_account("contract")?;

sandbox
    .transaction(&contract_id)
    .add_action(Action::create_account())
    .add_action(Action::transfer(NearToken::from_near(10)))
    .add_action(Action::deploy_contract(wasm))
    // Optional: initialize the contract in the same transaction
    .add_action(FunctionCall::new("new").args(json!({ "owner_id": sandbox.account_id() })))
    .await?
    .result()?;
```

- The account pays for storing the code and the contract's state (1 NEAR per 100 kB), so fund it accordingly. `dev_deploy` gave it 100 NEAR.
- The contract account above has no key, so nobody can sign as it. If a test needs to, for example to call a `#[private]` method, add `Action::add_full_access_key(key.public_key())` as in [the complete test](#a-complete-test).
- To deploy to an existing account (`account.deploy(&wasm)`), send the action to the signer's own account: `alice.transaction(alice.account_id()).add_action(Action::deploy_contract(wasm))`.
- near-kit can also publish and use global contracts: `Action::publish(wasm, mode)` and `Action::deploy_from(..)`.

### Building contracts in tests

Before:

```rust
// near-workspaces = { version = "0.24", features = ["unstable"] }
let wasm = near_workspaces::compile_project("./path/to/contract").await?;
```

After: `compile_project` was a thin wrapper around cargo-near-build. Call it yourself:

```rust
/// Builds the contract in `manifest_dir` with `cargo near` and returns the wasm.
async fn build_contract(manifest_dir: &str) -> anyhow::Result<Vec<u8>> {
    let manifest_path =
        cargo_near_build::camino::Utf8PathBuf::from(manifest_dir).join("Cargo.toml");

    // The build blocks, so run it off the async runtime
    tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<u8>> {
        let artifact = cargo_near_build::build_with_cli(cargo_near_build::BuildOpts {
            no_locked: true,
            manifest_path: Some(manifest_path),
            // As `compile_project` did: without it, a contract on near-sdk 5.27 or older
            // fails to build with rustc newer than 1.86, though the sandbox runs that wasm
            skip_rust_version_check: true,
            ..Default::default()
        })
        .map_err(|e| anyhow::anyhow!("cargo near build: {e:?}"))?;

        Ok(std::fs::read(&artifact)?)
    })
    .await?
}

// The contract in the crate the test belongs to
let wasm = build_contract(env!("CARGO_MANIFEST_DIR")).await?;
```

Each call runs `cargo build`. When many tests need the same contract, build it once per test binary:

```rust
use tokio::sync::OnceCell;

static CONTRACT_WASM: OnceCell<Vec<u8>> = OnceCell::const_new();

async fn contract_wasm() -> anyhow::Result<&'static [u8]> {
    let wasm = CONTRACT_WASM
        .get_or_try_init(|| build_contract(env!("CARGO_MANIFEST_DIR")))
        .await?;

    Ok(wasm)
}
```

### Calling contracts

Before:

```rust
let tx_details = alice
    .call(ft.id(), "storage_deposit")
    .args_json(json!({ "account_id": bob.id() }))
    .deposit(NearToken::from_millinear(125))
    .gas(Gas::from_tgas(10))
    .transact()
    .await?;
```

After:

```rust
use near_kit::transaction::FunctionCall;
use near_kit::{Gas, NearToken};

let tx_details = alice
    .transaction(&ft_account_id)
    .add_action(
        FunctionCall::new("storage_deposit")
            .args(json!({ "account_id": bob.account_id() }))
            .deposit(NearToken::from_millinear(125))
            .gas(Gas::from_tgas(10)),
    )
    .await?;
```

| near-workspaces | near-kit |
|---|---|
| `account.call(&contract_id, "method")` | `account.transaction(&contract_id).add_action(FunctionCall::new("method"))` |
| `contract.call("method")`, signed by the contract | `contract.transaction(contract.account_id()).add_action(..)` |
| `.args_json(args)`, `.args_borsh(args)`, `.args(bytes)` | `.args(args)`, `.args_borsh(args)`, `.args_raw(bytes)` |
| `.deposit(amount)`, `.gas(gas)` | the same |
| `.max_gas()` | `.gas(Gas::from_tgas(1_000))` |
| `.transact().await?` | `.await?` |

Awaiting the transaction signs it, sends it and waits until it is final. A failed call still returns `Ok`, as `transact()` did: see [Transaction results](#transaction-results).

### View functions

Before:

```rust
let status: Option<String> = contract
    .view("get_status")
    .args_json(json!({ "account_id": alice.id() }))
    .await?
    .json()?;
```

After:

```rust
let status: Option<String> = near
    .view(&contract_id, "get_status")
    .args(json!({ "account_id": alice.account_id() }))
    .await?;
```

- The variable's type decides how the JSON result is decoded. From a signing client, call `alice.near().view(..)`.
- `.at(height)` reads at a block, like `.block_height(height)` did.
- For a Borsh result, build the view with `near_kit::View::borsh(near.clone(), &contract_id, "method")`. For the raw bytes and logs, end with `.fetch_raw().await?`.

### Transaction results

A transaction returns a `FinalExecutionOutcome` in place of `ExecutionFinalResult`. As before, a failed action still comes back as `Ok`, so check it.

Before:

```rust
let tx_details = contract
    .call("set_status")
    .args_json(json!({ "message": "hello" }))
    .transact()
    .await?;

assert!(tx_details.is_success());
let logs: Vec<&str> = tx_details.logs();
println!("gas burnt: {}", tx_details.total_gas_burnt);
tx_details.into_result()?;
```

After:

```rust
let tx_details = alice
    .transaction(&contract_id)
    .add_action(FunctionCall::new("set_status").args(json!({ "message": "hello" })))
    .await?;

assert!(tx_details.is_success());
let logs: Vec<&str> = tx_details.logs().collect();
println!("gas used: {}", tx_details.total_gas_used());
tx_details.result()?;
```

| near-workspaces | near-kit |
|---|---|
| `into_result()?` | `result()?`, which returns the return value's bytes, or `checked()?`, which returns the transaction details |
| `json::<T>()?` | the same |
| `borsh::<T>()?`, `raw_bytes()?` | `borsh::from_slice::<T>(&tx_details.result()?)?`, `result()?` |
| `is_success()`, `is_failure()` | the same |
| `logs()` | `logs()`, an iterator |
| `outcome()`, `receipt_outcomes()` | the `transaction_outcome` and `receipts_outcome` fields |
| `receipt_failures()` | `receipt_failures()`, the error of each failed receipt |
| `total_gas_burnt` | `total_gas_used()` |

To check why a call failed, read the error text or match the failed receipts:

```rust
use near_kit::protocol::{ActionErrorKind, FunctionCallError};

assert!(tx_details.is_failure());
assert!(tx_details.failure_message().is_some_and(|message| {
    message.contains("Requires attached deposit of exactly 1 yoctoNEAR")
}));

let panics: Vec<&str> = tx_details
    .receipt_failures()
    .filter_map(|failure| match &failure.kind {
        ActionErrorKind::FunctionCallError(FunctionCallError::ExecutionError(message)) => {
            Some(message.as_str())
        }
        _ => None,
    })
    .collect();

assert_eq!(
    panics,
    ["Smart contract panicked: Requires attached deposit of exactly 1 yoctoNEAR"]
);
```

`result()` and `checked()` read the transaction's final status, as `into_result()` did. A cross-contract call can fail in a later receipt while the transaction succeeds, for example in the receiver of `ft_transfer_call`, so check `receipt_failures()` for those.

near-kit also decodes NEP-297 events: `tx_details.events(&contract_id)`. See [Checking NEP-297 events](https://docs.rs/near-kit/latest/near_kit/guide/testing/index.html#checking-nep-297-events) in its guide.

### Batch transactions

Before:

```rust
use near_workspaces::operations::Function;

alice
    .batch(contract.id())
    .call(Function::new("set_status").args_json(json!({ "message": "first" })))
    .call(Function::new("set_status").args_json(json!({ "message": "second" })))
    .transfer(NearToken::from_near(1))
    .transact()
    .await?
    .into_result()?;
```

After: every transaction takes a list of actions.

```rust
alice
    .transaction(&contract_id)
    .add_action(FunctionCall::new("set_status").args(json!({ "message": "first" })))
    .add_action(FunctionCall::new("set_status").args(json!({ "message": "second" })))
    .add_action(Action::transfer(NearToken::from_near(1)))
    .await?
    .result()?;
```

Each batch method has an `Action` constructor: `create_account()`, `transfer(..)`, `deploy_contract(..)` (was `deploy`), `add_full_access_key(..)` and `add_function_call_key(..)` (was `add_key`), `delete_key(..)`, `delete_account(..)` and `stake(..)`.

### Async transactions

Before:

```rust
let status = contract
    .call("set_status")
    .args_json(json!({ "message": "hello" }))
    .transact_async()
    .await?;

// Check without waiting
let poll = status.status().await?;
// Wait for the transaction to finish
let tx_details = status.await?;
```

After: choose how long the send waits, then look the transaction up by its hash.

```rust
use near_kit::transaction::{Final, Included};

// Returns once the transaction is in a block
let sent = alice
    .transaction(&contract_id)
    .add_action(FunctionCall::new("set_status").args(json!({ "message": "hello" })))
    .wait_until::<Included>()
    .await?;

// Check without waiting: `status.outcome` holds what has run so far, if anything
let status = near
    .tx_status(&sent.transaction_hash, alice.account_id().clone())
    .await?;

// Wait for the transaction to finish
let tx_details = near
    .tx_status(&sent.transaction_hash, alice.account_id().clone())
    .wait_until::<Final>()
    .await?;
```

The node answers each request within its RPC timeout. A transaction that takes longer, such as a call that yields and waits to be resumed, makes the `Final` wait fail with `RpcError::Timeout`, so ask again until it returns:

```rust
use near_kit::rpc::RpcError;

let tx_details = loop {
    match near
        .tx_status(&sent.transaction_hash, alice.account_id().clone())
        .wait_until::<Final>()
        .await
    {
        Err(error) if matches!(error.as_rpc(), Some(RpcError::Timeout { .. })) => continue,
        other => break other?,
    }
};
```

### Account details and other queries

Before:

```rust
let bob_balance = bob.view_account().await?.balance;

alice
    .transfer_near(bob.id(), NearToken::from_near(1))
    .await?
    .into_result()?;

assert_eq!(
    bob.view_account().await?.balance,
    bob_balance.saturating_add(NearToken::from_near(1))
);
```

After:

```rust
let bob_balance = near.balance(bob.account_id()).await?.total;

alice
    .transaction(bob.account_id())
    .add_action(Action::transfer(NearToken::from_near(1)))
    .await?
    .result()?;

assert_eq!(
    near.balance(bob.account_id()).await?.total,
    bob_balance.saturating_add(NearToken::from_near(1))
);
```

`near.balance(..)` also returns the `available` and `locked` amounts and the storage cost. The other queries are on `near.rpc()` and read at the block you pass:

```rust
use near_kit::rpc::Finality;

let rpc = near.rpc();

// account.view_account()
let account = rpc
    .view_account(alice.account_id(), Finality::Final)
    .await?;

// worker.view_block()
let height = rpc.block(Finality::Final).await?.header.height;

// contract.view_state(): every key and value, in `state.values`
let state = rpc
    .view_state_all(&contract_id, b"", 0, Finality::Final)
    .await?;

// contract.view_code()
let code = rpc.view_code(&contract_id, Finality::Final).await?.code;

// account.view_access_keys()
let keys = rpc
    .view_access_key_list(alice.account_id(), Finality::Final)
    .await?
    .keys;

// worker.status()
let status = rpc.status().await?;
```

`worker.tx_status(..)` is `near.tx_status(..)` and `worker.receipt(..)` is `near.rpc().receipt(..)`. For RPC methods without a typed call, such as the `experimental` feature's `genesis_config` and `protocol_config`, send the request yourself:

```rust
let genesis: serde_json::Value = near
    .rpc()
    .request("EXPERIMENTAL_genesis_config", json!({}))
    .await?;
```

### Time travel

Before:

```rust
worker.fast_forward(10_000).await?;
```

After:

```rust
node.fast_forward(10_000).await?;
```

`fast_forward` returns once the node has produced the blocks. Each epoch the jump crosses takes about half a second, and a sandbox epoch is 60 blocks long, so 10,000 blocks take over a minute. Unless the test depends on epoch changes, make epochs longer than the jump:

```rust
use near_sandbox::SandboxConfig;

let node = Sandbox::start_sandbox_with_config(SandboxConfig {
    additional_genesis: Some(json!({ "epoch_length": 20_000 })),
    ..Default::default()
})
.await?;

// Now well under a second
node.fast_forward(10_000).await?;
```

### Patching state

Before:

```rust
worker
    .patch(contract.id())
    .account(AccountDetailsPatch::default().balance(NearToken::from_near(1_000_000)))
    .state(b"STATE", &borsh::to_vec(&state)?)
    .transact()
    .await?;
```

After:

```rust
use base64::prelude::{BASE64_STANDARD, Engine as _};
use near_sandbox::FetchData;

node.patch_state(contract_id.clone())
    // Start from the account as it is, to change its balance
    .fetch_from(&node.rpc_addr, FetchData::NONE.account())
    .await?
    .initial_balance(NearToken::from_near(1_000_000))
    .storage(
        BASE64_STANDARD.encode(b"STATE"),
        BASE64_STANDARD.encode(borsh::to_vec(&state)?),
    )
    .send()
    .await?;
```

- near-sandbox takes keys, values and code as base64 strings, so add the `base64` crate to your dev-dependencies.
- `worker.patch_state(id, key, value)` is the same builder with only `.storage(key, value)`.
- `.code(..)`, `.access_key(..)` and `.storage_entries(..)` replace the other `patch` methods.
- In near-sandbox 0.4.1, `initial_balance` needs the account's current record, which `fetch_from` reads from the sandbox. Without it the node rejects the patch.

### Importing accounts from testnet and mainnet

Before:

```rust
let testnet = near_workspaces::testnet_archival().await?;

let contract = worker
    .import_contract(&"hello.near-examples.testnet".parse()?, &testnet)
    .initial_balance(NearToken::from_near(1000))
    .block_height(BLOCK_HEIGHT)
    .with_data()
    .transact()
    .await?;
```

After:

```rust
use near_kit::{Near, account};

let testnet = Near::testnet().build();
let hello_account_id = account!("hello.near-examples.testnet");

node.import_account(testnet.rpc().url(), hello_account_id.clone())
    .initial_balance(NearToken::from_near(1000))
    .with_storage()
    .send()
    .await?;
```

- The import copies the account as it is now. There is no `block_height`.
- The imported account gets the `sandbox` account's key, so a test can sign as it with `DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY`. `.public_key(..)` sets a different key and `.with_access_keys()` copies the account's own keys.
- RPC nodes don't return contract state over 50 kB. Import such a contract without `.with_storage()` and initialize it again in the test.

To copy a single value instead, read it with `testnet.rpc().view_state_all(..)` and write it with `node.patch_state(..)`.

### Testnet, mainnet and custom networks

Before:

```rust
let testnet = near_workspaces::testnet().await?;
let mainnet = near_workspaces::mainnet().await?;

let greeting: String = testnet
    .view(&"hello.near-examples.testnet".parse()?, "get_greeting")
    .await?
    .json()?;
```

After:

```rust
use near_kit::{Near, account};

let testnet = Near::testnet().build();
let mainnet = Near::mainnet().build();
let custom = Near::custom("https://rpc.example.com", "mainnet").build();

let greeting: String = testnet
    .view(account!("hello.near-examples.testnet"), "get_greeting")
    .await?;
```

- `Near::testnet()` and `Near::mainnet()` use FastNEAR's free, rate-limited RPC endpoints. For your own endpoint, or an archival one, use `Near::custom(url, chain_id)`. For an API key (`NEAR_RPC_API_KEY` in near-workspaces), pass `near_kit::rpc::RpcEndpoint::new(url).bearer(key)` as the URL.
- near-kit doesn't create testnet dev accounts. To send transactions, create an account (with [near-cli-rs](https://github.com/near/near-cli-rs), for example) and sign with its key: `testnet.with_signer(InMemorySigner::new(account_id, secret_key))`. `near_kit::SigningNear::from_env()` reads them from `NEAR_ACCOUNT_ID` and `NEAR_PRIVATE_KEY`.

### Helper functions

Before:

```rust
use near_workspaces::{Contract, DevNetwork, Worker};

async fn call_my_func(contract: &Contract, msg: &str) -> anyhow::Result<()> {
    contract
        .call("contract_function")
        .args_json(json!({ "message": msg }))
        .transact()
        .await?
        .into_result()?;

    Ok(())
}

async fn deploy_my_contract(worker: Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
    Ok(worker.dev_deploy(CONTRACT_BYTES).await?)
}
```

After: helpers take a `SigningNear` to send and an `AccountId` for the contract. A `SigningNear` works on any network, as `Worker<impl DevNetwork>` did. A helper that only reads takes a `&Near`, which `alice.near()` returns.

```rust
use near_kit::{AccountId, SigningNear};

async fn call_my_func(
    signer: &SigningNear,
    contract_id: &AccountId,
    msg: &str,
) -> anyhow::Result<()> {
    signer
        .transaction(contract_id)
        .add_action(FunctionCall::new("contract_function").args(json!({ "message": msg })))
        .await?
        .result()?;

    Ok(())
}

/// Deploys `CONTRACT_BYTES` to a new sub-account `name` of `parent`.
async fn deploy_my_contract(parent: &SigningNear, name: &str) -> anyhow::Result<AccountId> {
    let contract_id = parent.account_id().sub_account(name)?;

    parent
        .transaction(&contract_id)
        .add_action(Action::create_account())
        .add_action(Action::transfer(NearToken::from_near(10)))
        .add_action(Action::deploy_contract(CONTRACT_BYTES))
        .await?
        .result()?;

    Ok(contract_id)
}
```

A setup helper returns the `Sandbox` along with the clients, and the test keeps it:

```rust
/// Starts a sandbox node and returns a client that signs as the `sandbox` account.
async fn setup() -> anyhow::Result<(Sandbox, SigningNear)> {
    let node = Sandbox::start_sandbox().await?;
    let sandbox = Near::custom(&node.rpc_addr, "localnet")
        .build()
        .with_signer(InMemorySigner::new(
            DEFAULT_GENESIS_ACCOUNT,
            DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?,
        ));

    Ok((node, sandbox))
}

#[tokio::test]
async fn test_my_contract() -> anyhow::Result<()> {
    // `_node`, not `_`: the node must run until the test ends
    let (_node, sandbox) = setup().await?;
    let contract_id = deploy_my_contract(&sandbox, "contract").await?;

    call_my_func(&sandbox, &contract_id, "hello").await?;

    Ok(())
}
```

### Typed contract clients

near-workspaces re-exported [near-abi-client](https://github.com/near/near-abi-client-rs), which generates a client from a contract's ABI file. near-kit generates one from a trait that lists the contract's methods. It needs the `contracts` feature.

Before:

```rust
near_workspaces::near_abi_client::generate!(AdderClient for "res/adder.json");

let contract = worker.dev_deploy(&wasm).await?;
let adder = AdderClient { contract };
let sum = adder.add(vec![1, 2], vec![3, 4]).await?;
```

After:

```rust
// near-kit = { version = "0.33", features = ["contracts"] }
#[near_kit::contract]
pub trait Adder {
    fn add(&self, a: (u32, u32), b: (u32, u32)) -> (u32, u32);
}

let adder = near.contract::<Adder>(&contract_id);
let sum = adder.add((1, 2), (3, 4)).await?;
```

A method that changes state takes `&mut self` and is marked `#[call]`. Calling it needs a client built from a `SigningNear`:

```rust
use near_kit::AccountId;

#[near_kit::contract]
pub trait StatusMessage {
    fn get_status(&self, account_id: &AccountId) -> Option<String>;

    #[call]
    fn set_status(&mut self, message: &str);
}

let status_message = alice.contract::<StatusMessage>(&contract_id);

status_message.set_status("hello").await?.result()?;
let status = status_message.get_status(alice.account_id()).await?;
```

See [Typed contracts and tokens](https://docs.rs/near-kit/latest/near_kit/guide/contracts_and_tokens/index.html) in the near-kit guide, which also covers its ready-made clients for fungible and non-fungible tokens.

### Sandbox configuration and environment variables

`SandboxConfig` sets what near-workspaces didn't let you change:

```rust
use near_sandbox::{GenesisAccount, SandboxConfig};

let node = Sandbox::start_sandbox_with_config(SandboxConfig {
    // Merged into genesis.json
    additional_genesis: Some(json!({ "epoch_length": 100 })),
    // Accounts that exist from the start
    additional_accounts: vec![GenesisAccount::default_with_name(account!("alice"))],
    // A fixed RPC port instead of a random one
    rpc_port: Some(3030),
    ..Default::default()
})
.await?;
```

`additional_config` is merged into the node's config.json the same way, and `max_payload_size` and `max_open_files` replace the environment variables below.

| near-workspaces | near-sandbox |
|---|---|
| `NEAR_SANDBOX_BIN_PATH` | the same |
| `NEAR_ENABLE_SANDBOX_LOG` | the same. `NEAR_SANDBOX_LOG` sets the log filter, such as `near=info,runtime=debug` |
| `NEAR_SANDBOX_MAX_PAYLOAD_SIZE` | the same, or `SandboxConfig::max_payload_size` |
| `NEAR_SANDBOX_MAX_FILES` | the same, or `SandboxConfig::max_open_files` |
| `NEAR_RPC_TIMEOUT_SECS` | the same, but only for how long to wait for the node to start (10 seconds by default) |
| `NEAR_RPC_API_KEY` | `RpcEndpoint::new(url).bearer(key)` in near-kit |

near-sandbox also reads `SANDBOX_ARTIFACT_URL`, a URL to download the sandbox binary from.

### Quick reference

| near-workspaces | near-kit and near-sandbox |
|---|---|
| `near_workspaces::sandbox()` | `Sandbox::start_sandbox()`, then `Near::custom(&node.rpc_addr, "localnet").build()` |
| `sandbox_with_version(v)` | `Sandbox::start_sandbox_with_version(v)` |
| `testnet()`, `mainnet()` | `Near::testnet().build()`, `Near::mainnet().build()` |
| `worker.root_account()` | `near.with_signer(InMemorySigner::new(DEFAULT_GENESIS_ACCOUNT, key))` |
| `dev_create_account()`, `create_subaccount(..)` | a transaction with `create_account`, `transfer` and `add_full_access_key` |
| `dev_deploy(&wasm)` | the same transaction with `deploy_contract` |
| `create_tla(id, sk)` | `node.create_account(id).public_key(..).send()` |
| `Account`, `Contract` | a `SigningNear` for the account, an `AccountId` for the contract |
| `Account::from_secret_key(..)`, `as_account()` | `near.with_signer(InMemorySigner::new(id, key))` |
| `SecretKey::from_random(KeyType::ED25519)` | `SecretKey::generate_ed25519()` |
| `call(..).args_json(..).transact()` | `transaction(..).add_action(FunctionCall::new(..).args(..))` |
| `max_gas()` | `gas(Gas::from_tgas(1_000))` |
| `view(..).args_json(..).await?.json()?` | `near.view::<T>(..).args(..).await?` |
| `batch(id)` | one `transaction(id)` with several actions |
| `transact_async()`, `status()` | `wait_until::<Included>()`, `near.tx_status(..)` |
| `transfer_near(..)`, `deploy(..)`, `delete_account(..)` | `Action::transfer`, `Action::deploy_contract`, `Action::delete_account` |
| `into_result()?` | `result()?` or `checked()?` |
| `total_gas_burnt` | `total_gas_used()` |
| `GasMeter` | add up `total_gas_used()` of each transaction |
| `view_account()` | `near.balance(id)` or `near.rpc().view_account(..)` |
| `view_block()`, `view_state()`, `view_code()`, `view_access_keys()` | `block`, `view_state_all`, `view_code`, `view_access_key_list` on `near.rpc()` |
| `fast_forward(n)` | `node.fast_forward(n)` |
| `patch_state(..)`, `patch(..)` | `node.patch_state(id)` |
| `import_contract(..)` | `node.import_account(rpc_url, id)` |
| `compile_project(path)` | `cargo_near_build::build_with_cli(..)` |
| `near_abi_client` | `#[near_kit::contract]` |

## Release notes

Release notes of the published versions are in the [CHANGELOG](CHANGELOG.md).
