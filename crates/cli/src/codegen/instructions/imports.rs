use std::io::{self, Write};

use crate::{
    codegen::{
        macros::{line, list, pull, push, text},
        splat,
        traits::{Instruction, InstructionContext},
    },
    describe::{Describe, Primitive},
};

use super::conversion::{LuauToRust, RustToLuau};

pub struct WasmCreateImport {
    pub export_name: String,
    pub parameters: Vec<Describe>,
    pub output: Describe,
    pub body: Box<dyn Instruction>,
}

impl Instruction for WasmCreateImport {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        ctx.vars.scope();

        let ffi_name = &self.export_name;
        let needs_spill = self.body.get_outputs() > 1;

        let mut parameter_defs = Vec::new();
        let out_param = needs_spill.then(|| ctx.vars.next("ret"));
        parameter_defs.extend(out_param.iter().cloned());

        for (i, param) in self
            .parameters
            .iter()
            .filter(|v| v.value_count() != 0)
            .enumerate()
        {
            let names = splat(param, &format!("arg{i}"));
            parameter_defs.extend_from_slice(&names);

            ctx.inputs.extend(names);
        }

        text!(ctx, "WASM_FUNCS['{ffi_name}'] = function(");
        list!(ctx, parameter_defs);
        push!(ctx, ")");

        self.body.render(ctx)?;

        if let Some(ptr) = out_param {
            let primitives = &self.output.primitive_values();
            ctx.push(ptr);

            WriteMemory { primitives }.render(ctx)?;
        } else {
            text!(ctx, "return ");
            list!(ctx, ctx.pop_many(self.body.get_outputs()));
            line!(ctx);
        }

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

pub struct InvokeLuauFunction {
    pub function_name: String,
    pub parameter_count: usize,
    pub result_count: usize,
}

impl Instruction for InvokeLuauFunction {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let mut outputs = Vec::new();

        if self.result_count > 0 {
            let var_names = ctx.vars.many(self.result_count, "result");

            text!(ctx, "local ");
            list!(ctx, var_names);
            text!(ctx, " = ");

            outputs.extend(var_names);
        }

        text!(ctx, "{}(", self.function_name);
        list!(ctx, ctx.pop_many(self.parameter_count));
        line!(ctx, ")");

        ctx.inputs.extend(outputs);

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        self.parameter_count
    }

    fn get_outputs(&self) -> usize {
        self.result_count
    }
}

/// This is a block that automatically converts the inputs from Rust to Luau, and outputs from Luau to Rust
pub struct ImportBlock {
    pub inputs: Vec<Describe>,
    pub output: Describe,
    pub body: Box<dyn Instruction>,
}

impl Instruction for ImportBlock {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let mut inputs = Vec::new();

        for param in self.inputs.iter().rev() {
            let names = ctx.pop_many(param.value_count());
            inputs.push((param, names));
        }

        for (ty, names) in inputs.into_iter().rev() {
            ctx.inputs.extend(names);

            RustToLuau { ty }.render(ctx)?;
        }

        self.body.render(ctx)?;

        if self.body.get_outputs() != 0 {
            LuauToRust { ty: &self.output }.render(ctx)?;
        }

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        self.inputs.iter().map(|v| v.value_count()).sum()
    }

    fn get_outputs(&self) -> usize {
        self.output.value_count()
    }
}

pub struct WriteMemory<'a> {
    pub primitives: &'a [Primitive],
}

impl Instruction for WriteMemory<'_> {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let mut offset = 0;
        let memory = ctx.pop();
        let ty_exprs = ctx.pop_many(self.primitives.len());

        for (expr, prim) in ty_exprs.iter().zip(self.primitives) {
            let buffer_call = match prim {
                Primitive::F32 => "writef32",
                Primitive::F64 => "writef64",
                Primitive::I32 => "writei32",
                Primitive::U32 => "writeu32",
            };

            line!(
                ctx,
                "buffer.{buffer_call}(MEMORY.data, {memory} + {offset}, {expr})"
            );

            offset += prim.byte_size();
        }

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1 + self.primitives.len()
    }

    fn get_outputs(&self) -> usize {
        0
    }
}
