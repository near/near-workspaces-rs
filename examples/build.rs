fn main() -> anyhow::Result<()> {
    workspaces::near_abi_client::Generator::new("src/gen".into())
        .file("res/adder.json")
        .generate()?;
    Ok(())
}
