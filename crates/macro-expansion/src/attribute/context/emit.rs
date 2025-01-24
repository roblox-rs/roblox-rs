use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Ident, Type};

pub trait Emit {
    fn emit(&self, tokens: &mut TokenStream);
}

/// Utility function to create syn identifiers from strings.
pub fn emit_id(str: impl AsRef<str>) -> Ident {
    Ident::new(str.as_ref(), Span::call_site())
}

pub fn with_item(struct_name: impl AsRef<str>) -> TokenStream {
    let struct_name = emit_id(struct_name);

    quote! {
        roblox_rs::internal::#struct_name
    }
}

pub fn with_assoc(ty: impl ToTokens, assoc: impl AsRef<str>) -> TokenStream {
    let assoc = emit_id(assoc);

    quote! {
        #ty::#assoc
    }
}

pub fn with_trait(ty: impl ToTokens, trait_name: impl AsRef<str>) -> TokenStream {
    let trait_name = emit_id(trait_name);

    quote! {
        <#ty as roblox_rs::internal::#trait_name>
    }
}

pub fn get_abi_args(arg_name: impl Display, ty: &Type) -> (Vec<Ident>, Vec<TokenStream>) {
    let mut abi_arg_names = Vec::new();
    let mut abi_args = Vec::new();

    let from_abi = with_trait(ty, "WasmFromAbi");
    let wasm_abi = with_trait(with_assoc(&from_abi, "Abi"), "WasmAbi");

    for prim in 1..=4 {
        let arg_name = emit_id(format!("{arg_name}_{prim}"));
        let prim_type = with_assoc(&wasm_abi, format!("Prim{prim}"));

        abi_args.push(quote! { #arg_name: #prim_type });
        abi_arg_names.push(arg_name);
    }

    (abi_arg_names, abi_args)
}
