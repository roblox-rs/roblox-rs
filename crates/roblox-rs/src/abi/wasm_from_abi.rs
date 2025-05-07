use crate::internal::WasmDescribe;

use super::{wasm_abi::WasmAbi, wasm_primitive::WasmPrimitive};

pub trait WasmFromAbi: WasmDescribe {
    type Abi: WasmAbi;

    unsafe fn from_abi(value: Self::Abi) -> Self;
}

impl<T: WasmPrimitive> WasmFromAbi for T {
    type Abi = T;

    #[inline(always)]
    unsafe fn from_abi(value: Self::Abi) -> Self {
        value
    }
}

// TODO: implement `wasm-bindgen` Option trait family to avoid using a primitive slot?
impl<T: WasmFromAbi<Abi: WasmAbi<Prim4 = ()>>> WasmFromAbi for Option<T> {
    type Abi = Option<T::Abi>;

    #[inline(always)]
    unsafe fn from_abi(value: Self::Abi) -> Self {
        value.map(|v| T::from_abi(v))
    }
}
