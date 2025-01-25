use roblox_rs::prelude::*;

/// The #[luau] macro is responsible for generating bindings between Rust and Luau.
/// You can import Luau/Roblox globals using `extern` and simply defining the function signature.
#[luau]
extern "C" {
    // This imports the Luau `print` function and allows you to pass an optional LuauValue to it.
    // You can change the signature to any values supported by the underlying Luau function.
    fn print(value: Option<LuauValue>);

    // You can import scoped functions, or rename functions, using the `name` attribute.
    // This will call `Vector3.new` in the generated bindings, and return the resulting Vector3 as a LuauValue.
    #[luau(name = "Vector3.new")]
    fn vector3_new(x: f64, y: f64, z: f64) -> LuauValue;
}

/// The #[luau] macro can also be applied to public functions.
///
/// These functions will be returned in the generated module,
/// and the values will automatically be translated between Rust and Luau.
///
/// You can rename the function name using the #[luau] macro's `name` attribute,
/// which will change its name in the generated module's exports.
#[luau]
pub fn echo_non_zero(value: u32) -> Option<u32> {
    if value == 0 {
        None
    } else {
        Some(value)
    }
}

/// The #[luau(main)] macro variant is a special function which automatically gets executed at runtime.
///
/// This can be used to write game code without explicitly needing to call a public function.
#[luau(main)]
pub fn main() {
    let value = vector3_new(1.5, 2.5, 3.5);
    print(None);
    print(Some(value));
}
