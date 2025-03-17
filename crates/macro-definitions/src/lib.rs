use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn luau(attr: TokenStream, input: TokenStream) -> TokenStream {
    roblox_rs_macro_expansion::attribute::expand_attribute(attr.into(), input.into()).into()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn intrinsic(attr: TokenStream, input: TokenStream) -> TokenStream {
    roblox_rs_macro_expansion::attribute::expand_intrinsic(attr.into(), input.into()).into()
}
