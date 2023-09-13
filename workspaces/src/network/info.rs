use crate::types::AccountId;

pub struct Info {
    /// Name of the network itself
    pub name: String,
    /// Root Account ID of the network. Mainnet has `near`, testnet has `testnet`.
    pub root_id: AccountId,

    /// Rpc endpoint to point our client to
    pub rpc_url: url::Url,
}
