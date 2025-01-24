use std::fmt::Debug;

use proc_macro2::TokenStream;
use quote::quote;
use roblox_rs_shared_context::shared_context::SharedImportFunction;

use super::{
    description::FunctionDescription,
    emit::{emit_id, get_abi_args, with_item, with_trait, Emit},
};

#[derive(Debug)]
pub struct ContextImport {
    pub namespace: Option<String>,
    pub import_kind: ImportKind,
}

impl Emit for ContextImport {
    fn emit(&self, tokens: &mut TokenStream) {
        self.import_kind.emit(tokens);
    }
}

impl From<ContextImport> for SharedImportFunction {
    fn from(value: ContextImport) -> Self {
        match value.import_kind {
            ImportKind::Function(f) => Self {
                describe_name: f.describe_name,
                luau_name: f.luau_name,
                rust_name: f.rust_name,
                export_name: f.export_name,
            },
        }
    }
}

#[derive(Debug)]
pub enum ImportKind {
    Function(ImportFunction),
}

impl Emit for ImportKind {
    fn emit(&self, tokens: &mut TokenStream) {
        match self {
            ImportKind::Function(f) => f.emit(tokens),
        }
    }
}

pub struct ImportFunction {
    pub luau_name: String,
    pub rust_name: String,
    pub export_name: String,
    pub describe_name: String,
    pub return_type: Option<syn::Type>,
    pub arguments: Vec<syn::Type>,
}

impl Emit for ImportFunction {
    fn emit(&self, tokens: &mut TokenStream) {
        FunctionDescription {
            arg_types: &self.arguments,
            describe_name: &self.describe_name,
            output_type: self.return_type.as_ref(),
        }
        .emit(tokens);

        let mut def_args = Vec::new();
        let mut def_arg_conversion = Vec::new();

        let def_return = match &self.return_type {
            Some(value) => quote! { -> #value },
            None => quote! {},
        };

        let rust_name = emit_id(&self.rust_name);
        let abi_name = emit_id(&self.export_name);
        let mut abi_args = Vec::new();
        let mut abi_arg_names = Vec::new();
        for (index, arg) in self.arguments.iter().enumerate() {
            let arg_name = emit_id(format!("arg{index}"));
            let into_abi = with_trait(arg, "WasmIntoAbi");
            let wasm_abi = with_trait(arg, "WasmAbi");
            let (names, args) = get_abi_args(&arg_name, arg);

            def_args.push(quote! { #arg_name: #arg });
            def_arg_conversion.push(quote! {
                let #arg_name = #into_abi::into_abi(#arg_name);
                let (#(#names),*) = #wasm_abi::split(#arg_name);
            });

            abi_args.extend(args);
            abi_arg_names.extend(names);
        }

        let return_id = emit_id("result");
        let return_abi = with_item("ReturnAbi");
        let from_abi = with_item("WasmFromAbi");
        let (abi_return, abi_return_transform) = match &self.return_type {
            Some(value) => (
                quote! { -> #return_abi<#value> },
                quote! { <#value as #from_abi>::from_abi(#return_id.join()) },
            ),
            None => (quote! {}, quote! {}),
        };

        let abi_fn = quote! {
            #[link(wasm_import_module = "luau")]
            extern "C" {
                fn #abi_name(#(#abi_args),*) #abi_return;
            }
        };

        tokens.extend(quote! {
            fn #rust_name(#(#def_args),*) #def_return {
                #abi_fn
                #(#def_arg_conversion)*
                let #return_id = unsafe { #abi_name(#(#abi_arg_names),*) };
                #abi_return_transform
            }
        });
    }
}

impl Debug for ImportFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ImportFunction")
    }
}
