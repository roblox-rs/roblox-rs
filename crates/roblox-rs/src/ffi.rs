#[link(wasm_import_module = "roblox-rs")]
unsafe extern "C" {
    pub safe fn describe(value: u32);
}
