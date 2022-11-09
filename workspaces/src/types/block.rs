use near_account_id::AccountId;
use near_primitives::views::{BlockHeaderView, BlockView};

use crate::types::Balance;
use crate::{BlockHeight, CryptoHash};

/// Struct containing information on block coming from the network
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Block {
    pub author: AccountId,
    pub header: BlockHeader,
}

impl Block {
    /// The block timestamp in nanoseconds.
    pub fn timestamp(&self) -> u64 {
        self.header.timestamp_nanosec
    }

    /// Current height of this block.
    pub fn height(&self) -> BlockHeight {
        self.header.height
    }

    /// The hash of the block itself.
    pub fn hash(&self) -> &CryptoHash {
        &self.header.hash
    }

    /// The id of an epoch this block belongs to.
    pub fn epoch_id(&self) -> &CryptoHash {
        &self.header.epoch_id
    }
}

/// The block header info. This is a non-exhaustive list of items that
/// could be present in a block header. More can be added in the future.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct BlockHeader {
    pub height: BlockHeight,
    pub epoch_id: CryptoHash,
    pub next_epoch_id: CryptoHash,
    pub hash: CryptoHash,
    pub prev_hash: CryptoHash,
    pub timestamp_nanosec: u64,
    pub random_value: CryptoHash,
    pub gas_price: Balance,
    pub block_ordinal: Option<u64>,
    pub total_supply: Balance,
    pub last_final_block: CryptoHash,
    pub last_ds_final_block: CryptoHash,
    pub next_bp_hash: CryptoHash,
    pub latest_protocol_version: u32,

    pub prev_state_root: CryptoHash,
    pub chunk_receipts_root: CryptoHash,
    pub chunk_headers_root: CryptoHash,
    pub chunk_tx_root: CryptoHash,
    pub outcome_root: CryptoHash,
    pub challenges_root: CryptoHash,
    pub block_merkle_root: CryptoHash,
}

impl From<BlockView> for Block {
    fn from(view: BlockView) -> Self {
        Self {
            author: view.author,
            header: view.header.into(),
        }
    }
}

impl From<BlockHeaderView> for BlockHeader {
    fn from(header_view: BlockHeaderView) -> Self {
        Self {
            height: header_view.height,
            epoch_id: header_view.epoch_id.into(),
            next_epoch_id: header_view.next_epoch_id.into(),
            hash: header_view.hash.into(),
            prev_hash: header_view.prev_hash.into(),
            timestamp_nanosec: header_view.timestamp_nanosec,
            random_value: header_view.random_value.into(),
            gas_price: header_view.gas_price,
            block_ordinal: header_view.block_ordinal,
            total_supply: header_view.total_supply,
            last_final_block: header_view.last_final_block.into(),
            last_ds_final_block: header_view.last_ds_final_block.into(),
            next_bp_hash: header_view.next_bp_hash.into(),
            latest_protocol_version: header_view.latest_protocol_version,

            prev_state_root: header_view.prev_state_root.into(),
            chunk_receipts_root: header_view.chunk_receipts_root.into(),
            chunk_headers_root: header_view.chunk_headers_root.into(),
            chunk_tx_root: header_view.chunk_tx_root.into(),
            outcome_root: header_view.outcome_root.into(),
            challenges_root: header_view.challenges_root.into(),
            block_merkle_root: header_view.block_merkle_root.into(),
        }
    }
}
