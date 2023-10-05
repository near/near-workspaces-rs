# Changelog

## [Unreleased]

### Added

- RPC API Keys used to interact with services such as Pagoda Console.
- [Import a couple functions over from near_crypto for PublicKey](https://github.com/near/workspaces-rs/pull/265)
  - Impl `Ord`, `PartialOrd`, `Hash`, `BorshSerialize`, `BorshDeserialize`, `Display`, and `FromStr` for `PublicKey`
    - NOTE: Borsh bytes format is the same as near-sdk, where it is in the form of [bytes_len, key_type, key_data..]
  - Added `PublicKey::{empty, len, key_data}`
  - Impl `Display` for `SecretKey`.
  - more docs were added to both `SecretKey` and `PublicKey`.
  - Impl `Display`, `FromStr`, `TryFrom<u8>` for `KeyType`.
- [Added `TryFrom<near_sdk::PublicKey>` for `workspaces::PublicKey`](https://github.com/near/workspaces-rs/pull/267)
  - Added `KeyType::len` and `PublicKey::try_from_bytes`
- [Added experimental apis from near-sdk-rs](https://github.com/near/near-workspaces-rs/pull/285), available under the **experimental** flag.
  - Methods added are: EXPERIMENTAL_changes_in_block, EXPERIMENTAL_changes, EXPERIMENTAL_genesis_config, EXPERIMENTAL_protocol_config, EXPERIMENTAL_receipt, EXPERIMENTAL_tx_status, EXPERIMENTAL_validators_ordered
- [Added Worker::patch to patch account, keys, code, and state in a generic builder](https://github.com/near/near-workspaces-rs/pull/291)
  - Added `Worker::patch` and `PatchTransaction` that provide builders for patching accounts, keys, code, and state.
  - Added `AccountDetails` and `AccountDetailsPatch` which hold the state of the patch.

### Changed

- [`Transaction::transact_async` no longer has a lifetime parameter to make it easier to use](https://github.com/near/workspaces-rs/pull/249)
- [Improved error message on calling a json on a void function](https://github.com/near/near-workspaces-rs/pull/286)
- [Removed serde-arbitrary-precision feature in examples](https://github.com/near/near-workspaces-rs/pull/287)

### Fixed

- [Upgraded to Rust Stable Toolchain](https://github.com/near/near-workspaces-rs/commit/8d93197d06aee2a426b6da99e270ce1658c2cd4f). Deprecates requirement of only using rustc-1.69 and lower.
- [Run `neard` on `localhost` instead of `0.0.0.0` to prevent firewall popups on MacOS](https://github.com/near/workspaces-rs/issues/276)

## [0.7.0]

### Added

- [`view_*` asynchronous builders have been added which provides being able to query from a specific block hash or block height](https://github.com/near/workspaces-rs/pull/218)
- [`{CallTransaction, Transaction}::transact_async` for performing transactions without directly having to wait for it complete it on chain](https://github.com/near/workspaces-rs/pull/222)
- [`view_chunk` added for querying into chunk related info on the network.](https://github.com/near/workspaces-rs/pull/234)
  - Adds `Chunk` and `ChunkHeader` type to reference specific chunk info.
- [`Error::{simple, message, custom}` are now public and usable for custom errors](https://github.com/near/workspaces-rs/pull/224)

### Changed

- [Apart of the changes from adding `view_*` async builders, we have a couple breaking changes to the `view_*` functions](https://github.com/near/workspaces-rs/pull/218):
  - `{Account, Contract, Worker}::view_state` moved `prefix` parameter into builder. i.e.
    ```
    worker.view_state("account_id", Some(prefix)).await?;
    // is now
    worker.view_state("account_id")
        .prefix(prefix)
        .await?;
    // if prefix was `None`, then simply delete the None argument.
    ```
  - `view` function changed to be a builder, and no longer take in `args` as a parameter. It instead has been moved to the builder side.
  - Changed `Worker::view_latest_block` to `Worker::view_block` as the default behavior is equivalent.
  - `operations::Function` type no longer takes a lifetime parameter.
  - `operations::CallTransaction` type takes one less lifetime parameter.
- [`Worker::call` signature changed to be more in line with `view_*` async builders. It will now return a builder like `{Account, Contract}::call`](https://github.com/near/workspaces-rs/pull/245)
  - This `call` no longer accepts `Contract` since that was not as accessible. Instead a `InMemorySigner` is now required to sign transactions (which can be retrieved from `{Account, Contract}::signer` or `InMemorySigner::{from_secret_key, from_file}`).
  - `{Account, Contract}::signer` now exposed.

### Fixed

- [Changed the docs to reflect proper size of of rate limits on near.org RPC](https://github.com/near/workspaces-rs/pull/219)
- [Cached nonces now are per account-id and public-key instead of just public-key](https://github.com/near/workspaces-rs/pull/231)
  - this didn't matter if only one KeyPair was being used per account, but could be problematic when there were multiple KeyPairs per account utilizing the same nonces.
- [Error message context wasn't being exposed properly by sandbox, so this fixed it](https://github.com/near/workspaces-rs/pull/236)

## [0.6.1]

### Fixed

- Fixed query variant error when supply invalid function name or arguments: https://github.com/near/workspaces-rs/pull/239

## [0.6.0]

### Added

- `Account::view` API exposed: https://github.com/near/workspaces-rs/pull/202

### Changed

- Unstable `compile_project` uses new the workspaces errors: https://github.com/near/workspaces-rs/pull/204
- `ValueOrReceiptId::Value(String)` changed to `ValueOrReceiptId::Value(Value)`: https://github.com/near/workspaces-rs/pull/208
  - `Value` type offers convenient APIs like `raw_bytes`, `json`, and `borsh` like one would find from a `ExecutionFinalResult`.
- internal dependencies like near-jsonrpc-client upgraded to 0.4.0 from 0.4.0-beta: https://github.com/near/workspaces-rs/pull/210
  - Note, the RNG for `SecretKey::{from_random, from_seed}(KeyType::SECP256K1, ...)` has been changed as well, and will produce different keys than before.

### Fixed

- `docs.rs` now shows `unstable` feature flag: https://github.com/near/workspaces-rs/pull/198
- No longer orphaning sandbox processes on early termination of tests: https://github.com/near/workspaces-rs/pull/205
- Fixed sandbox colliding installs: https://github.com/near/workspaces-rs/pull/211
- sandbox no longer spamming stats logs: https://github.com/near/workspaces-rs/pull/213

## [0.5.0]

### Added

- Error handling with opaque `workspaces::error::Error` type: https://github.com/near/workspaces-rs/pull/149
- Require `#[must_use]` on the Execution value returned by `transact()`: https://github.com/near/workspaces-rs/pull/150
  - Added `ExecutionFinalResult`, `ExecutionResult`, `ExecutionSuccess` and `ExecutionFailure` types
  - Added `into_result()` to easily handle `#[must_use] ExecutionFinalResult`
  - Added `unwrap()` to not care about Err variant in `ExecutionResult`s

### Changed

- Renamed CallExecution\* types: https://github.com/near/workspaces-rs/pull/150
  - Renamed ` CallExecution`` to  `Execution`
  - Renamed `CallExecutionDetails` to `ExecutionFinalResult`
- `args_json` and `args_borsh` no longer return `Result`s and are deferred till later when `transact()`ed: https://github.com/near/workspaces-rs/pull/149
- API changes from removing `worker` parameter from function calls: https://github.com/near/workspaces-rs/pull/181
  - `Account::from_file` function signature change, requiring a `&worker` to be passed in.
  - `workspaces::prelude::*` import no longer necessary, where we no longer able to import `workspaces::prelude::DevAccountDeployer` directly.

### Removed

- Removed impls from exection result: https://github.com/near/workspaces-rs/pull/150
  - Removed `impl<T> From<CallExecution<T>> for Result<T>`
  - Removed `impl From<FinalExecutionOutcomeView> for CallExecutionDetails`
- No longer require `worker` to be passed in for each transaction: https://github.com/near/workspaces-rs/pull/181

### Fixed

- Gas estimation issue resolved with latest sandbox node (Aug 29, 2022): https://github.com/near/workspaces-rs/pull/188
- Fixed parallel tests, where calling into the same contract would require waiting on a previous call: https://github.com/near/workspaces-rs/pull/173

## [0.4.1] - 2022-08-16

### Added

- Derive `Eq` on `AccountDetails` type: https://github.com/near/workspaces-rs/pull/177/files

### Fixed

- Fix macOS non-deterministic overflow error when starting up sandbox: https://github.com/near/workspaces-rs/pull/179

## [0.4.0] - 2022-07-20

### Added

- Mac M1 Support: https://github.com/near/workspaces-rs/pull/169
- Added `Account::secret_key` to grab the account's secret key: https://github.com/near/workspaces-rs/pull/144
- `Debug`/`Clone` impls for `Account`/`Contract`, and `Debug` for `Worker`: https://github.com/near/workspaces-rs/pull/167
- `ExecutionOutcome::tokens_burnt` is now available: https://github.com/near/workspaces-rs/pull/168

### Fixed

- internally no longer creating a new RPC client per call: https://github.com/near/workspaces-rs/pull/154
- upped near dependencies to fix transitive vulnerabilities: https://github.com/near/workspaces-rs/pull/169

### Changed

- Default sandbox version is now using commit hash master/13a66dda709a4148f6395636914dca2a55df1390 (July 18, 2022): https://github.com/near/workspaces-rs/pull/169

## [0.3.1] - 2022-06-20

### Added

- Raw bytes API similar to `json`/`borsh` calls: https://github.com/near/workspaces-rs/pull/133/files
- Expose `types` module and added `SecretKey` creation: https://github.com/near/workspaces-rs/pull/139

### Fixed

- If sandbox gets started multiple times, short circuit it early on: https://github.com/near/workspaces-rs/pull/135
- Fix short timeouts on connecting to RPC for macos with custom env variable to specify timeout if needed: https://github.com/near/workspaces-rs/pull/143

## [0.3.0] - 2022-05-10

### Added

- Added betanet support https://github.com/near/workspaces-rs/pull/116

### Changed

- Updated default sandbox version to `97c0410de519ecaca369aaee26f0ca5eb9e7de06` commit of nearcore to include 1.26 protocol changes https://github.com/near/workspaces-rs/pull/134

- Exposed `CallExecutionDetails::raw_bytes` API: https://github.com/near/workspaces-rs/pull/133

## [0.2.1] - 2022-04-12

### Added

- Added more docs to top level or exposed types/functions: https://github.com/near/workspaces-rs/pull/115

### Fixed

- Fix `docs.rs` builds failing on sandbox install: https://github.com/near/workspaces-rs/pull/115

## [0.2.0] - 2022-04-05

### Added

- Time-traveling - the ability to go forwards in block height within tests. This allows to test time specific data changing within contracts: https://github.com/near/workspaces-rs/pull/73
- Credentials created from account/contract creation are now allowed to be stored and specified by users. https://github.com/near/workspaces-rs/pull/98
- [Unstable] Allow users to compile contract projects within tests without having to manually go through this step. https://github.com/near/workspaces-rs/pull/77
- Batch transactions or transactions with multiple actions are now possible. https://github.com/near/workspaces-rs/pull/72
- Sandbox node (nearcore binary) logs are now suppressed and can be re-enabled if desired. https://github.com/near/workspaces-rs/pull/85
- Results now expose logs, receipts, and transaction outcome values. https://github.com/near/workspaces-rs/pull/70
- Convenience methods `Worker::view_code`, `Worker::view_latest_block`, `Worker::view_account`, `Account::view_account`, `Contract::view_account`, `Contract::view_code` now available. https://github.com/near/workspaces-rs/pull/82
- Improve error handling. If a transaction fails, this error will now be apart of the `Result` return initially. https://github.com/near/workspaces-rs/pull/83
- Added `tracing` logging to internal code and examples. https://github.com/near/workspaces-rs/pull/55 and https://github.com/near/workspaces-rs/pull/75
- Convenient `CallExecutionDetails::{is_success, is_failure}` for testing outcomes of transactions. https://github.com/near/workspaces-rs/pull/58
- Added `mainnet_archival` and `testnet_archival`, where `ref-finance` example now uses `mainnet_archival`. https://github.com/near/workspaces-rs/pull/57 and https://github.com/near/workspaces-rs/pull/94

### Changed

- key type for `patch_state` now a slice and no longer require `StoreKey`. https://github.com/near/workspaces-rs/pull/109
- Reorganized imports internally for better maintainability. https://github.com/near/workspaces-rs/pull/102
- No longer running into non-deterministic query failures if RPC isn't available, but this is a breaking API. All `workspaces::{sandbox, testnet, mainnet}` now require `.await?` at the end. https://github.com/near/workspaces-rs/pull/99
- TLA trait no longer apart of all networks -- only dev-networks (sandbox, testnet). https://github.com/near/workspaces-rs/pull/101
- Retry times have now been shorten and should take a maximum of 1 second. https://github.com/near/workspaces-rs/pull/92
- doc builds on [docs.rs](https://docs.rs) has now been fixed. https://github.com/near/workspaces-rs/pull/90
- `patch_state` now takes in slices. https://github.com/near/workspaces-rs/pull/80 and https://github.com/near/workspaces-rs/pull/79
- Make `access_key` call do optimistic queries which led to better retry times. https://github.com/near/workspaces-rs/pull/60
- Functions no longer take in owned but referenced `AccountId`s now. https://github.com/near/workspaces-rs/pull/52

### Removed

- Empty JSON array is no longer a valid default argument supplied to transactions. Recommended to supply empty `{}` in the case of JSON if all function arguments in the contract are optional types. https://github.com/near/workspaces-rs/pull/84

## [0.1.1] - 2021-01-24

### Changed

- Fix race condition when installing sandbox and running multiples tests at the same time. https://github.com/near/workspaces-rs/pull/46

[unreleased]: https://github.com/near/workspaces-rs/compare/0.7.0...HEAD
[0.7.0]: https://github.com/near/workspaces-rs/compare/0.6.1...0.7.0
[0.6.1]: https://github.com/near/workspaces-rs/compare/0.6.0...0.6.1
[0.6.0]: https://github.com/near/workspaces-rs/compare/0.5.0...0.6.0
[0.5.0]: https://github.com/near/workspaces-rs/compare/0.4.1...0.5.0
[0.4.1]: https://github.com/near/workspaces-rs/compare/0.4.0...0.4.1
[0.4.0]: https://github.com/near/workspaces-rs/compare/0.3.1...0.4.0
[0.3.1]: https://github.com/near/workspaces-rs/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/near/workspaces-rs/compare/0.2.1...0.3.0
[0.2.1]: https://github.com/near/workspaces-rs/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/near/workspaces-rs/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/near/workspaces-rs/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/near/workspaces-rs/releases/tag/0.1.0
