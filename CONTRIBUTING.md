# Contributing to workspaces-rs

Thank you for your interest in contributing to NEAR's Rust Workspaces! We appreciate any type of contribution.

Below are various bits of information to help you get started. If you require additional help, please reach out to us on the `Rust Support` channel on [discord](https://discord.gg/nearprotocol) or our [zulip channel](https://near.zulipchat.com/)

## Quick Start

`workspaces-rs` is like any other rust project so we can easily get started by doing `cargo build`.

## Code of Conduct

We have an open and welcoming environment, please review our [code of conduct](CODE_OF_CONDUCT.md).

## Development

### Architecture

The current design of workspaces is mostly just a client communicating to an RPC service. For mainnet/testnet, this is just pointing the client to the `near.org` RPCs for each of them respectively. For sandbox, this is a little more involved where we spin up a local sandbox node when a user directly calls into `workspaces::sandbox()`. A `Worker` is considered the client and main way to interact with each of these networks. It offers a single interface into calling common RPC calls between all the networks, and also allowing for specific features for each network.

### Commits

Please use descriptive PR titles. We loosely follow the [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) style, but this is not a requirement to follow exactly. PRs will be addressed more quickly if it is clear what the intention is.

### Before opening a PR

Ensure the following are satisfied before opening a PR:

- Code is formatted with `rustfmt` by running `cargo fmt`
- Run `cargo clippy`
- Run tests with `cargo test`
- Ensure any new functionality is adequately tested
- If any new public types or functions are added, ensure they have appropriate [rustdoc](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html) documentation
- Add an entry to the CHANGELOG.md
- Optional. Consider running the actions locally to ensure they pass. See [here](https://github.com/nektos/act).
