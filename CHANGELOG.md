# Changelog

## [Unreleased]

## [0.8.0](https://github.com/near/near-workspaces-rs/compare/near-workspaces-v0.7.0...near-workspaces-v0.8.0) - 2023-10-04

### Added
- Allow to select a specific version of near-sandbox ([#311](https://github.com/near/near-workspaces-rs/pull/311))
- Enable support for RPCs that require API keys and support for custom networks ([#306](https://github.com/near/near-workspaces-rs/pull/306))
- expose more `Block` and `Chunk` fields ([#243](https://github.com/near/near-workspaces-rs/pull/243))
- Expose experimental apis (under a feature-flag) ([#285](https://github.com/near/near-workspaces-rs/pull/285))
- New patch state, key, account API ([#291](https://github.com/near/near-workspaces-rs/pull/291))
- support manually supplied validator key ([#274](https://github.com/near/near-workspaces-rs/pull/274))
- workspaces::sandbox() can connect to user provided sandbox node ([#220](https://github.com/near/near-workspaces-rs/pull/220))
- Expose `transact async` for asynchronous transactions ([#222](https://github.com/near/near-workspaces-rs/pull/222))
- Add async builders for `view_*` functions ([#218](https://github.com/near/near-workspaces-rs/pull/218))
- add view calls to an `Account` ([#202](https://github.com/near/near-workspaces-rs/pull/202))
- must use CallExecution* type ([#150](https://github.com/near/near-workspaces-rs/pull/150))
- `worker` no longer required to be passed into arguments when transacting transactions ([#181](https://github.com/near/near-workspaces-rs/pull/181))
- error handling ([#149](https://github.com/near/near-workspaces-rs/pull/149))
- allow Worker<Betanet> creation ([#116](https://github.com/near/near-workspaces-rs/pull/116))
- add fast-forwarding ([#73](https://github.com/near/near-workspaces-rs/pull/73))
- Moved credentials storage to user side ([#98](https://github.com/near/near-workspaces-rs/pull/98))
- add a way to automatically compile and deploy a contract ([#77](https://github.com/near/near-workspaces-rs/pull/77))
- batch transactions ([#72](https://github.com/near/near-workspaces-rs/pull/72))
- add rpc logging ([#75](https://github.com/near/near-workspaces-rs/pull/75))
- Switch prints to `tracing` ([#55](https://github.com/near/near-workspaces-rs/pull/55))

### Fixed
- improve error msg on calling `json` on void function ([#286](https://github.com/near/near-workspaces-rs/pull/286))
- fix typos ([#280](https://github.com/near/near-workspaces-rs/pull/280))
- Run `neard` on `localhost` instead of `0.0.0.0` to prevent firewall popups on MacOS ([#277](https://github.com/near/near-workspaces-rs/pull/277))
- storing credentials ([#258](https://github.com/near/near-workspaces-rs/pull/258))
- ulimit error ([#241](https://github.com/near/near-workspaces-rs/pull/241))
- Make call consistent with worker::view ([#245](https://github.com/near/near-workspaces-rs/pull/245))
- improve err message ([#236](https://github.com/near/near-workspaces-rs/pull/236))
- remove unused import of metadata ([#238](https://github.com/near/near-workspaces-rs/pull/238))
- Use public keys for cached nonces ([#231](https://github.com/near/near-workspaces-rs/pull/231))
- Parallel tests should not wait on each other ([#173](https://github.com/near/near-workspaces-rs/pull/173))
- Rewording docs to be more intuitive ([#186](https://github.com/near/near-workspaces-rs/pull/186))
- avoid creating new client for every request & hyper issues ([#154](https://github.com/near/near-workspaces-rs/pull/154))
- rpc connect timeout ([#143](https://github.com/near/near-workspaces-rs/pull/143))
- error if sandbox process has already started ([#135](https://github.com/near/near-workspaces-rs/pull/135))
- [**breaking**] Key type of state map and remove exposed primitives type ([#109](https://github.com/near/near-workspaces-rs/pull/109))
- make users wait for sandbox to completely startup ([#99](https://github.com/near/near-workspaces-rs/pull/99))
- move TLA creator trait to dev network ([#101](https://github.com/near/near-workspaces-rs/pull/101))
- make empty array the default function argument ([#84](https://github.com/near/near-workspaces-rs/pull/84))
- Make `access_key` do an optimistic RPC query ([#60](https://github.com/near/near-workspaces-rs/pull/60))

### Other
- [**breaking**] renamed crate to near-workspaces to avoid confusion with Cargo workspaces; imports should now use `near_workspaces` instead of just `workspaces` ([#318](https://github.com/near/near-workspaces-rs/pull/318))
- drop async-process in favor of tokio ([#316](https://github.com/near/near-workspaces-rs/pull/316))
- switch to `near-gas` crate for Gas where possible ([#305](https://github.com/near/near-workspaces-rs/pull/305))
- Improved fast_forward docs ([#299](https://github.com/near/near-workspaces-rs/pull/299))
- Added test for delete_account ([#289](https://github.com/near/near-workspaces-rs/pull/289))
- Added a test for transfer_near ([#290](https://github.com/near/near-workspaces-rs/pull/290))
- using url return type ([#297](https://github.com/near/near-workspaces-rs/pull/297))
- dependencies and removed unused deps ([#292](https://github.com/near/near-workspaces-rs/pull/292))
- upgrade to stable toolchain ([#293](https://github.com/near/near-workspaces-rs/pull/293))
- Updated near deps to 0.17 ([#283](https://github.com/near/near-workspaces-rs/pull/283))
- add `sdk::PublicKey` to `workspaces::PublicKey` conversion ([#267](https://github.com/near/near-workspaces-rs/pull/267))
- Use cargo-near to build project ([#275](https://github.com/near/near-workspaces-rs/pull/275))
- Added network builder for mainnet, testnet, betanet ([#221](https://github.com/near/near-workspaces-rs/pull/221))
- bump borsh version and other deps ([#271](https://github.com/near/near-workspaces-rs/pull/271))
- bump sandbox to 0.6.2 ([#270](https://github.com/near/near-workspaces-rs/pull/270))
- Import some functions over from near_crypto for PublicKey ([#265](https://github.com/near/near-workspaces-rs/pull/265))
- Added destination account-id for `import_contract` call ([#260](https://github.com/near/near-workspaces-rs/pull/260))
- Fix port collision ([#257](https://github.com/near/near-workspaces-rs/pull/257))
- Removed the lifetime in transact_async ([#249](https://github.com/near/near-workspaces-rs/pull/249))
- configure sandbox ([#251](https://github.com/near/near-workspaces-rs/pull/251))
- 0.7.0 ([#244](https://github.com/near/near-workspaces-rs/pull/244))
- expose `view_chunk` ([#234](https://github.com/near/near-workspaces-rs/pull/234))
- Expose error creation methods ([#224](https://github.com/near/near-workspaces-rs/pull/224))
- Bump to version 0.6.1 ([#240](https://github.com/near/near-workspaces-rs/pull/240))
- Fix query variant error + add test ([#239](https://github.com/near/near-workspaces-rs/pull/239))
- Fix docs containing 50mb instead of 50kb ([#219](https://github.com/near/near-workspaces-rs/pull/219))
- 0.6.0 ([#213](https://github.com/near/near-workspaces-rs/pull/213))
- Made ValueOrReceiptId::Value object consistent ([#208](https://github.com/near/near-workspaces-rs/pull/208))
- Bump sandbox to latest ([#211](https://github.com/near/near-workspaces-rs/pull/211))
- Upgrade to version 0.4.0 of near-jsonrpc-client, and version 0.15.0 for nearcore libraries ([#210](https://github.com/near/near-workspaces-rs/pull/210))
- Fix child sandbox process becoming orphaned on early termination ([#205](https://github.com/near/near-workspaces-rs/pull/205))
- Make compile_project use latest error API ([#204](https://github.com/near/near-workspaces-rs/pull/204))
- add unstable feature to docs build ([#198](https://github.com/near/near-workspaces-rs/pull/198))
- Up version to 0.5.0
- 0.5.0 ([#188](https://github.com/near/near-workspaces-rs/pull/188))
- Added convenience functions `Account::{from_secret_key, set_secret_key}` ([#185](https://github.com/near/near-workspaces-rs/pull/185))
- 0.4.1 ([#180](https://github.com/near/near-workspaces-rs/pull/180))
- Fix macos non-deterministic overflow error ([#179](https://github.com/near/near-workspaces-rs/pull/179))
- Fix tests erroring out on newest rust v1.63.0 ([#177](https://github.com/near/near-workspaces-rs/pull/177))
- 0.4.0 ([#170](https://github.com/near/near-workspaces-rs/pull/170))
- Expose tokens_burnt ([#168](https://github.com/near/near-workspaces-rs/pull/168))
- Update Deps Along w/ M1 Support ([#169](https://github.com/near/near-workspaces-rs/pull/169))
- Added Debug + Clone to Account and Contract ([#167](https://github.com/near/near-workspaces-rs/pull/167))
- Expose account keys ([#144](https://github.com/near/near-workspaces-rs/pull/144))
- Bump vworkspaces to 0.3.1 ([#152](https://github.com/near/near-workspaces-rs/pull/152))
- add secret key creation methods and added new KeyType type ([#139](https://github.com/near/near-workspaces-rs/pull/139))
- expose raw_bytes API ([#133](https://github.com/near/near-workspaces-rs/pull/133))
- bump workspaces patch version and update changelog ([#136](https://github.com/near/near-workspaces-rs/pull/136))
- bump sandbox version default for SDK change ([#134](https://github.com/near/near-workspaces-rs/pull/134))
- Added cross contract tests ([#123](https://github.com/near/near-workspaces-rs/pull/123))
- 0.2.1 ([#117](https://github.com/near/near-workspaces-rs/pull/117))
- doc builds on docs.rs ([#115](https://github.com/near/near-workspaces-rs/pull/115))
- 0.2 ([#104](https://github.com/near/near-workspaces-rs/pull/104))
- [**breaking**] Reorganize exports and modules exposed ([#102](https://github.com/near/near-workspaces-rs/pull/102))
- cleanup before release 0.2 ([#94](https://github.com/near/near-workspaces-rs/pull/94))
- Reduce retries time ([#92](https://github.com/near/near-workspaces-rs/pull/92))
- fix doc builds and adds README ([#90](https://github.com/near/near-workspaces-rs/pull/90))
- suppress sandbox bin logs ([#85](https://github.com/near/near-workspaces-rs/pull/85))
- `CallResultDetails` now expose logs, transaction and receipt outcomes ([#70](https://github.com/near/near-workspaces-rs/pull/70))
- Add view_account, view_code, view_block ([#82](https://github.com/near/near-workspaces-rs/pull/82))
- Improve error handling: error out on failed final status ([#83](https://github.com/near/near-workspaces-rs/pull/83))
- change patch state parameters to be slices ([#80](https://github.com/near/near-workspaces-rs/pull/80))
- make patch_state accept Vec<u8> ([#79](https://github.com/near/near-workspaces-rs/pull/79))
- Added `is_success` / `is_failure` for CallExecution results ([#58](https://github.com/near/near-workspaces-rs/pull/58))
- 0.1.1 ([#59](https://github.com/near/near-workspaces-rs/pull/59))
- Adds mainnet archival rpc and fixes ref-finance example ([#57](https://github.com/near/near-workspaces-rs/pull/57))
- (dev_)deploy is now consistent across the board ([#56](https://github.com/near/near-workspaces-rs/pull/56))
- [**breaking**] don't require owned AccountId for RPC abstractions ([#52](https://github.com/near/near-workspaces-rs/pull/52))
- Fixup race condition when installing near-sandbox
- Merge branch 'main' into release/0.1
- Update deploy to not consume self
- Update README w/ sandbox-utils version from crates.io
- Added licenses
- worker.create_tla for Mainnet unsupported for now
- Update docs
- Merge branch 'feat/call-builder' of github.com:near/workspaces-rs into example/ref-finance
- Rename amount_yocto to amount & correct docs
- Added docs
- Add convenient deploy function
- Format
- Update examples with newer accounts API
- Move client.call defaults out
- Rename dev_create{,_account}
- Add DeserializedOwned & result sig for json/borsh
- Add #[non_exhaustive] to detail objects
- Drop with_ prefix for builders
- Add additional lifetimes
- Rename try_* to .borsh and .json
- Add serde::Serialize to with_args_json
- Fixup examples/tests
- Add serde/borsh deser from view calls
- Added create_account for sub account creation
- Format
- Merge branch 'main' of github.com:near/workspaces-rs into feat/call-builder
- Added helper functions for transfer_near, delete_account, view
- Change function sig of view to return Json String
- Add call builder
- fmt
- Fix macos build
- Depend on libc for unix only
- Guard unix code
- Do not expose internal borsh
- Merge branch 'feat/rearrange' of github.com:near/workspaces-rs into feat/no-pub-nearcore-types
- Update example to use json! & make proj version agnostic for now
- Consistent fn sig between call & view
- Make dev_generate async
- One more fmt
- Merge branch 'main' of github.com:near/workspaces-rs into feat/rearrange
- Added into_result for CallExecution
- Do not expose internal client functions
- fmt
- Clean up impls and prelude
- Update patch_state to take in Vec<u8>
- deploy code now only accepts bytes instead of filepath
- Made NetworkInfo return a Info struct
- Removed send/sync for account.rs and misc small fixes
- Bump workspaces version up to 0.2
- Update tests to use Result
- Removed unused cargo deps
- Removed Network Actions
- Deleted tokio::main export
- Moved SandboxServer code into network/
- Removed old style API code
- CallExecution{Result => Details} and add transfer_near/delete_account
- Added CallExecution* to network/result.rs
- Remove unused imports and fix clippy warnings
- Update create_tla sig to take signer
- Format
- Update spooning example w/ new API
- Update status_msg/patch_state ex/test
- Add worker.view_state
- Minor fix for warnings
- Added patching state to sandbox network
- client.{call => query, _call => call}
- Added Worker<Testnet>
- Some clean up
- fmt deploy test
- Cargo fmt
- tests/deploy.rs now conform to new API
- Change TLA deploy to take in signer
- Added account.rs
- Add Worker::sandbox()
- Added client.deploy fn
- Lessen visibility of default construction for Sandbox network
- Add Arc<T> for Worker<T>
- Added forgotten generic code for Worker
- Moved account/contract code into account.rs
- Added some docs on NetworkInfo
- Update deploy example
- Formatting
- Added dev_deploy test for new style
- Make sandbox server components accessible within project
- Cleaning up Server trait
- Move dev_generate into trait & added client dep
- Blanket impl for anything that impls the parts of a Network
- Added dev_deploy impl
- Change return sig of dev_deploy to contract instead
- Expose helper fn for sandbox server
- Added sandbox Network impl
- Added forgotten type parameters
- Added network/worker modules
- Client now takes in rpc_addr
- Moved delete_account/view_state into client.rs
- Moved create_account into client.rs
- Moved view/transfer_near into client.rs
- Move call into client.rs
- Merge branch 'main' of https://github.com/near/workspaces-rs into feat/exponential-backoff
- Merge pull request [#16](https://github.com/near/near-workspaces-rs/pull/16) from near/feat/spooning
- Stop exposing `FinalExecutionOutcomeView` as the return type of many call-like functions
- Merge branch 'main' of https://github.com/near/sandbox-api into austin/tokio_gen
- Merge branch 'main' of https://github.com/near/sandbox-api into feat/testnet-impl
- Cargo fmt
- Merge branch 'main' of https://github.com/near/sandbox-api into chore/rename-workspaces
- Renamed runner => workspaces

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
