use std::io::{self, Write};

use crate::{
    codegen::{
        instructions::conversion::{LuauToRust, RustToLuau},
        macros::{line, list, pull, push, text},
        traits::{Instruction, InstructionContext},
    },
    describe::{Describe, Primitive},
};

pub struct WasmCreateExport {
    pub luau_name: String,
    pub parameters: Vec<Describe>,
    pub body: Box<dyn Instruction>,
}

impl Instruction for WasmCreateExport {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        ctx.vars.scope();

        let luau_name = &self.luau_name;
        let parameters = ctx.vars.many(self.parameters.len(), "param");

        text!(ctx, "WASM_EXPORTS[\"{luau_name}\"] = function(");
        list!(ctx, parameters);
        push!(ctx, ")");

        ctx.inputs.extend(parameters);

        self.body.render(ctx)?;

        let value = ctx.pop();
        line!(ctx, "return {value}");
        pull!(ctx, "end");

        ctx.vars.unscope();

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        0
    }

    fn get_outputs(&self) -> usize {
        0
    }
}

/// This is a block that automatically converts the inputs from Luau to Rust, and outputs from Rust to Luau
pub struct ExportBlock {
    pub inputs: Vec<Describe>,
    pub output: Describe,
    pub body: Box<dyn Instruction>,
}

impl Instruction for ExportBlock {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let inputs = ctx.pop_many(self.inputs.len());

        for (param, ty) in inputs.iter().zip(self.inputs.iter()) {
            ctx.push(param);

            LuauToRust { ty }.render(ctx)?;
        }

        self.body.render(ctx)?;

        if self.body.get_outputs() != 0 {
            RustToLuau { ty: &self.output }.render(ctx)?;
        }

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        self.inputs.len()
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct InvokeRustFunction {
    pub function_name: String,
    pub parameters: Vec<Describe>,
    pub output_type: Describe,
}

impl Instruction for InvokeRustFunction {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let function_name = &self.function_name;
        let output_size = self.output_type.memory_size();
        let output_count = self.output_type.value_count();
        let mut inputs = Vec::new();
        let spill_ptr = if output_count > 1 {
            let var = ctx.vars.next("spill");
            inputs.push(var.clone());

            Some(var)
        } else {
            None
        };

        let mut parameter_inputs = Vec::new();
        for ty in self.parameters.iter().rev() {
            parameter_inputs.push(ctx.pop_many(ty.value_count()));
        }

        for param_input in parameter_inputs.into_iter().rev() {
            inputs.extend(param_input);
        }

        let output_names = ctx.vars.many(output_count, "output");
        if output_count != 0 {
            let output_names_sep = output_names.join(", ");
            ctx.inputs.extend(output_names.clone());

            if let Some(spill_ptr) = &spill_ptr {
                line!(ctx, "local {spill_ptr} = WASM_STACK.value - {output_size}");
                line!(ctx, "WASM_STACK.value = {spill_ptr}");
            } else {
                text!(ctx, "local {output_names_sep} = ");
            }
        }

        line!(ctx, "WASM.func_list.{function_name}({})", inputs.join(", "));

        if let Some(spill_ptr) = &spill_ptr {
            let primitives = &self.output_type.primitive_values();
            ctx.push(spill_ptr);

            PullMemory { primitives }.render(ctx)?;

            for (name, expr) in output_names.iter().zip(ctx.pop_many(output_count)) {
                line!(ctx, "local {name} = {expr}");
            }

            line!(ctx, "WASM_STACK.value = {spill_ptr} + {output_size}");
        }

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        self.parameters.iter().map(|v| v.value_count()).sum()
    }

    fn get_outputs(&self) -> usize {
        self.output_type.value_count()
    }
}

pub struct PullMemory<'a> {
    pub primitives: &'a [Primitive],
}

impl Instruction for PullMemory<'_> {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let mut ptr = ctx.pop();

        if self.primitives.len() > 1 {
            ptr = ctx.prereq_complex(ptr)?;
        }

        let mut offset = 0;
        for prim in self.primitives {
            let aligned_offset = prim.next_align(offset);
            let buffer_name = prim.buffer_name();
            let expr = format!("buffer.read{buffer_name}(MEMORY.data, {ptr} + {aligned_offset})");

            ctx.push(expr);

            offset = aligned_offset + prim.byte_size();
        }

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1
    }

    fn get_outputs(&self) -> usize {
        self.primitives.len()
    }
}
