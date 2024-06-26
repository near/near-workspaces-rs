# Utilizing custom sandbox node

This example will show us how to spin up a sandbox node of our own choosing. Follow the guide in <https://github.com/near/sandbox> to download it. This is mainly needed if a user wants to manage their own node and/or not require each test to spin up a new node each time.

Then initialize the chain via `init` and run it:

```sh
near-sandbox --home ${MY_HOME_DIRECTORY} init
near-sandbox --home ${MY_HOME_DIRECTORY} run
```

This will launch the chain onto `localhost:3030` by default. The `${MY_HOME_DIRECTORY}` is a path of our choosing here and this will be needed when running the workspaces code later on. In the following example, we had it set to `/home/user/.near-sandbox-home`.

In workspaces, to connect to our manually launched node, all we have to do is add a few additional parameters to `workspaces::sandbox()`:

```rs
#[tokio::main]
fn main() {
    let worker = workspaces::sandbox()
        .rpc_addr("http://localhost:3030")
        .validator_key(workspaces::network::ValidatorKey::HomeDir(
          "/.near/validator_key.json".into(),
        ))
        .await?;

    Ok(())
}
```

Then afterwards, we can continue performing our tests as we normally would if workspaces has spawned its own sandbox process.
