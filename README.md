# roblox-rs
This is the work in progress rewrite of `roblox-rs` with binding generation and proper tooling.

If you'd like to use the legacy version which contains Roblox bindings, see the [legacy](https://github.com/roblox-rs/roblox-rs/tree/legacy) branch.

## Install roblox-rs
The roblox-rs crate can be installed from `crates.io` or you can use the Git repository.
```toml
[dependencies]
roblox-rs = { version = <LATEST_VERSION> }
roblox-rs = { git = "https://github.com/roblox-rs/bindings.git", branch = "v2" }
```

The roblox-rs-cli must be installed via Git for now.
```bash
cargo install --git https://github.com/roblox-rs/bindings --branch v2 roblox-rs-cli
```
