use crate::internal::{WasmFromAbi, WasmIntoAbi};

use super::{wasm_abi::WasmAbi, wasm_primitive::WasmPrimitive};

#[repr(C)]
pub struct WasmSlice {
    ptr: *mut u8,
    len: usize,
}

impl WasmAbi for WasmSlice {
    type Prim1 = u32;
    type Prim2 = u32;
    type Prim3 = ();
    type Prim4 = ();

    fn join(prim1: Self::Prim1, prim2: Self::Prim2, _: Self::Prim3, _: Self::Prim4) -> Self {
        Self {
            ptr: prim1 as *mut u8,
            len: prim2 as usize,
        }
    }

    fn split(self) -> (Self::Prim1, Self::Prim2, Self::Prim3, Self::Prim4) {
        (self.ptr as u32, self.len as u32, (), ())
    }
}

impl WasmIntoAbi for String {
    type Abi = <Vec<u8> as WasmIntoAbi>::Abi;

    fn into_abi(self) -> Self::Abi {
        self.into_bytes().into_abi()
    }
}

impl WasmFromAbi for String {
    type Abi = <Vec<u8> as WasmFromAbi>::Abi;

    unsafe fn from_abi(value: Self::Abi) -> Self {
        String::from_utf8_unchecked(<Vec<u8> as WasmFromAbi>::from_abi(value))
    }
}

impl<T: WasmPrimitive> WasmIntoAbi for &[T] {
    type Abi = WasmSlice;

    fn into_abi(self) -> Self::Abi {
        WasmSlice {
            ptr: self.as_ptr() as *mut u8,
            len: self.len(),
        }
    }
}

impl<T: WasmPrimitive> WasmIntoAbi for &mut [T] {
    type Abi = WasmSlice;

    fn into_abi(self) -> Self::Abi {
        (&*self).into_abi()
    }
}

impl<T: WasmIntoAbi> WasmIntoAbi for Box<[T]> {
    type Abi = WasmSlice;

    fn into_abi(self) -> Self::Abi {
        // TODO: Should we introduce a specialization trait to avoid this allocation on, e.g `Box<[u8]>` or `Vec<u8>`?
        let slice: Box<[_]> = self.into_vec().into_iter().map(|v| v.into_abi()).collect();
        let ptr = slice.as_ptr() as *mut u8;
        let len = slice.len();
        std::mem::forget(slice);

        WasmSlice { ptr, len }
    }
}

impl<T: WasmFromAbi> WasmFromAbi for Box<[T]> {
    type Abi = WasmSlice;

    unsafe fn from_abi(value: Self::Abi) -> Self {
        Vec::from_raw_parts(value.ptr as *mut T::Abi, value.len, value.len)
            .into_iter()
            .map(|v| T::from_abi(v))
            .collect()
    }
}

impl<T: WasmIntoAbi> WasmIntoAbi for Vec<T> {
    type Abi = <Box<[T]> as WasmIntoAbi>::Abi;

    fn into_abi(self) -> Self::Abi {
        self.into_boxed_slice().into_abi()
    }
}

impl<T: WasmFromAbi> WasmFromAbi for Vec<T> {
    type Abi = <Box<[T]> as WasmFromAbi>::Abi;

    unsafe fn from_abi(value: Self::Abi) -> Self {
        <Box<[T]> as WasmFromAbi>::from_abi(value).into_vec()
    }
}

impl<'a> WasmIntoAbi for &'a str {
    type Abi = <&'a [u8] as WasmIntoAbi>::Abi;

    fn into_abi(self) -> Self::Abi {
        self.as_bytes().into_abi()
    }
}
