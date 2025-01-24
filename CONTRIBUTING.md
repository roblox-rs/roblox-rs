# Publishing
Publishing depends on [cargo-release](https://github.com/crate-ci/cargo-release) which should be installed beforehand.

```bash
# roblox-rs-cli can't be published to crates.io until Wasynth releases.
cargo release \
	-p roblox-rs \
	-p roblox-rs-macro-expansion \
	-p roblox-rs-macro-definitions \
	-p roblox-rs-shared-context
```
