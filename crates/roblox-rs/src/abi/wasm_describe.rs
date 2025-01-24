use crate::ffi::describe;

pub trait WasmDescribe {
    fn describe();
}

pub const U8: u32 = 0;
pub const U16: u32 = 1;
pub const U32: u32 = 2;
pub const I8: u32 = 3;
pub const I16: u32 = 4;
pub const I32: u32 = 5;
pub const BOOLEAN: u32 = 6;
pub const REF: u32 = 7;
pub const REF_MUT: u32 = 8;
pub const FUNCTION: u32 = 9;
pub const VOID: u32 = 10;
pub const F32: u32 = 11;
pub const F64: u32 = 12;
pub const OPTION: u32 = 13;

macro_rules! simple {
	($($t:ty:$e:expr;)*) => {
		$(
			impl WasmDescribe for $t {
                #[inline(always)]
				fn describe() {
					$crate::ffi::describe($e);
				}
			}
		)*
	};
}

simple!(
    u8: U8;
    u16: U16;
    u32: U32;
    i8: I8;
    i16: I16;
    i32: I32;
    f32: F32;
    f64: F64;
    bool: BOOLEAN;
);

impl<T> WasmDescribe for *const T {
    #[inline(always)]
    fn describe() {
        <u32 as WasmDescribe>::describe();
    }
}

impl<T> WasmDescribe for *mut T {
    #[inline(always)]
    fn describe() {
        <u32 as WasmDescribe>::describe();
    }
}

impl<T: WasmDescribe + ?Sized> WasmDescribe for &T {
    #[inline(always)]
    fn describe() {
        describe(REF);
        T::describe();
    }
}

impl<T: WasmDescribe + ?Sized> WasmDescribe for &mut T {
    #[inline(always)]
    fn describe() {
        describe(REF_MUT);
        T::describe();
    }
}

impl WasmDescribe for () {
    #[inline(always)]
    fn describe() {
        describe(VOID);
    }
}

impl<T: WasmDescribe> WasmDescribe for Option<T> {
    #[inline(always)]
    fn describe() {
        describe(OPTION);
        T::describe();
    }
}
