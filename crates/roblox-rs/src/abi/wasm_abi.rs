use super::wasm_primitive::WasmPrimitive;

pub trait WasmAbi {
    type Prim1: WasmPrimitive;
    type Prim2: WasmPrimitive;
    type Prim3: WasmPrimitive;
    type Prim4: WasmPrimitive;

    /// Splits this type up into primitives to be sent over the ABI.
    fn split(self) -> (Self::Prim1, Self::Prim2, Self::Prim3, Self::Prim4);

    /// Reconstructs this type from primitives received over the ABI.
    fn join(prim1: Self::Prim1, prim2: Self::Prim2, prim3: Self::Prim3, prim4: Self::Prim4)
        -> Self;
}

impl<T: WasmPrimitive> WasmAbi for T {
    type Prim1 = Self;
    type Prim2 = ();
    type Prim3 = ();
    type Prim4 = ();

    #[inline(always)]
    fn join(prim1: Self::Prim1, _: (), _: (), _: ()) -> Self {
        prim1
    }

    #[inline(always)]
    fn split(self) -> (Self::Prim1, Self::Prim2, Self::Prim3, Self::Prim4) {
        (self, (), (), ())
    }
}

impl<T: WasmAbi<Prim4 = ()>> WasmAbi for Option<T> {
    type Prim1 = u8;
    type Prim2 = T::Prim1;
    type Prim3 = T::Prim2;
    type Prim4 = T::Prim3;

    fn join(
        prim1: Self::Prim1,
        prim2: Self::Prim2,
        prim3: Self::Prim3,
        prim4: Self::Prim4,
    ) -> Self {
        match prim1 {
            1.. => Some(T::join(prim2, prim3, prim4, ())),
            0 => None,
        }
    }

    fn split(self) -> (Self::Prim1, Self::Prim2, Self::Prim3, Self::Prim4) {
        match self {
            Some(value) => {
                let (prim1, prim2, prim3, _) = T::split(value);
                (1, prim1, prim2, prim3)
            }
            None => (
                0,
                Default::default(),
                Default::default(),
                Default::default(),
            ),
        }
    }
}
