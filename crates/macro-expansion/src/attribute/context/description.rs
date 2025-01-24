use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

use super::emit::{emit_id, Emit};

pub struct FunctionDescription<'a> {
    pub describe_name: &'a str,
    pub arg_types: &'a [Type],
    pub output_type: Option<&'a Type>,
}

impl Emit for FunctionDescription<'_> {
    fn emit(&self, tokens: &mut TokenStream) {
        let describe_name = emit_id(self.describe_name);
        let arg_types = self.arg_types;
        let arg_count = self.arg_types.len() as u32;
        let output_type = match self.output_type {
            Some(ty) => quote! { #ty },
            None => quote! { () },
        };

        tokens.extend(quote! {
            #[no_mangle]
            #[inline(never)]
            extern "C" fn #describe_name() {
                use roblox_rs::internal::*;

                describe(FUNCTION);
                describe(#arg_count);
                #(<#arg_types as WasmDescribe>::describe();)*;
                <#output_type as WasmDescribe>::describe();
            }
        });
    }
}
