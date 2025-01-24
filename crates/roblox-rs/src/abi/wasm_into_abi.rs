use crate::internal::WasmDescribe;

use super::{wasm_abi::WasmAbi, wasm_primitive::WasmPrimitive};

pub trait WasmIntoAbi: WasmDescribe {
    type Abi: WasmAbi;

    fn into_abi(self) -> Self::Abi;
}

impl<T: WasmPrimitive> WasmIntoAbi for T {
    type Abi = T;

    #[inline(always)]
    fn into_abi(self) -> Self::Abi {
        self
    }
}

// TODO: implement `wasm-bindgen` Option trait family to avoid using a primitive slot?
impl<T: WasmIntoAbi<Abi: WasmAbi<Prim4 = ()>>> WasmIntoAbi for Option<T> {
    type Abi = Option<T::Abi>;

    #[inline(always)]
    fn into_abi(self) -> Self::Abi {
        self.map(|v| v.into_abi())
    }
}
