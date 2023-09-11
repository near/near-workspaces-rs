#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let genesis_config = worker.genesis_config().await?;

    // Example output:
    //
    // GenesisConfig GenesisConfig {
    //     protocol_version: 60,
    //     genesis_time: 2023-08-22T10:05:32.129104Z,
    //     chain_id: "test-chain-5oKXo",
    //     genesis_height: 0,
    //     num_block_producer_seats: 50,
    //     num_block_producer_seats_per_shard: [
    //         50,
    //     ],
    //     avg_hidden_validator_seats_per_shard: [
    //         0,
    //     ],
    //     dynamic_resharding: false,
    //     protocol_upgrade_stake_threshold: Ratio {
    //         numer: 4,
    //         denom: 5,
    //     },
    //     epoch_length: 500,
    //     gas_limit: 1000000000000000,
    //     min_gas_price: 100000000,
    //     max_gas_price: 10000000000000000000000,
    //     block_producer_kickout_threshold: 90,
    //     chunk_producer_kickout_threshold: 90,
    //     online_min_threshold: Ratio {
    //         numer: 9,
    //         denom: 10,
    //     },
    //     online_max_threshold: Ratio {
    //         numer: 99,
    //         denom: 100,
    //     },
    //     gas_price_adjustment_rate: Ratio {
    //         numer: 1,
    //         denom: 100,
    //     },
    //     validators: [
    //         AccountInfo {
    //             account_id: AccountId(
    //                 "test.near",
    //             ),
    //             public_key: ed25519:4Q4fpCWcsVFj3WT7xkCt45qwW84hskFB4SRMHAQfuCne,
    //             amount: 50000000000000000000000000000000,
    //         },
    //     ],
    //     transaction_validity_period: 100,
    //     protocol_reward_rate: Ratio {
    //         numer: 1,
    //         denom: 10,
    //     },
    //     max_inflation_rate: Ratio {
    //         numer: 1,
    //         denom: 20,
    //     },
    //     total_supply: 2050000000000000000000000000000000,
    //     num_blocks_per_year: 31536000,
    //     protocol_treasury_account: AccountId(
    //         "test.near",
    //     ),
    //     fishermen_threshold: 10000000000000000000000000,
    //     minimum_stake_divisor: 10,
    //     shard_layout: V0(
    //         ShardLayoutV0 {
    //             num_shards: 1,
    //             version: 0,
    //         },
    //     ),
    //     num_chunk_only_producer_seats: 300,
    //     minimum_validators_per_shard: 1,
    //     max_kickout_stake_perc: 100,
    //     minimum_stake_ratio: Ratio {
    //         numer: 1,
    //         denom: 6250,
    //     },
    //     use_production_config: false,
    // }
    println!("GenesisConfig {:#?}", genesis_config);
    Ok(())
}
