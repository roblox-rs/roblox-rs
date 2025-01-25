use roblox_rs::prelude::*;

#[luau]
extern "C" {
    fn print(value: Option<LuauValue>);

    #[luau(name = "Vector3.new")]
    fn vector3_new(x: f64, y: f64, z: f64) -> LuauValue;
}

#[luau(main)]
pub fn main() {
    let value = vector3_new(1.5, 2.5, 3.5);
    print(None);
    print(Some(value));
}
