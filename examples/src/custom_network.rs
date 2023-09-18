/// URL to the Pagoda API to use for testnet.
pub const PAGODA_TESTNET_RPC_URL: &str = "https://near-testnet.api.pagoda.co/rpc/v1/";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `NEAR_RPC_API_KEY="xxxx" cargo test --package workspaces --test custom_network -- test_custom_network --exact --nocapture`
    if std::env::var("NEAR_RPC_API_KEY").is_err() {
        // skip the test
        println!("NEAR_RPC_API_KEY is not set, skipping the example");
        return Ok(());
    }

    // Reference to what can be called by this network: https://docs.pagoda.co/endpoints
    let worker = workspaces::custom(PAGODA_TESTNET_RPC_URL).await?;
    let res = worker.view_block().await?;

    assert!(res.height() > 0);

    Ok(())
}
