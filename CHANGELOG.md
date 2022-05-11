# Changelog

## [unreleased]

### Changed
- Updated default sandbox version to `97c0410de519ecaca369aaee26f0ca5eb9e7de06` commit of nearcore to include 1.26 protocol changes

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


[Unreleased]: https://github.com/near/workspaces-rs/compare/0.2.1...HEAD
[0.2.1]: https://github.com/near/workspaces-rs/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/near/workspaces-rs/compare/0.1.1...0.2.0
[0.1.1]: https://github.com/near/workspaces-rs/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/near/workspaces-rs/releases/tag/0.1.0