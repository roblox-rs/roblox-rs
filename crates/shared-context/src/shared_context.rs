use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SharedContext {
    pub imports: Vec<SharedImportFunction>,
    pub exports: Vec<SharedExportFunction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SharedImportFunction {
    pub rust_name: String,
    pub luau_name: String,
    pub describe_name: String,
    pub export_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SharedExportFunction {
    pub rust_name: String,
    pub luau_name: String,
    pub export_name: String,
    pub describe_name: String,
}

pub enum SharedFunction<'a> {
    Import(&'a SharedImportFunction),
    Export(&'a SharedExportFunction),
}

impl SharedContext {
    pub fn find_function<'a>(&'a self, name: &'_ str) -> Option<SharedFunction<'a>> {
        let import = self.imports.iter().find(|v| v.rust_name == name);
        let export = self.exports.iter().find(|v| v.rust_name == name);

        import
            .map(SharedFunction::Import)
            .or(export.map(SharedFunction::Export))
    }
}
