/// gets the first digit of the number
pub fn first_digit(value: f32) -> i32 {
    (value / (10_i32.pow(value.log10().floor() as u32)) as f32).floor() as i32
}
