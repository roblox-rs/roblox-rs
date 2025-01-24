use std::fmt::Debug;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use roblox_rs_shared_context::shared_context::SharedExportFunction;

use crate::attribute::type_from_arg;

use super::{
    description::FunctionDescription,
    emit::{emit_id, get_abi_args, with_assoc, with_item, with_trait, Emit},
};

#[derive(Debug)]
pub enum ContextExport {
    Function(ExportFunction),
}

impl Emit for ContextExport {
    fn emit(&self, tokens: &mut TokenStream) {
        match self {
            ContextExport::Function(f) => f.emit(tokens),
        }
    }
}

impl From<ContextExport> for SharedExportFunction {
    fn from(value: ContextExport) -> Self {
        match value {
            ContextExport::Function(f) => Self {
                describe_name: f.describe_name,
                export_name: f.export_name,
                luau_name: f.luau_name,
                rust_name: f.rust_name,
            },
        }
    }
}

pub struct ExportFunction {
    pub item: syn::ItemFn,
    pub rust_name: String,
    pub luau_name: String,
    pub export_name: String,
    pub describe_name: String,
    pub arguments: Vec<syn::Type>,
    pub return_type: Option<syn::Type>,
}

impl Emit for ExportFunction {
    fn emit(&self, tokens: &mut TokenStream) {
        let name = &self.item.sig.ident;
        let export_name = emit_id(&self.export_name);

        FunctionDescription {
            arg_types: &self.arguments,
            describe_name: &self.describe_name,
            output_type: self.return_type.as_ref(),
        }
        .emit(tokens);

        let mut arg_names = Vec::new();
        let mut abi_args = Vec::new();
        let mut abi_arg_conversions = Vec::new();
        for (i, arg) in self.item.sig.inputs.iter().enumerate() {
            let ty = type_from_arg(arg);
            let arg_name = emit_id(format!("arg{i}"));
            let from_abi = with_trait(&ty, "WasmFromAbi");
            let wasm_abi = with_trait(with_assoc(&from_abi, "Abi"), "WasmAbi");

            let (names, args) = get_abi_args(&arg_name, &ty);
            abi_arg_conversions.push(quote! {
                let #arg_name = {
                    #from_abi::from_abi(#wasm_abi::join(#(#names),*))
                };
            });

            abi_args.extend(args);
            arg_names.push(arg_name);
        }

        let result_id = emit_id("result");
        let return_abi = with_item("ReturnAbi");
        let into_abi = with_item("WasmIntoAbi");
        let (abi_return, abi_return_conversion) = match &self.return_type {
            Some(value) => (
                quote! { -> #return_abi<<#value as #into_abi>::Abi> },
                quote! { #return_abi::from(<#value as #into_abi>::into_abi(#result_id)) },
            ),
            None => (quote! {}, quote! {}),
        };

        tokens.extend(quote! {
            #[no_mangle]
            extern "C" fn #export_name(#(#abi_args),*) #abi_return {
                #(#abi_arg_conversions)*
                let #result_id = #name(#(#arg_names),*);
                #abi_return_conversion
            }
        });

        self.item.to_tokens(tokens);
    }
}

impl Debug for ExportFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExportFunction")
    }
}
