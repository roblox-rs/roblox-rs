use syn::{ForeignItemFn, Item, ItemFn, ItemForeignMod, ReturnType};

use crate::attribute::{parse::ParsedAttributes, symbol::new_symbol_name};

use super::{
    context::{
        export::{ContextExport, ExportFunction},
        import::{ContextImport, ImportFunction, ImportKind},
    },
    type_from_arg, Context,
};

pub trait Expand {
    fn expand(&self, ctx: &mut Context);
}

impl Expand for Item {
    fn expand(&self, ctx: &mut Context) {
        match self {
            Item::ForeignMod(item) => item.expand(ctx),
            Item::Fn(item) => item.expand(ctx),
            _ => {}
        }
    }
}

impl Expand for ItemForeignMod {
    fn expand(&self, ctx: &mut Context) {
        for item in &self.items {
            match item {
                syn::ForeignItem::Fn(f) => f.expand(ctx),
                syn::ForeignItem::Type(_) => (),
                _ => (),
            }
        }
    }
}

impl Expand for ForeignItemFn {
    fn expand(&self, ctx: &mut Context) {
        let attributes = ParsedAttributes::fetch(&self.attrs);
        let namespace = attributes.namespace;
        let rust_name = self.sig.ident.to_string();
        let luau_name = attributes.name.as_ref().unwrap_or(&rust_name).clone();
        let describe_name = new_symbol_name(&rust_name);
        let export_name = new_symbol_name(&rust_name);
        let arguments = self.sig.inputs.iter().map(type_from_arg).collect();
        let return_type = match &self.sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(*ty.clone()),
        };

        ctx.imports.push(ContextImport {
            namespace,
            import_kind: ImportKind::Function(ImportFunction {
                rust_name,
                luau_name,
                describe_name,
                export_name,
                return_type,
                arguments,
            }),
        });
    }
}

impl Expand for ItemFn {
    fn expand(&self, ctx: &mut Context) {
        let attributes = ParsedAttributes::fetch(&self.attrs);
        let item = self.clone();
        let rust_name = item.sig.ident.to_string();
        let export_name = new_symbol_name(&rust_name);
        let describe_name = new_symbol_name(&rust_name);
        let luau_name = attributes.name.as_ref().unwrap_or(&rust_name).clone();
        let arguments = item.sig.inputs.iter().map(type_from_arg).collect();
        let return_type = match &item.sig.output {
            ReturnType::Type(_, ty) => Some(*ty.clone()),
            ReturnType::Default => None,
        };

        ctx.exports.push(ContextExport::Function(ExportFunction {
            item,
            export_name,
            describe_name,
            rust_name,
            luau_name,
            return_type,
            arguments,
        }));
    }
}
