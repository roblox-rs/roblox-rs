use std::io::{self, Write};

use crate::{
    codegen::{
        instructions::WriteMemory,
        macros::{line, pull, push},
        traits::{Instruction, InstructionContext},
    },
    describe::Describe,
};

/// This instruction exists as a matching utility
pub struct LuauToRust<'a> {
    pub ty: &'a Describe,
}

impl Instruction for LuauToRust<'_> {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        match self.ty {
            // These don't require conversion, so just pass the inputs along.
            Describe::F32
            | Describe::F64
            | Describe::U8
            | Describe::U16
            | Describe::U32
            | Describe::I8
            | Describe::I16
            | Describe::I32 => Ok(()),
            Describe::ExternRef => LuauExternRefToRust.render(ctx),
            Describe::Boolean => LuauBooleanToRust.render(ctx),
            Describe::String => LuauStringToRust.render(ctx),
            Describe::Option { ty } => LuauOptionToRust { ty: *ty.clone() }.render(ctx),
            Describe::Vector { ty } => LuauVecToRust { ty: *ty.clone() }.render(ctx),
            Describe::Void => {
                ctx.pop();
                Ok(())
            }
            _ => unimplemented!(),
        }
    }

    fn get_inputs(&self) -> usize {
        1
    }

    fn get_outputs(&self) -> usize {
        self.ty.value_count()
    }
}

pub struct LuauBooleanToRust;

impl Instruction for LuauBooleanToRust {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let value = ctx.pop();
        ctx.push(format!("if {value} then 1 else 0"));

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct LuauStringToRust;

impl Instruction for LuauStringToRust {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let value = ctx.pop_complex()?;
        let result = ctx.vars.next("string");
        let alloc = ctx.intrinsics.get("alloc");

        line!(ctx, "local {result} = {alloc}(#{value}, 1)");
        line!(ctx, "buffer.writestring(MEMORY.data, {result}, {value})");

        ctx.push(result);
        ctx.push(format!("#{value}"));

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1
    }

    fn get_outputs(&self) -> usize {
        2
    }
}

pub struct LuauVecToRust {
    ty: Describe,
}

impl Instruction for LuauVecToRust {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let vec = ctx.pop_complex()?;
        let var = ctx.vars.next("vec");
        let alloc = ctx.intrinsics.get("alloc");
        let size = self.ty.memory_size();
        let primitives = &self.ty.primitive_values();

        line!(ctx, "local {var} = {alloc}(#{vec} * {size}, 4)");
        push!(ctx, "for i, v in ipairs({vec}) do");

        ctx.push("v");
        LuauToRust { ty: &self.ty }.render(ctx)?;

        ctx.push(format!("{var} + (i - 1) * {size}"));
        WriteMemory { primitives }.render(ctx)?;

        pull!(ctx, "end");

        ctx.push(var);
        ctx.push(format!("#{vec}"));

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1
    }

    fn get_outputs(&self) -> usize {
        2
    }
}

pub struct LuauOptionToRust {
    ty: Describe,
}

impl Instruction for LuauOptionToRust {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let value = ctx.pop_complex()?;
        let existance_name = ctx.vars.next("option");
        let output_names = ctx.vars.many(self.ty.value_count(), "option");

        for name in [&existance_name].into_iter().chain(output_names.iter()) {
            line!(ctx, "local {name} = 0");
        }

        push!(ctx, "if {value} ~= nil then");
        line!(ctx, "{existance_name} = 1");

        ctx.push(value);
        LuauToRust { ty: &self.ty }.render(ctx)?;

        let output_exprs = ctx.pop_many(self.ty.value_count());
        for (output_name, value) in output_names.iter().zip(output_exprs.iter()) {
            line!(ctx, "{output_name} = {value}");
        }

        pull!(ctx, "end");

        ctx.push(existance_name);
        ctx.inputs.extend(output_names);

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1
    }

    fn get_outputs(&self) -> usize {
        1 + self.ty.value_count()
    }
}

pub struct LuauExternRefToRust;

impl Instruction for LuauExternRefToRust {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let heap = ctx.vars.next("heap");
        let value = ctx.pop();

        line!(ctx, "HEAP_ID += 1");
        line!(ctx, "local {heap} = HEAP_ID");
        line!(ctx, "HEAP[HEAP_ID] = {value}");

        ctx.push(heap);

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1
    }

    fn get_outputs(&self) -> usize {
        1
    }
}
