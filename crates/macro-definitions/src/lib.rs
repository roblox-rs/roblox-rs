use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn luau(attr: TokenStream, input: TokenStream) -> TokenStream {
    roblox_rs_macro_expansion::attribute::expand_attribute(attr.into(), input.into()).into()
}
