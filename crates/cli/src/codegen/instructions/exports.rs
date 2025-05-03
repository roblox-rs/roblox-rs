use std::io::{self, Write};

use crate::{
    codegen::{
        instructions::conversion::{LuauToRust, RustToLuau},
        macros::{line, list, pull, push, text},
        traits::{Instruction, InstructionContext},
    },
    describe::Describe,
    iter_ext::IterDoneExt,
};

pub struct WasmCreateExport {
    pub luau_name: String,
    pub parameters: Vec<Describe>,
    pub output_type: Describe,
    pub body: Box<dyn Instruction>,
}

impl Instruction for WasmCreateExport {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        ctx.vars.scope();

        let luau_name = &self.luau_name;
        let parameters = self
            .parameters
            .iter()
            .map(|v| (v, ctx.vars.next("param")))
            .collect::<Vec<_>>();

        text!(ctx, "WASM_EXPORTS[\"{luau_name}\"] = function(");
        list!(ctx, parameters; |v| v.1);
        push!(ctx, ")");

        for (ty, name) in &parameters {
            ctx.push(name);

            LuauToRust { ty }.render(ctx)?;
        }

        self.body.render(ctx)?;

        RustToLuau {
            ty: &self.output_type,
        }
        .render(ctx)?;

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

pub struct InvokeRustFunction {
    pub function_name: String,
    pub parameters: Vec<Describe>,
    pub output_type: Describe,
}

impl Instruction for InvokeRustFunction {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let function_name = &self.function_name;
        let output_size = self.output_type.memory_size();
        let mut inputs = Vec::new();
        let spill_ptr = if self.output_type.value_count() > 1 {
            let var = ctx.vars.next("spill");
            inputs.push(var.clone());

            Some(var)
        } else {
            None
        };

        for ty in &self.parameters {
            inputs.extend(ctx.pop_many(ty.value_count()));
        }

        let output_names = ctx.vars.many(self.output_type.value_count(), "output");

        if self.output_type.value_count() != 0 {
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
            let output_primitives = self.output_type.primitive_values();
            let mut offset = 0;
            for (name, prim) in output_names.iter().zip(output_primitives.iter()) {
                let buffer_name = prim.buffer_name();

                line!(
                    ctx,
                    "local {name} = buffer.read{buffer_name}(MEMORY.data, {spill_ptr} + {offset})"
                );

                offset += prim.byte_size();
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
