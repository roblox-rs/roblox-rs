mod description;
mod emit;
pub mod export;
pub mod import;

use emit::Emit;
use export::ContextExport;
use import::ContextImport;
use proc_macro2::TokenStream;
use quote::quote;
use roblox_rs_shared_context::shared_context::SharedContext;

#[derive(Debug)]
pub struct Context {
    pub imports: Vec<ContextImport>,
    pub exports: Vec<ContextExport>,
}

impl Context {
    pub fn emit(self) -> TokenStream {
        let mut tokens = TokenStream::new();

        for import in &self.imports {
            import.emit(&mut tokens);
        }

        for export in &self.exports {
            export.emit(&mut tokens);
        }

        let shared_context = SharedContext {
            imports: self.imports.into_iter().map(Into::into).collect(),
            exports: self.exports.into_iter().map(Into::into).collect(),
        };

        let encoded_data = roblox_rs_shared_context::encode(&shared_context);
        let encoded_len = encoded_data.len();

        tokens.extend(quote! {
            const _: () = {
                #[link_section = ".roblox-rs"]
                static ATTR: [u8; #encoded_len] = [#(#encoded_data),*];
            };
        });

        tokens
    }
}
