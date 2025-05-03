use std::io::{self, Write};

use crate::{
    codegen::{
        instructions::PushConst,
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
            Describe::ExternRef => RustExternRefToLuau.render(ctx),
            Describe::Boolean => RustBooleanToLuau.render(ctx),
            Describe::Option { ty } => RustOptionToLuau { ty: *ty.clone() }.render(ctx),
            Describe::Void => PushConst::new("nil").render(ctx),
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

pub struct RustExternRefToLuau;

impl Instruction for RustExternRefToLuau {
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
