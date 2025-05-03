use std::{collections::HashSet, fs, io::Write, path::PathBuf};

use walrus::{
    ir::{Call, Const, Instr, Value},
    Export, ExportItem, FunctionId, FunctionKind, Import, ImportKind, LocalFunction, Module,
};

use crate::{
    codegen::{
        instructions::{
            self,
            headers::{CreateRuntimeHeader, CreateRuntimeTail},
        },
        traits::{Instruction, InstructionContext},
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

    let main_names = shared_context.main_fns;

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
        let parameters = args;
        let body = Box::new(instructions::InvokeRustFunction {
            function_name: export_name,
            output_type: output_type.clone(),
            parameters: parameters.clone(),
        });

        export_fns.push(instructions::WasmCreateExport {
            luau_name,
            output_type,
            parameters,
            body,
        })
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

        removed_exports.insert(describe_export.id());
        removed_functions.insert(describe_func_id);

        // This is a valid import, but the Rust code doesn't use it so we shouldn't generate it.
        if find_import(&module, &import.export_name).is_none() {
            continue;
        }

        let Describe::Function { args, return_type } = interpret_describe(describe_id, func) else {
            continue;
        };

        let luau_name = import.luau_name.to_string();
        let export_name = import.export_name.to_string();
        let output = *return_type.clone();
        let parameters = args;
        let body = Box::new(instructions::AbiBlock {
            inputs: parameters.clone(),
            output: output.clone(),
            body: Box::new(instructions::InvokeLuauFunction {
                function_name: luau_name.clone(),
                parameter_count: parameters.len(),
                result_count: output.value_count().min(1),
            }),
        });

        import_fns.push(instructions::WasmCreateImport {
            export_name,
            parameters,
            output,
            body,
        });
    }

    // Expose the stack pointer, if it exists.
    for global in module.globals.iter() {
        if let Some("__stack_pointer") = global.name.as_deref() {
            module.exports.add("__stack_pointer", global.id());
            break;
        }
    }

    // Expose the function table list, if it exists.
    if let Some(table) = module.tables.main_function_table().unwrap() {
        module.exports.add("__func_table", table);
    }

    fs::create_dir_all(out.join("server")).expect("could not create dir");

    write(out.join("default.project.json"), ROJO_TEMPLATE);
    write(out.join("server/runner.server.luau"), RUNNER_TEMPLATE);

    let mut wasm = fs::File::create(out.join("server/wasm.luau")).expect("file open failed");
    writeln!(wasm, "--!optimize 2").ok();
    writeln!(wasm, "{}", codegen_luau::RUNTIME).ok();

    let mut runtime = fs::File::create(out.join("server/runtime.luau")).expect("file open failed");
    let mut ctx = InstructionContext::new(&mut runtime, &shared_context.intrinsics);

    CreateRuntimeHeader.render(&mut ctx).unwrap();

    for instr in import_fns {
        instr.render(&mut ctx).expect("render failed");

        debug_assert_eq!(ctx.inputs.len(), 0);
    }

    for instr in export_fns {
        instr.render(&mut ctx).expect("render failed");

        debug_assert_eq!(ctx.inputs.len(), 0);
    }

    CreateRuntimeTail { main_names }.render(&mut ctx).unwrap();

    for intrinsic in &shared_context.intrinsics {
        let intrinsic_name = intrinsic.name.as_str();

        if !ctx.intrinsics.used.contains(&intrinsic_name) {
            let export = find_export(&module, &intrinsic.export_name).expect("intrinsic to exist");
            removed_exports.insert(export.id());
        }
    }

    for id in removed_exports {
        module.exports.delete(id);
    }

    for id in removed_functions {
        module.funcs.delete(id);
    }

    walrus::passes::gc::run(&mut module);

    let emit = module.emit_wasm();
    let wasynth_module = wasm_ast::module::Module::try_from_data(&emit).expect("module failure");
    codegen_luau::from_module_untyped(&wasynth_module, &mut wasm).expect("wasm2luau failure");

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
                _ => {
                    unimplemented!("unexpected instruction in description function: {instr:?}")
                }
            }
        }
        Describe::parse(&describe)
    }
}
