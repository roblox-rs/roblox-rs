mod context;
mod expand;
mod parse;
mod symbol;

use context::Context;
use expand::Expand;
use proc_macro2::TokenStream;
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

fn type_from_arg(ty: &FnArg) -> Type {
    match ty {
        FnArg::Typed(ty) => *ty.ty.clone(),
        FnArg::Receiver(_) => unimplemented!(),
    }
}
