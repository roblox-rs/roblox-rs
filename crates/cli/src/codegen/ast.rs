#![allow(clippy::comparison_chain)]

use std::io::{self, Write};

use crate::{
    describe::{Describe, Primitive},
    iter_ext::IterDoneExt,
};

use super::{
    splat,
    traits::{Render, RenderContext},
};

pub struct RuntimeHeader;

impl Render for RuntimeHeader {
    fn render(&self, ctx: &mut RenderContext) -> io::Result<()> {
        writeln!(ctx, "--!native")?;
        writeln!(ctx, "--!optimize 2")?;
        writeln!(ctx, "local WASM_CTOR = require(script.Parent.wasm)")?;
        writeln!(ctx, "local WASM_FUNCS = {{}}")?;
        writeln!(ctx, "local WASM_EXPORTS = {{}}")?;
        writeln!(ctx, "local HEAP, HEAP_ID = {{}}, 0")?;
        writeln!(ctx, "local WASM, MEMORY, WASM_STACK")?;

        // heap intrinsics
        writeln!(ctx, "local function HEAP_TAKE(id)")?;
        writeln!(ctx, "\tlocal value = HEAP[id]")?;
        writeln!(ctx, "\tHEAP[id] = nil")?;
        writeln!(ctx, "\treturn value")?;
        writeln!(ctx, "end")?;

        Ok(())
    }
}

pub struct RuntimeTail {
    pub main_names: Vec<String>,
}

impl Render for RuntimeTail {
    fn render(&self, ctx: &mut RenderContext) -> io::Result<()> {
        writeln!(
            ctx,
            "WASM = WASM_CTOR({{ luau = {{ func_list = WASM_FUNCS }} }})"
        )?;

        writeln!(ctx, "MEMORY = WASM.memory_list.memory")?;
        writeln!(ctx, "WASM_STACK = WASM.global_list.__stack_pointer")?;

        for name in &self.main_names {
            writeln!(ctx, "WASM.func_list.{name}()")?;
        }

        writeln!(ctx, "return WASM_EXPORTS")?;

        Ok(())
    }
}

pub struct ImportFn {
    pub luau_name: String,
    pub export_name: String,
    pub parameters: Vec<InputFnParameter>,
    pub output_type: Describe,
}

impl Render for ImportFn {
    fn render(&self, ctx: &mut RenderContext) -> io::Result<()> {
        let ffi_name = &self.export_name;
        let needs_spill = self.output_type.value_count() > 1;

        write!(ctx, "WASM_FUNCS['{ffi_name}'] = function(")?;

        // In the future, we'll force multivalues but this will do for now.
        let out_param = needs_spill.then(|| InputFnParameter {
            names: vec!["out_ptr".to_string()],
            ty: Describe::U32,
        });

        for (done, param) in out_param.iter().chain(self.parameters.iter()).until_done() {
            ctx.render(param)?;

            if param.ty.value_count() > 0 && !done {
                write!(ctx, ", ")?;
            }
        }

        writeln!(ctx, ")")?;

        ctx.up();

        if !matches!(self.output_type, Describe::Void) {
            write!(ctx, "local result = ")?;
        }

        write!(ctx, "{}(", self.luau_name)?;

        for (done, param) in self.parameters.iter().until_done() {
            AbiToLuauConversion {
                names: &param.names,
                ty: &param.ty,
            }
            .render(ctx)?;

            if !done {
                write!(ctx, ", ")?;
            }
        }

        writeln!(ctx, ")")?;

        if !matches!(self.output_type, Describe::Void) {
            let output_names = splat(&self.output_type, "abi");
            let (prereq, expr) = ctx.render_expr(LuauToAbiConversion {
                name: "result",
                out_names: &output_names,
                ty: &self.output_type,
            })?;

            let mut output_exprs = Vec::new();
            if let Some(expr) = expr {
                output_exprs.push(expr);
            } else {
                for name in output_names {
                    writeln!(ctx, "local {name} = 0")?;
                    output_exprs.push(name);
                }
            }

            ctx.write_all(&prereq)?;

            if needs_spill {
                let mut offset = 0;

                for (name, prim) in output_exprs.iter().zip(self.output_type.primitive_values()) {
                    let buffer_call = match prim {
                        Primitive::F32 => "writef32",
                        Primitive::F64 => "writef64",
                        Primitive::I32 => "writei32",
                        Primitive::U32 => "writeu32",
                    };

                    writeln!(
                        ctx,
                        "buffer.{buffer_call}(MEMORY.data, out_ptr + {offset}, {name})"
                    )?;

                    offset += prim.byte_size();
                }
            } else {
                writeln!(ctx, "return {}", output_exprs.join(", "))?;
            }
        }

        ctx.down();

        writeln!(ctx, "end")?;

        Ok(())
    }
}

pub struct ExportFn {
    pub export_name: String,
    pub luau_name: String,
    pub parameters: Vec<ExportFnParameter>,
    pub output_type: Describe,
}

