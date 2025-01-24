use crate::describe::Describe;

pub mod ast;
pub mod traits;

pub fn splat(ty: &Describe, prefix: &str) -> Vec<String> {
    (1..=ty.value_count())
        .map(|v| format!("{prefix}_{v}"))
        .collect()
}
