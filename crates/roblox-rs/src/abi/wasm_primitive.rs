use crate::internal::WasmDescribe;

/// # Safety
/// This can only be implemented on types that are safe to pass through ABI.
pub unsafe trait WasmPrimitive: Default + WasmDescribe {}

macro_rules! impl_primitive {
    ($($id:ty),*) => {
        $(
			unsafe impl WasmPrimitive for $id {}
		)*
    };
}

impl_primitive!(u8, u16, u32, i8, i16, i32, f32, f64, ());
