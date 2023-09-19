#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let res = worker.status().await?;

    // status: StatusResponse {
    //     version: Version {
    //         version: "trunk",
    //         build: "eb2bbe1",
    //         rustc_version: "1.72.0",
    //     },
    //     chain_id: "test-chain-vIC0E",
    //     protocol_version: 63,
    //     latest_protocol_version: 63,
    //     rpc_addr: Some(
    //         "0.0.0.0:3030",
    //     ),
    //     validators: [
    //         ValidatorInfo {
    //             account_id: AccountId(
    //                 "test.near",
    //             ),
    //             is_slashed: false,
    //         },
    //     ],
    //     sync_info: StatusSyncInfo {
    //         latest_block_hash: GunSGsMD8fEmxsoyzdUGWBE4AiCUsBEefzxQJYMPdZoD,
    //         latest_block_height: 0,
    //         latest_state_root: 2tKZ7u2YU5GihxRveb2YMg5oxHBnCxNqgooUKfj9XSzh,
    //         latest_block_time: 2023-09-19T05:06:44.748482Z,
    //         syncing: false,
    //         earliest_block_hash: Some(
    //             GunSGsMD8fEmxsoyzdUGWBE4AiCUsBEefzxQJYMPdZoD,
    //         ),
    //         earliest_block_height: Some(
    //             0,
    //         ),
    //         earliest_block_time: Some(
    //             2023-09-19T05:06:44.748482Z,
    //         ),
    //         epoch_id: Some(
    //             EpochId(
    //                 11111111111111111111111111111111,
    //             ),
    //         ),
    //         epoch_start_height: Some(
    //             0,
    //         ),
    //     },
    //     validator_account_id: Some(
    //         AccountId(
    //             "test.near",
    //         ),
    //     ),
    //     validator_public_key: Some(
    //         ed25519:FHvRfJv7WYoaQVSQD3AES98rTJMyk5wKYPFuLJKXb3nx,
    //     ),
    //     node_public_key: ed25519:7gUkJ6EQvSZmRp98hS5mUwojwU8fqQxHjrGcpsfn88um,
    //     node_key: Some(
    //         ed25519:FHvRfJv7WYoaQVSQD3AES98rTJMyk5wKYPFuLJKXb3nx,
    //     ),
    //     uptime_sec: 0,
    //     detailed_debug_status: None,
    // }
    println!("status: {res:#?}");
    Ok(())
}
