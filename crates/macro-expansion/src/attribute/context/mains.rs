use std::fmt::Debug;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::emit::{emit_id, Emit};

pub struct ContextMain {
    pub item: syn::ItemFn,
    pub rust_name: String,
    pub export_name: String,
}

impl Debug for ContextMain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ContextMain")
    }
}

impl Emit for ContextMain {
    fn emit(&self, tokens: &mut TokenStream) {
        let rust_name = emit_id(&self.rust_name);
        let export_name = emit_id(&self.export_name);

        tokens.extend(quote! {
            #[no_mangle]
            pub extern "C" fn #export_name() {
                #rust_name();
            }
        });

        self.item.to_tokens(tokens);
    }
}
