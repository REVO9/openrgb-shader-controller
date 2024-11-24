pub fn float_to_string_decimal(f: f32) -> String {
    let mut str = f.to_string();
    if !str.contains(".") {
        str.push_str(".0");
    }
    str
}
