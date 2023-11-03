//! # NEAR Workspaces
//!
//! A library for automating workflows and writing tests for NEAR smart contracts.
//! This software is not final, and will likely change.

// We want to enable all clippy lints, but some of them generate false positives.
#![allow(clippy::missing_const_for_fn, clippy::redundant_pub_crate)]

#[cfg(feature = "unstable")]
mod cargo;
#[cfg(feature = "unstable")]
pub use cargo::compile_project;

mod worker;

pub mod error;
pub mod network;
pub mod operations;
pub mod prelude;
pub mod result;
pub mod rpc;
pub mod types;

/// The near_abi_client implementation is currently in flux and we offer a re-export
/// of it and example code. No public near_abi APIs are baked into workspace-rs yet.
pub use near_abi_client;

pub use network::pick_unused_port;
pub use network::variants::{DevNetwork, Network};
pub use result::Result;
pub use types::account::{Account, AccountDetailsPatch, Contract};
pub use types::block::Block;
pub use types::chunk::Chunk;
pub use types::{AccessKey, AccountId, BlockHeight, CryptoHash, InMemorySigner};
pub use worker::{
    betanet, mainnet, mainnet_archival, sandbox, sandbox_with_version, testnet, testnet_archival,
    with_betanet, with_mainnet, with_mainnet_archival, with_sandbox, with_testnet,
    with_testnet_archival, Worker,
};

#[cfg(feature = "unstable")]
pub use worker::{custom, with_custom};
