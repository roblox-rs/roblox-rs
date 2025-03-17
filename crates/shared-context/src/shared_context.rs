use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SharedContext {
    pub imports: Vec<SharedImportFunction>,
    pub exports: Vec<SharedExportFunction>,
    pub main_fns: Vec<String>,
    pub intrinsics: Vec<SharedIntrinsic>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct SharedIntrinsic {
    pub name: String,
    pub export_name: String,
}
