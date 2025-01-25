//! This module is responsible for sharing information between the roblox-rs-macro-expansion and roblox-rs-cli.

use std::io::Cursor;

use shared_context::SharedContext;

pub mod shared_context;

pub fn encode(context: &SharedContext) -> Vec<u8> {
    bincode::serialize(context).expect("failed to serialize shared context")
}

pub fn decode(content: &[u8]) -> SharedContext {
    let mut cursor = Cursor::new(content);
    let mut context = SharedContext::default();

    while let Ok(value) = bincode::deserialize_from::<_, SharedContext>(&mut cursor) {
        context.imports.extend(value.imports);
        context.exports.extend(value.exports);
        context.main_fns.extend(value.main_fns);
    }

    context
}
