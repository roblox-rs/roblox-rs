use std::{
    cell::Cell,
    hash::{DefaultHasher, Hasher},
};

thread_local! {
    static SYMBOL_ID: Cell<u64> = const { Cell::new(0) };
}

/// Generates a consistent, but reasonably random, hash and prefixes it with the specified string.
pub fn new_symbol_name(name: impl AsRef<str>) -> String {
    let mut hasher = DefaultHasher::new();
    hasher.write(env!("CARGO_PKG_NAME").as_bytes());
    hasher.write(env!("CARGO_PKG_VERSION").as_bytes());
    hasher.write_u64(SYMBOL_ID.replace(SYMBOL_ID.get() + 1));

    let name = name.as_ref();
    let symbol_id = hasher.finish();

    format!("{name}_{symbol_id:x}")
}
