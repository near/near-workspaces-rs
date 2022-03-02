use near_primitives::views::{BlockHeaderView, BlockView};

use crate::{BlockHeight, CryptoHash};

/// Struct containing information on block coming from the network
pub struct Block {
    header: BlockHeader,
}

impl Block {
    /// The block timestamp in nanoseconds.
    fn timestamp(&self) -> u64 {
        self.header.timestamp_nanosec
    }

    /// Current height of this block.
    fn height(&self) -> BlockHeight {
        self.header.height
    }

    fn hash(&self) -> &CryptoHash {
        &self.header.hash
    }

    fn epoch_id(&self) -> &CryptoHash {
        &self.header.epoch_id
    }
}

/// The block header info. This is a non-exhaustive list of items that
/// could be present in a block header. More can be added in the future.
#[non_exhaustive]
struct BlockHeader {
    height: BlockHeight,
    epoch_id: CryptoHash,
    hash: CryptoHash,
    timestamp_nanosec: u64,
}

impl From<BlockView> for Block {
    fn from(block_view: BlockView) -> Self {
        Self {
            header: block_view.header.into(),
        }
    }
}

impl From<BlockHeaderView> for BlockHeader {
    fn from(header_view: BlockHeaderView) -> Self {
        Self {
            height: header_view.height,
            epoch_id: CryptoHash(header_view.epoch_id.0),
            hash: CryptoHash(header_view.hash.0),
            timestamp_nanosec: header_view.timestamp_nanosec,
        }
    }
}
