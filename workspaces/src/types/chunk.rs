use near_primitives::views::ChunkView;

use crate::types::ShardId;
use crate::CryptoHash;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Chunk {
    pub header: ChunkHeader,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ChunkHeader {
    pub chunk_hash: CryptoHash,
    pub prev_block_hash: CryptoHash,
    pub shard_id: ShardId,
}

impl From<ChunkView> for Chunk {
    fn from(view: ChunkView) -> Self {
        Self {
            header: ChunkHeader {
                chunk_hash: view.header.chunk_hash.into(),
                prev_block_hash: view.header.prev_block_hash.into(),
                shard_id: view.header.shard_id,
            },
        }
    }
}

impl Chunk {
    pub fn hash(&self) -> &CryptoHash {
        &self.header.chunk_hash
    }

    pub fn shard_id(&self) -> ShardId {
        self.header.shard_id
    }
}
