use near_account_id::AccountId;
use near_primitives::views::ChunkView;

use crate::types::{Balance, Gas, ShardId};
use crate::{BlockHeight, CryptoHash};

// Chunk object associated to a chunk on chain. This provides info about what
// current state of a chunk is like.
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct Chunk {
    pub author: AccountId,
    pub header: ChunkHeader,
}

/// The header belonging to a [`Chunk`]. This is a non-exhaustive list of
/// members belonging to a Chunk, where newer fields can be added in the future.
///
/// NOTE: validator_proposals have been omitted for now. If you need it, submit a ticket to:
/// https://github.com/near/workspaces-rs/issues
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct ChunkHeader {
    pub chunk_hash: CryptoHash,
    pub prev_block_hash: CryptoHash,
    pub height_created: BlockHeight,
    pub height_included: BlockHeight,
    pub shard_id: ShardId,
    pub gas_used: Gas,
    pub gas_limit: Gas,
    pub balance_burnt: Balance,

    pub tx_root: CryptoHash,
    pub outcome_root: CryptoHash,
    pub prev_state_root: CryptoHash,
    pub outgoing_receipts_root: CryptoHash,
    pub encoded_merkle_root: CryptoHash,
    pub encoded_length: u64,
}

impl From<ChunkView> for Chunk {
    fn from(view: ChunkView) -> Self {
        Self {
            author: view.author,
            header: ChunkHeader {
                chunk_hash: view.header.chunk_hash.into(),
                prev_block_hash: view.header.prev_block_hash.into(),
                height_created: view.header.height_created,
                height_included: view.header.height_included,
                shard_id: view.header.shard_id,
                gas_used: view.header.gas_used,
                gas_limit: view.header.gas_limit,
                balance_burnt: view.header.balance_burnt,

                tx_root: view.header.tx_root.into(),
                outcome_root: view.header.outcome_root.into(),
                prev_state_root: view.header.prev_state_root.into(),
                outgoing_receipts_root: view.header.outgoing_receipts_root.into(),
                encoded_merkle_root: view.header.encoded_merkle_root.into(),
                encoded_length: view.header.encoded_length,
            },
        }
    }
}

impl Chunk {
    /// The author's [`AccountId`] relating to the creation of this chunk.
    pub fn author(&self) -> &AccountId {
        &self.author
    }

    /// The hash of the chunk itself.
    pub fn hash(&self) -> &CryptoHash {
        &self.header.chunk_hash
    }

    /// Which specific shard this chunk belongs to.
    pub fn shard_id(&self) -> ShardId {
        self.header.shard_id
    }
}
