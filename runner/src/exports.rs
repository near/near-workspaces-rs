pub use near_crypto::{InMemorySigner, PublicKey, Signer};
pub use near_primitives::borsh;
pub use near_primitives::types::AccountId;

/// Allow users to use `#[runner::basic]` to not use any kind of NEAR runtimes
/// and just purely use the underlying "basic" runtime to run tasks instead.
pub use tokio::main as basic;

// /// Internal runtime used to schedule tasks.
// pub mod runner_rt {
//     pub use tokio::spawn;

//     // pub fn spawn<T>(task: T) -> tokio::task::JoinHandle<T::Output>
//     // where
//     //     T: core::future::Future + Send + 'static,
//     //     T::Output: Send + 'static,
//     // {

//     //     tokio::spawn(task)
//     // }

// }
