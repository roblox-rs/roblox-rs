use crate::internal::{WasmAbi, WasmDescribe, WasmFromAbi, WasmIntoAbi};

pub struct LuauValue(u32);

impl WasmDescribe for LuauValue {
    #[inline(always)]
    fn describe() {
        use crate::internal::*;

        describe(EXTERNREF);
    }
}

impl WasmAbi for LuauValue {
    type Prim1 = u32;
    type Prim2 = ();
    type Prim3 = ();
    type Prim4 = ();

    fn join(
        prim1: Self::Prim1,
        _prim2: Self::Prim2,
        _prim3: Self::Prim3,
        _prim4: Self::Prim4,
    ) -> Self {
        LuauValue(prim1)
    }

    fn split(self) -> (Self::Prim1, Self::Prim2, Self::Prim3, Self::Prim4) {
        (self.0, (), (), ())
    }
}

impl WasmIntoAbi for LuauValue {
    type Abi = LuauValue;

    fn into_abi(self) -> Self::Abi {
        self
    }
}

impl WasmFromAbi for LuauValue {
    type Abi = LuauValue;

    unsafe fn from_abi(value: Self::Abi) -> Self {
        value
    }
}
