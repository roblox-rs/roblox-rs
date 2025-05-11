use std::alloc::Layout;

use roblox_rs_macro_definitions::intrinsic;

#[link(wasm_import_module = "roblox-rs")]
unsafe extern "C" {
    pub safe fn describe(value: u32);
}

#[intrinsic]
fn alloc(bytes: usize, align: usize) -> *mut u8 {
    let Ok(layout) = Layout::from_size_align(bytes, align) else {
        panic!("invalid allocation size")
    };

    let heap = std::alloc::alloc(layout);
    assert!(!heap.is_null(), "allocation error");

    heap
}

#[intrinsic]
fn free(ptr: *mut u8, bytes: usize, align: usize) {
    let Ok(layout) = Layout::from_size_align(bytes, align) else {
        panic!("invalid allocation size")
    };

    std::alloc::dealloc(ptr, layout);
}
