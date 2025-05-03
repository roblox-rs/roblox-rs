use std::io::{self, Write};

use crate::codegen::{
    macros::line,
    traits::{Instruction, InstructionContext},
};

const RUNTIME_HEAD: &str = "\
--!native
--!optimize 2
local WASM_CTOR = require(script.Parent.wasm)
local WASM_FUNCS = {{}}
local WASM_EXPORTS = {{}}
local HEAP, HEAP_ID = {{}}, 0
local WASM, MEMORY, WASM_STACK";

const RUNTIME_TAIL: &str = "\
WASM = WASM_CTOR({{ luau = {{ func_list = WASM_FUNCS }} }})
MEMORY = WASM.memory_list.memory
WASM_STACK = WASM.global_list.__stack_pointer";

pub struct CreateRuntimeHeader;

impl Instruction for CreateRuntimeHeader {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        line!(ctx, "{RUNTIME_HEAD}");

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        0
    }

    fn get_outputs(&self) -> usize {
        0
    }
}

pub struct CreateRuntimeTail {
    pub main_names: Vec<String>,
}

impl Instruction for CreateRuntimeTail {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        line!(ctx, "{RUNTIME_TAIL}");

        for name in &self.main_names {
            line!(ctx, "WASM.func_list.{name}()");
        }

        line!(ctx, "return WASM_EXPORTS");

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        0
    }

    fn get_outputs(&self) -> usize {
        0
    }
}
