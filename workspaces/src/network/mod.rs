//! All builtin network types and traits.
//!
//! Currently the builtin network types are [`Mainnet`], [`Testnet`], and [`Sandbox`].

mod betanet;
mod info;
mod mainnet;
mod sandbox;
mod server;
mod testnet;
pub(crate) mod variants;

pub(crate) use sandbox::SandboxPatchAcessKeyBuilder; //not needed directly outside of the crate
pub(crate) use sandbox::SandboxPatchStateAccountBuilder; //not needed directly outside of the crate
pub(crate) use sandbox::SandboxPatchStateBuilder; //not needed directly outside of the crate
pub(crate) use variants::DEV_ACCOUNT_SEED;

pub use betanet::Betanet;
pub use info::Info;
pub use mainnet::Mainnet;
pub use sandbox::Sandbox;
pub use testnet::Testnet;
pub use variants::{
    AllowDevAccountCreation, DevAccountDeployer, NetworkClient, NetworkInfo, TopLevelAccountCreator,
};
