pub mod conversion;
pub mod exports;
pub mod headers;
pub mod imports;

pub use exports::*;
pub use imports::*;

use std::io;

use super::traits::{Instruction, InstructionContext};

pub struct PushConst {
    pub value: String,
}

impl PushConst {
    pub fn new(value: impl Into<String>) -> Self {
        PushConst {
            value: value.into(),
        }
    }
}

impl Instruction for PushConst {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        ctx.push(&self.value);
        Ok(())
    }

    fn get_inputs(&self) -> usize {
        0
    }

    fn get_outputs(&self) -> usize {
        1
    }
}
