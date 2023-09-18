/// URL to the Pagoda API to use for testnet.
pub const PAGODA_TESTNET_RPC_URL: &str = "https://near-testnet.api.pagoda.co/rpc/v1/";

#[tokio::test]
async fn test_custom_network() -> anyhow::Result<()> {
    if std::env::var("NEAR_RPC_API_KEY").is_err() {
        // skip the test
        return Ok(());
    }

    let worker = workspaces::custom(PAGODA_TESTNET_RPC_URL).await?;
    let res = worker.view_block().await?;

    assert!(res.height() > 0);

    Ok(())
}
