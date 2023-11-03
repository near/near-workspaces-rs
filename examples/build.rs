fn main() -> anyhow::Result<()> {
    near_workspaces::near_abi_client::Generator::new("src/gen".into())
        .file("res/adder.json")
        .generate()?;
    Ok(())
}