impl Render for ExportFn {
    fn render(&self, ctx: &mut RenderContext) -> io::Result<()> {
        write!(ctx, "WASM_EXPORTS[\"{}\"] = function(", self.luau_name)?;

        for (done, param) in self.parameters.iter().until_done() {
            write!(ctx, "{}", param.name)?;

            if param.ty.value_count() > 0 && !done {
                write!(ctx, ", ")?;
            }
        }

        writeln!(ctx, ")")?;

        ctx.up();

        let mut param_exprs = Vec::new();

        for param in &self.parameters {
            let param_names = splat(&param.ty, &param.name);
            let (prereq, expr) = ctx.render_expr(LuauToAbiConversion {
                name: &param.name,
                out_names: &param_names,
                ty: &param.ty,
            })?;

            if let Some(expr) = expr {
                param_exprs.push(expr);
            } else {
                for name in param_names {
                    writeln!(ctx, "local {name} = 0")?;
                    param_exprs.push(name);
                }
            }

            ctx.write_all(&prereq)?;
        }

        let output_names = splat(&self.output_type, "result");
        let output_size = self.output_type.memory_size();
        let output_prims = self.output_type.primitive_values();
        if output_names.len() == 1 {
            // A single return slot, no spilling
            write!(ctx, "local {} = ", output_names[0])?;
        } else if output_names.len() > 1 {
            // Multiple return slots, need to allocate stack space
            writeln!(ctx, "local _STACK = WASM_STACK.value - {output_size}")?;
            writeln!(ctx, "WASM_STACK.value = _STACK")?;
            param_exprs.insert(0, "_STACK".to_string());
        }

        let export_name = &self.export_name;
        let formatted_exprs = param_exprs.join(", ");
        writeln!(ctx, "WASM.func_list.{export_name}({formatted_exprs})")?;

        if output_names.len() > 1 {
            let mut offset = 0;
            for (name, prim) in output_names.iter().zip(output_prims.iter()) {
                let buffer_name = prim.buffer_name();

                writeln!(
                    ctx,
                    "local {name} = buffer.read{buffer_name}(MEMORY.data, _STACK + {offset})"
                )?;

                offset += prim.byte_size();
            }
            writeln!(ctx, "WASM_STACK.value = _STACK + {output_size}")?;
        }

        if !output_names.is_empty() {
            write!(ctx, "return ")?;

            ctx.render(AbiToLuauConversion {
                names: &output_names,
                ty: &self.output_type,
            })?;

            writeln!(ctx)?;
        }

        ctx.down();
        writeln!(ctx, "end")?;

        Ok(())
    }
}

pub struct ExportFnParameter {
    pub name: String,
    pub ty: Describe,
}

pub struct InputFnParameter {
    pub names: Vec<String>,
    pub ty: Describe,
}

impl Render for InputFnParameter {
    fn render(&self, context: &mut RenderContext) -> io::Result<()> {
        for (done, name) in self.names.iter().until_done() {
            write!(context, "{name}: number")?;

            if !done {
                write!(context, ", ")?;
            }
        }

        Ok(())
    }
}

pub struct AbiToLuauConversion<'a> {
    names: &'a [String],
    ty: &'a Describe,
}

impl Render for AbiToLuauConversion<'_> {
    fn render(&self, ctx: &mut RenderContext) -> io::Result<()> {
        match self.ty {
            Describe::U8
            | Describe::U16
            | Describe::U32
            | Describe::I8
            | Describe::I16
            | Describe::I32
            | Describe::F64
            | Describe::F32 => write!(ctx, "{}", self.names[0]),
            Describe::ExternRef => write!(ctx, "HEAP_TAKE({})", self.names[0]),
            Describe::Boolean => write!(ctx, "{} == 1", self.names[0]),
            Describe::Void => write!(ctx, "nil"),
            Describe::Ref | Describe::RefMut => unimplemented!(),
            Describe::Function { .. } => unimplemented!(),
            Describe::Option { ty } => {
                write!(ctx, "if {} == 1 then ", self.names[0])?;
                ctx.render(AbiToLuauConversion {
                    names: &self.names[1..],
                    ty,
                })?;
                write!(ctx, " else nil")
            }
        }
    }
}

pub struct LuauToAbiConversion<'a> {
    name: &'a str,
    out_names: &'a [String],
    ty: &'a Describe,
}

impl Render for LuauToAbiConversion<'_> {
    fn render(&self, ctx: &mut RenderContext) -> io::Result<()> {
        let out0 = &self.out_names[0];
        match self.ty {
            Describe::U8
            | Describe::U16
            | Describe::U32
            | Describe::I8
            | Describe::I16
            | Describe::I32
            | Describe::F64
            | Describe::F32 => write!(ctx, "{}", self.name),
            Describe::Boolean => write!(ctx, "if {} then 1 else 0", self.name),
            Describe::Void => Ok(()),
            Describe::Ref | Describe::RefMut => unimplemented!(),
            Describe::Function { .. } => unimplemented!(),
            Describe::ExternRef => {
                writeln!(ctx.prereq, "HEAP_ID += 1")?;
                writeln!(ctx.prereq, "local HEAP_{} = HEAP_ID", self.name)?;
                writeln!(ctx.prereq, "HEAP[HEAP_ID] = {}", self.name)?;
                write!(ctx, "HEAP_{}", self.name)
            }
            Describe::Option { ty } => {
                writeln!(ctx.prereq, "if {} ~= nil then", self.name)?;

                let out1 = &self.out_names[1];
                let (prereq, expr) = ctx.render_expr(LuauToAbiConversion {
                    name: self.name,
                    out_names: &self.out_names[1..],
                    ty,
                })?;

                ctx.prereq.up();
                writeln!(ctx.prereq, "{out0} = 1")?;
                ctx.prereq.write_all(&prereq)?;
                if let Some(expr) = expr {
                    writeln!(ctx.prereq, "{out1} = {expr}")?;
                }
                ctx.prereq.down();

                writeln!(ctx.prereq, "end")
            }
        }
    }
}
