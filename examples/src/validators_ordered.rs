#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let validators = worker.validators_ordered(None).await?;

    // Example output:
    //
    // Validators [
    //     V1(
    //         ValidatorStakeViewV1 {
    //             account_id: AccountId(
    //                 "test.near",
    //             ),
    //             public_key: ed25519:HguH1hFyR4voJUomR67QqxACtSo3sYTMXtJ9oSzFSoYy,
    //             stake: 50000000000000000000000000000000,
    //         },
    //     ),
    // ]
    println!("Validators {validators:#?}");
    Ok(())
}
