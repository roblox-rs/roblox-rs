mod context;
mod expand;
mod parse;
pub mod symbol;

use context::Context;
use expand::Expand;
use proc_macro2::TokenStream;
use quote::quote;
use roblox_rs_shared_context::shared_context::{SharedContext, SharedIntrinsic};
use symbol::new_symbol_name;
use syn::{FnArg, Item, Type};

pub fn expand_attribute(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut context = Context {
        imports: Vec::new(),
        exports: Vec::new(),
        main_fns: Vec::new(),
        attributes: syn::parse2(attrs).unwrap(),
    };

    syn::parse2::<Item>(input).unwrap().expand(&mut context);

    context.emit()
}

pub fn expand_intrinsic(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse2::<Item>(input);
    match item {
        Ok(Item::Fn(item)) => {
            let export_name = new_symbol_name(item.sig.ident.to_string());
            let item_name = &item.sig.ident;
            let item_inputs = &item.sig.inputs;
            let item_output = &item.sig.output;
            let item_block = &item.block;
            let item_attrs = &item.attrs;

            let mut shared_context = SharedContext::default();
            shared_context.intrinsics.push(SharedIntrinsic {
                name: item.sig.ident.to_string(),
                export_name: export_name.clone(),
            });

            let encoded_data = roblox_rs_shared_context::encode(&shared_context);
            let encoded_len = encoded_data.len();

            quote! {
                const _: () = {
                    #[link_section = ".roblox-rs"]
                    static ATTR: [u8; #encoded_len] = [#(#encoded_data),*];
                };

                const _: () = {
                    #[no_mangle]
                    #[export_name=#export_name]
                    #(#item_attrs)*
                    unsafe extern "C" fn #item_name(#item_inputs) #item_output #item_block
                };
            }
        }
        _ => quote! {},
    }
}

fn type_from_arg(ty: &FnArg) -> Type {
    match ty {
        FnArg::Typed(ty) => *ty.ty.clone(),
        FnArg::Receiver(_) => unimplemented!(),
    }
}
