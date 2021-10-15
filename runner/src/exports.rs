pub use near_crypto::{InMemorySigner, PublicKey, Signer};
pub use near_primitives::borsh;
pub use near_primitives::types::AccountId;

/// Allow users to use `#[runner::basic]` to not use any kind of NEAR runtimes
/// and just purely use the underlying "basic" runtime to run tasks instead.
pub use tokio::main as basic;
