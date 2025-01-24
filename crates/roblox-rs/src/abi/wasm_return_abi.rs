use super::wasm_abi::WasmAbi;

#[repr(C)]
pub struct ReturnAbi<T: WasmAbi> {
    pub prim1: T::Prim1,
    pub prim2: T::Prim2,
    pub prim3: T::Prim3,
    pub prim4: T::Prim4,
}

impl<T: WasmAbi> From<T> for ReturnAbi<T> {
    fn from(value: T) -> Self {
        let (prim1, prim2, prim3, prim4) = T::split(value);

        Self {
            prim1,
            prim2,
            prim3,
            prim4,
        }
    }
}

impl<T: WasmAbi> ReturnAbi<T> {
    pub fn join(self) -> T {
        T::join(self.prim1, self.prim2, self.prim3, self.prim4)
    }
}
