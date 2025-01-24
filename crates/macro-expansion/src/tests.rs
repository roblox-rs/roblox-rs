#![cfg(test)]

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ItemForeignMod;

use crate::attribute;

#[test]
fn attribute_expansion() {
    let pseudo_impl = quote! {
        #[wasm_bindgen(namespace = "test")]
        extern "C" {
            fn my_func(a: String, b: u64);
        }
    };

    let mut parsed = syn::parse2::<ItemForeignMod>(pseudo_impl).unwrap();
    let attribute = parsed.attrs[0].parse_args().unwrap_or_default();
    parsed.attrs.clear();

    let input: TokenStream = parsed.into_token_stream();

    println!("{:?}", attribute);
    println!("{:?}", input);

    let result = attribute::expand_attribute(attribute, input);
    println!("{}", result);
}
