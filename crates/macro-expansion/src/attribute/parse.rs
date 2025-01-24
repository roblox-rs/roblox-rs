use proc_macro2::{Punct, Span};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, Expr, ExprLit, Ident, Lit, Token,
};

#[derive(Default, Debug)]
pub struct ParsedAttributes {
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub main: Option<bool>,
}

#[derive(Debug)]
pub struct ParsedAttribute {
    name: String,
    name_span: Span,
    value: AttributeValue,
    value_span: Span,
}

#[derive(Debug)]
pub enum AttributeValue {
    String(String),
    Boolean(bool),
}

impl ParsedAttributes {
    pub fn fetch(attributes: &[Attribute]) -> Self {
        attributes
            .iter()
            .find(|attr| attr.path().segments[0].ident == "luau")
            .map(|v| syn::parse2(v.parse_args().unwrap()).unwrap())
            .unwrap_or_default()
    }
}

impl Parse for ParsedAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attributes = ParsedAttributes::default();

        let attribute_list = Punctuated::<ParsedAttribute, Comma>::parse_terminated(input)?;

        for attribute in attribute_list {
            parse_attributes!(
                attribute & attributes;
                namespace => String
                name => String
                main => Boolean
            )
        }

        Ok(attributes)
    }
}

impl Parse for ParsedAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if input.is_empty() || input.peek(Token![,]) {
            return Ok(Self {
                name: ident.to_string(),
                name_span: ident.span(),
                value: AttributeValue::Boolean(true),
                value_span: ident.span(),
            });
        }

        let equals: Punct = input.parse()?;
        if equals.as_char() != '=' {
            return Err(input.error("Invalid input, expected equals sign."));
        }

        let expr: Expr = input.parse()?;

        Ok(Self {
            name: ident.to_string(),
            name_span: ident.span(),
            value_span: expr.span(),
            value: match expr {
                Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Str(str),
                }) => AttributeValue::String(str.value()),
                Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Bool(bool),
                }) => AttributeValue::Boolean(bool.value),
                _ => Err(input.error("unexpected expression type"))?,
            },
        })
    }
}

macro_rules! parse_attributes {
	($attr:ident & $attrs:ident; $($name:ident => $pat:ident)*) => {
		if (false) {}

		$(
			else if ($attr.name == stringify!($name)) {
				$attrs.$name = match $attr.value {
					AttributeValue::$pat(value) => Some(value),
					_ => Err(syn::Error::new(
						$attr.value_span,
						format!("invalid value type, expected {}", stringify!($pat)),
					))?
				}
			}
		)*

		else {
			Err(syn::Error::new(
				$attr.name_span,
				format!("attribute name not recognized")
			))?
		}
	};
}

use parse_attributes;
