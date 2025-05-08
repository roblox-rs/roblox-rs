use std::io::{self, Write};

use crate::{
    codegen::{
        instructions::{PullMemory, PushConst},
        macros::{line, pull, push},
        traits::{Instruction, InstructionContext},
    },
    describe::Describe,
};

/// This instruction exists as a matching utility
pub struct RustToLuau<'a> {
    pub ty: &'a Describe,
}

impl Instruction for RustToLuau<'_> {
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
            Describe::ExternRef => RustOwnedExternRefToLuau.render(ctx),
            Describe::Boolean => RustBooleanToLuau.render(ctx),
            Describe::Option { ty } => RustOptionToLuau { ty: *ty.clone() }.render(ctx),
            Describe::Vector { ty } => RustVectorToLuau { ty: *ty.clone() }.render(ctx),
            Describe::Void => PushConst::new("nil").render(ctx),
            Describe::String => RustOwnedStringToLuau.render(ctx),
            Describe::Ref { ty } => RustRefToLuau { ty }.render(ctx),
            _ => unimplemented!(),
        }
    }

    fn get_inputs(&self) -> usize {
        self.ty.value_count()
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct RustRefToLuau<'a> {
    ty: &'a Describe,
}

impl Instruction for RustRefToLuau<'_> {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        match &self.ty {
            Describe::String => RustRefStringToLuau.render(ctx),
            Describe::Slice { ty } => RustSliceToLuau { ty: *ty.clone() }.render(ctx),
            Describe::ExternRef => RustRefExternRefToLuau.render(ctx),
            ty => {
                unimplemented!("invalid rust reference type: {ty:?}");
            }
        }
    }

    fn get_inputs(&self) -> usize {
        self.ty.value_count()
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct RustSliceToLuau {
    ty: Describe,
}

impl Instruction for RustSliceToLuau {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        // currently only WasmPrimitive slices are accepted
        let [addr, len] = ctx.pop_array();
        let result_name = ctx.vars.next("slice");
        let primitive = self.ty.primitive_values()[0];
        let buffer = primitive.buffer_name();
        let size = primitive.byte_size();
        let len = ctx.prereq_complex(len)?;

        line!(ctx, "local {result_name} = table.create({len})");
        push!(ctx, "for i = 1, {len} do");
        line!(ctx, "table.insert({result_name}, buffer.read{buffer}(MEMORY.data, {addr} + (i - 1) * {size}))");
        pull!(ctx, "end");

        ctx.push(result_name);

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        2
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct RustVectorToLuau {
    ty: Describe,
}

impl Instruction for RustVectorToLuau {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let [addr, len] = ctx.pop_array();
        let result_name = ctx.vars.next("vector");
        let free = ctx.intrinsics.get("free");
        let ty_size = self.ty.memory_size();
        let primitives = &self.ty.primitive_values();

        line!(ctx, "local {result_name} = table.create({len})");
        push!(ctx, "for i = 1, {len} do");

        ctx.push(format!("{addr} + (i - 1) * {ty_size}"));
        PullMemory { primitives }.render(ctx)?;
        RustToLuau { ty: &self.ty }.render(ctx)?;

        let value = ctx.pop();
        line!(ctx, "table.insert({result_name}, {value})");
        pull!(ctx, "end");

        // TODO: is this `free` sound? should we use `memory_size` or `byte_size` for the length?
        line!(ctx, "WASM.func_list.{free}({addr}, {len} * {ty_size}, 4)");

        ctx.push(result_name);

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        2
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct RustRefStringToLuau;

impl Instruction for RustRefStringToLuau {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let [addr, len] = ctx.pop_array();

        ctx.push(format!("buffer.readstring(MEMORY.data, {addr}, {len})"));

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        2
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct RustOwnedStringToLuau;

impl Instruction for RustOwnedStringToLuau {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let [addr, len] = ctx.pop_array();
        let addr = ctx.prereq_complex(addr)?;
        let len = ctx.prereq_complex(len)?;
        let var = ctx.vars.next("string");
        let free = ctx.intrinsics.get("free");

        ctx.push(&addr);
        ctx.push(&len);
        RustRefStringToLuau.render(ctx)?;

        let read_expr = ctx.pop();
        line!(ctx, "local {var} = {read_expr}");
        line!(ctx, "WASM.func_list.{free}({addr}, {len}, 1)");

        ctx.push(var);

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        2
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct RustBooleanToLuau;

impl Instruction for RustBooleanToLuau {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let value = ctx.pop();
        ctx.push(format!("{value} ~= 0"));
        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct RustOptionToLuau {
    ty: Describe,
}

impl Instruction for RustOptionToLuau {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let existance = ctx.inputs[ctx.inputs.len() - self.get_inputs()].clone();
        let output_name = ctx.vars.next("optional");

        line!(ctx, "local {output_name}");
        push!(ctx, "if {existance} == 1 then");

        RustToLuau { ty: &self.ty }.render(ctx)?;

        let value = ctx.pop();
        line!(ctx, "{output_name} = {value}");
        pull!(ctx, "end");

        // Pop the existance flag off, since we couldn't pop it earlier.
        ctx.pop();
        ctx.push(output_name);
        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1 + self.ty.value_count()
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct RustOwnedExternRefToLuau;

impl Instruction for RustOwnedExternRefToLuau {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let value = ctx.pop_complex()?;
        let value_name = ctx.vars.next("value");

        line!(ctx, "local {value_name} = HEAP[{value}]");
        line!(ctx, "HEAP[{value}] = nil");

        ctx.push(value_name);

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1
    }

    fn get_outputs(&self) -> usize {
        1
    }
}

pub struct RustRefExternRefToLuau;

impl Instruction for RustRefExternRefToLuau {
    fn render(&self, ctx: &mut InstructionContext) -> io::Result<()> {
        let value = ctx.pop();

        ctx.push(format!("HEAP[{value}]"));

        Ok(())
    }

    fn get_inputs(&self) -> usize {
        1
    }

    fn get_outputs(&self) -> usize {
        1
    }
}
