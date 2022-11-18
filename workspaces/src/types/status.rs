use std::collections::HashMap;

use chrono::DateTime;
use near_account_id::AccountId;
use near_primitives::{types::ShardId, views::StatusResponse};

use crate::{BlockHeight, CryptoHash};

use super::{EpochId, PublicKey};

// exclude status sync info
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Status {
    /// Binary version.
    pub version: Version,
    /// Unique chain id.
    pub chain_id: String,
    /// Currently active protocol version.
    pub protocol_version: u32,
    /// Latest protocol version that this client supports.
    pub latest_protocol_version: u32,
    /// Address for RPC server.  None if node doesn’t have RPC endpoint enabled.
    pub rpc_addr: Option<String>,
    /// Current epoch validators.
    pub validators: Vec<ValidatorInfo>,
    /// Sync status of the node.
    pub sync_info: StatusSyncInfo,
    /// Validator id of the node
    pub validator_account_id: Option<AccountId>,
    /// Public key of the node.
    pub node_key: Option<PublicKey>,
    /// Uptime of the node.
    pub uptime_sec: i64,
    /// Information about last blocks, network, epoch and chain & chunk info.
    pub detailed_debug_status: Option<DetailedDebugStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct Version {
    pub version: String,
    pub build: String,
    pub rustc_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ValidatorInfo {
    pub account_id: AccountId,
    pub is_slashed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct StatusSyncInfo {
    pub latest_block_hash: CryptoHash,
    pub latest_block_height: BlockHeight,
    pub latest_state_root: CryptoHash,
    pub latest_block_time: DateTime<chrono::Utc>,
    pub syncing: bool,
    pub earliest_block_hash: Option<CryptoHash>,
    pub earliest_block_height: Option<BlockHeight>,
    pub earliest_block_time: Option<DateTime<chrono::Utc>>,
    pub epoch_id: Option<EpochId>,
    pub epoch_start_height: Option<BlockHeight>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct DetailedDebugStatus {
    pub network_info: NetworkInfoView,
    pub sync_status: String,
    pub catchup_status: Vec<CatchupStatusView>,
    pub current_head_status: BlockStatusView,
    pub current_header_head_status: BlockStatusView,
    pub block_production_delay_millis: u64,
    pub chain_processing_info: ChainProcessingInfo,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct NetworkInfoView {
    pub peer_max_count: u32,
    pub num_connected_peers: usize,
    pub connected_peers: Vec<PeerInfoView>,
    pub known_producers: Vec<KnownProducerView>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CatchupStatusView {
    // This is the first block of the epoch that we are catching up
    pub sync_block_hash: CryptoHash,
    pub sync_block_height: BlockHeight,
    // Status of all shards that need to sync
    pub shard_sync_status: HashMap<ShardId, String>,
    // Blocks that we need to catchup, if it is empty, it means catching up is done
    pub blocks_to_catchup: Vec<BlockStatusView>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct BlockStatusView {
    pub height: BlockHeight,
    pub hash: CryptoHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ChainProcessingInfo {
    pub num_blocks_in_processing: usize,
    pub num_orphans: usize,
    pub num_blocks_missing_chunks: usize,
    /// contains processing info of recent blocks, ordered by height high to low
    pub blocks_info: Vec<BlockProcessingInfo>,
    /// contains processing info of chunks that we don't know which block it belongs to yet
    pub floating_chunks_info: Vec<ChunkProcessingInfo>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PeerInfoView {
    pub addr: String,
    pub account_id: Option<AccountId>,
    pub height: BlockHeight,
    pub tracked_shards: Vec<ShardId>,
    pub archival: bool,
    pub peer_id: PublicKey,
    pub received_bytes_per_sec: u64,
    pub sent_bytes_per_sec: u64,
    pub last_time_peer_requested_millis: u64,
    pub last_time_received_message_millis: u64,
    pub connection_established_time_millis: u64,
    pub is_outbound_peer: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct KnownProducerView {
    pub account_id: AccountId,
    pub peer_id: PublicKey,
    pub next_hops: Option<Vec<PublicKey>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct BlockProcessingInfo {
    pub height: BlockHeight,
    pub hash: CryptoHash,
    pub received_timestamp: DateTime<chrono::Utc>,
    /// Timestamp when block was received.
    //pub received_timestamp: DateTime<chrono::Utc>,
    /// Time (in ms) between when the block was first received and when it was processed
    pub in_progress_ms: u128,
    /// Time (in ms) that the block spent in the orphan pool. If the block was never put in the
    /// orphan pool, it is None. If the block is still in the orphan pool, it is since the time
    /// it was put into the pool until the current time.
    pub orphaned_ms: Option<u128>,
    /// Time (in ms) that the block spent in the missing chunks pool. If the block was never put in the
    /// missing chunks pool, it is None. If the block is still in the missing chunks pool, it is
    /// since the time it was put into the pool until the current time.
    pub missing_chunks_ms: Option<u128>,
    pub block_status: BlockProcessingStatus,
    /// Only contains new chunks that belong to this block, if the block doesn't produce a new chunk
    /// for a shard, the corresponding item will be None.
    pub chunks_info: Vec<Option<ChunkProcessingInfo>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ChunkProcessingInfo {
    pub height_created: BlockHeight,
    pub shard_id: ShardId,
    pub chunk_hash: CryptoHash,
    pub prev_block_hash: CryptoHash,
    /// Account id of the validator who created this chunk
    /// Theoretically this field should never be None unless there is some database corruption.
    pub created_by: Option<AccountId>,
    pub status: ChunkProcessingStatus,
    /// Timestamp of first time when we request for this chunk.
    pub requested_timestamp: Option<DateTime<chrono::Utc>>,
    /// Timestamp of when the chunk is complete
    pub completed_timestamp: Option<DateTime<chrono::Utc>>,
    /// Time (in millis) that it takes between when the chunk is requested and when it is completed.
    pub request_duration: Option<u64>,
    pub chunk_parts_collection: Vec<PartCollectionInfo>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BlockProcessingStatus {
    Orphan,
    WaitingForChunks,
    InProcessing,
    Processed,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ChunkProcessingStatus {
    NeedToRequest,
    Requested,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct PartCollectionInfo {
    pub part_owner: AccountId,
    // Time when the part is received through any message
    pub received_time: Option<DateTime<chrono::Utc>>,
    // Time when we receive a PartialEncodedChunkForward containing this part
    pub forwarded_received_time: Option<DateTime<chrono::Utc>>,
    // Time when we receive the PartialEncodedChunk message containing this part
    pub chunk_received_time: Option<DateTime<chrono::Utc>>,
}

impl From<StatusResponse> for Status {
    fn from(status: StatusResponse) -> Self {
        Self {
            version: Version {
                version: status.version.version,
                build: status.version.build,
                rustc_version: status.version.rustc_version,
            },
            chain_id: status.chain_id,
            protocol_version: status.protocol_version,
            latest_protocol_version: status.latest_protocol_version,
            rpc_addr: status.rpc_addr,
            validators: status
                .validators
                .into_iter()
                .map(|validator| ValidatorInfo {
                    account_id: validator.account_id,
                    is_slashed: validator.is_slashed,
                })
                .collect(),
            sync_info: StatusSyncInfo {
                latest_block_hash: status.sync_info.latest_block_hash.into(),
                latest_block_height: status.sync_info.latest_block_height,
                latest_state_root: status.sync_info.latest_state_root.into(),
                latest_block_time: status.sync_info.latest_block_time,
                syncing: status.sync_info.syncing,
                earliest_block_hash: status.sync_info.earliest_block_hash.into(),
                earliest_block_height: status.sync_info.earliest_block_height,
                earliest_block_time: status.sync_info.earliest_block_time,
                epoch_id: status.sync_info.epoch_id,
                epoch_start_height: status.sync_info.epoch_start_height,
            },
            validator_account_id: status.validator_account_id,
            node_key: status.node_key,
            uptime_sec: status.uptime_sec,
            detailed_debug_status: status.detailed_debug_status,
        }
    }
}
