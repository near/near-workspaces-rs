//! # NEAR Workspaces
//!
//! A library for automating workflows and writing tests for NEAR smart contracts.
//! This software is not final, and will likely change.

#[cfg(feature = "unstable")]
mod cargo;
#[cfg(feature = "unstable")]
pub use cargo::compile_project;

mod rpc;
mod worker;

pub mod error;
pub mod network;
pub mod operations;
pub mod prelude;
pub mod result;
pub mod types;

pub use network::variants::{DevNetwork, Network};
pub use types::account::{Account, AccountDetails, Contract};
pub use types::block::Block;
pub use types::{AccessKey, AccountId, BlockHeight, CryptoHash, InMemorySigner};
pub use worker::{
    betanet, mainnet, mainnet_archival, sandbox, testnet, testnet_archival, with_betanet,
    with_mainnet, with_mainnet_archival, with_sandbox, with_testnet, with_testnet_archival, Worker,
};
