# Sanbox RPC Helpers
This repo hosts a bunch of helper functions for querying into the sandbox directly.

---

**NOTE**
This may be short-lived as these functions will later be incorporated into the more general `near-api-rs`.

---

## Showcase
This showcase will run the [NFT example](https://github.com/near-examples/NFT) right in the sandbox.

### Requirements
Some requirements are needed first:
- [Rust](https://rustup.rs/) installed
- [Near Sandbox](https://github.com/near/sandbox) installed
- [near-cli](https://github.com/near/near-cli) (optional only if we want to view results)

### Spinning up sandbox
Start up the sandbox in a separate process/terminal:
```bash
near-sandbox init  # do this only once
near-sandbox run
```

### Running the Demo
The following command will deploy the NFT example contract to a dev account, initialize it, and mint a single NFT token we can play around with:
```
cargo test test_nft_example -- --nocapture
```

To view our minted NFT in the sandbox:
```
export NEAR_ENV=local
near view $ID nft_metadata
```
where `$ID` should be printed from our test

