use crate::internal::{WasmDescribe, WasmFromAbi, WasmIntoAbi};

pub struct LuauValue(u32);

impl WasmDescribe for LuauValue {
    #[inline(always)]
    fn describe() {
        use crate::internal::*;

        describe(EXTERNREF);
    }
}

impl WasmIntoAbi for LuauValue {
    type Abi = u32;

    fn into_abi(self) -> Self::Abi {
        self.0
    }
}

impl WasmIntoAbi for &LuauValue {
    type Abi = u32;

    fn into_abi(self) -> Self::Abi {
        self.0
    }
}

impl WasmFromAbi for LuauValue {
    type Abi = u32;

    unsafe fn from_abi(value: Self::Abi) -> Self {
        LuauValue(value)
    }
}
