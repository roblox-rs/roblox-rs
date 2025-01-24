use std::{collections::HashSet, fs, io::Write, path::PathBuf};

use log::debug;
use walrus::{
    ir::{Call, Const, Instr, Value},
    Export, ExportItem, FunctionId, FunctionKind, Import, ImportKind, LocalFunction, Module,
};

use crate::{
    codegen::{
        ast::{self, RuntimeHeader, RuntimeTail},
        splat,
        traits::{Render, RenderContext},
    },
    describe::Describe,
};

const ROJO_TEMPLATE: &str = include_str!("../rojo-template.json");
const RUNNER_TEMPLATE: &str = include_str!("../runner-template.luau");

pub fn build(mut module: Module, out: PathBuf) {
    let describe_id = module.imports.iter().find_map(|v| match v.kind {
        ImportKind::Function(f) if v.module == "roblox-rs" && v.name == "describe" => Some(f),
        _ => None,
    });

    let shared_context = module
        .customs
        .remove_raw(".roblox-rs")
        .map(|v| roblox_rs_shared_context::decode(&v.data))
        .unwrap_or_default();

    let mut import_fns = Vec::new();
    let mut export_fns = Vec::new();

    let mut removed_exports = HashSet::new();
    let mut removed_functions = HashSet::new();

    for export in shared_context.exports.iter() {
        let Some(describe_export) = find_export(&module, &export.describe_name) else {
            continue;
        };

        let ExportItem::Function(describe_func_id) = describe_export.item else {
            continue;
        };

        let FunctionKind::Local(func) = &module.funcs.get(describe_func_id).kind else {
            continue;
        };

        removed_exports.insert(describe_export.id());
        removed_functions.insert(describe_func_id);

        let Describe::Function { args, return_type } = interpret_describe(describe_id, func) else {
            continue;
        };

        let export_name = export.export_name.to_string();
        let luau_name = export.luau_name.to_string();
        let output_type = *return_type.clone();
        let parameters = args
            .into_iter()
            .enumerate()
            .map(|(i, ty)| {
                let name = format!("arg{i}");
                ast::ExportFnParameter { name, ty }
            })
            .collect();

        export_fns.push(ast::ExportFn {
            export_name,
            luau_name,
            parameters,
            output_type,
        });
    }

    for import in shared_context.imports.iter() {
        let Some(describe_export) = find_export(&module, &import.describe_name) else {
            continue;
        };

        let ExportItem::Function(describe_func_id) = describe_export.item else {
            continue;
        };

        let FunctionKind::Local(func) = &module.funcs.get(describe_func_id).kind else {
            continue;
        };

        // This is a valid import, but the Rust code doesn't use it so we shouldn't generate it.
        if find_import(&module, &import.export_name).is_none() {
            continue;
        }

        removed_exports.insert(describe_export.id());
        removed_functions.insert(describe_func_id);

        let Describe::Function { args, return_type } = interpret_describe(describe_id, func) else {
            continue;
        };

        let luau_name = import.luau_name.to_string();
        let export_name = import.export_name.to_string();
        let output_type = *return_type.clone();
        let parameters = args
            .into_iter()
            .enumerate()
            .map(|(i, ty)| {
                let names = splat(&ty, &format!("arg{i}"));
                ast::InputFnParameter { names, ty }
            })
            .collect();

        import_fns.push(ast::ImportFn {
            luau_name,
            export_name,
            parameters,
            output_type,
        })
    }

    for id in removed_exports {
        module.exports.delete(id);
    }

    for id in removed_functions {
        module.funcs.delete(id);
    }

    walrus::passes::gc::run(&mut module);

    // Expose the stack pointer, if it exists.
    for global in module.globals.iter() {
        debug!("global: {global:?}");

        if let Some("__stack_pointer") = global.name.as_deref() {
            module.exports.add("__stack_pointer", global.id());
            break;
        }
    }

    // Expose the function table list, if it exists.
    if let Some(table) = module.tables.main_function_table().unwrap() {
        module.exports.add("__func_table", table);
    }

    let emit = module.emit_wasm();
    let wasynth_module = wasm_ast::module::Module::try_from_data(&emit).expect("module failure");

    fs::create_dir_all(out.join("server")).expect("could not create dir");

    write(out.join("default.project.json"), ROJO_TEMPLATE);
    write(out.join("server/runner.server.luau"), RUNNER_TEMPLATE);

    let mut wasm = fs::File::create(out.join("server/wasm.luau")).expect("file open failed");
    writeln!(wasm, "--!optimize 2").ok();
    writeln!(wasm, "{}", codegen_luau::RUNTIME).ok();

    codegen_luau::from_module_untyped(&wasynth_module, &mut wasm).expect("wasm2luau failure");

    let mut runtime = fs::File::create(out.join("server/runtime.luau")).expect("file open failed");
    let mut render_context = RenderContext::new(&mut runtime);
    render_context.render(RuntimeHeader).unwrap();

    for import_fn in import_fns {
        import_fn
            .render(&mut render_context)
            .expect("render failed");
    }

    for export_fn in export_fns {
        export_fn
            .render(&mut render_context)
            .expect("render failed");
    }

    render_context.render(RuntimeTail).unwrap();

    wasm.flush().expect("flush failed");
    runtime.flush().expect("flush failed");

    fn write(path: PathBuf, contents: &str) {
        fs::write(path, contents).expect("failed to write file")
    }

    fn find_export<'a>(module: &'a Module, name: &'_ str) -> Option<&'a Export> {
        module.exports.iter().find(|v| v.name == name)
    }

    fn find_import<'a>(module: &'a Module, name: &'_ str) -> Option<&'a Import> {
        module.imports.iter().find(|v| v.name == name)
    }

    fn interpret_describe(describe_id: Option<FunctionId>, func: &LocalFunction) -> Describe {
        let block = func.block(func.entry_block());
        let mut describe = Vec::new();
        let mut stack = 0u32;
        for (instr, _) in &block.instrs {
            match instr {
                Instr::Const(Const {
                    value: Value::I32(i),
                }) => stack = *i as u32,
                Instr::Call(Call { func }) if Some(*func) == describe_id => {
                    describe.push(stack);
                }
                _ => {}
            }
        }
        Describe::parse(&describe)
    }
}
