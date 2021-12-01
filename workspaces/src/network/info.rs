use std::path::PathBuf;

use crate::types::AccountId;

pub struct Info {
    /// Name of the network itself
    pub name: String,
    /// Root Account ID of the network. Mainnet has `near`, testnet has `testnet`.
    pub root_id: AccountId,
    /// Path to the keystore directory
    pub keystore_path: PathBuf,

    // TODO: change return type to Url instead of String
    /// Rpc endpoint to point our client to
    pub rpc_url: String,
}
