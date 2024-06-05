pub type Pair = Vec<i64>;
pub struct AbiClient {
    pub contract: near_workspaces::Contract,
}
impl AbiClient {
    pub async fn add(&self, a: Pair, b: Pair) -> anyhow::Result<Pair> {
        let result = self.contract.call("add").args_json([a, b]).view().await?;
        Ok(result.json::<Pair>()?)
    }
}
