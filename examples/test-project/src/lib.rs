use std::hint::black_box;

use roblox_rs::prelude::*;

#[luau]
extern "C" {
    fn print(value: Option<u32>);

    #[luau(name = "print")]
    fn no_spill(value: Option<u32>) -> u32;

    #[luau(name = "math.random")]
    fn will_spill(value: u32) -> Option<f64>;
}

#[luau]
pub fn do_something_will_spill(a: u32, b: Option<u32>) -> Option<u32> {
    // print(Some(a));
    // print(b);
    b
}

#[luau]
pub fn do_something_wont_spill(a: u32, b: Option<u32>) -> u32 {
    // print(Some(a));
    // print(b);
    a
}

#[no_mangle]
pub fn main() {
    print(Some(15));
    print(None);
    no_spill(None);
}

#[no_mangle]
pub fn ignore(value: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8), x: Box<dyn FnOnce()>) {
    black_box(value);
    x();
}
