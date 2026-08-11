pub fn to_superscript(num: i32) -> String {
    let superscripts = ["⁰", "¹", "²", "³", "⁴", "⁵", "⁶", "⁷", "⁸", "⁹"];

    num.to_string()
        .chars()
        .map(|c| {
            if let Some(digit) = c.to_digit(10) {
                superscripts[digit as usize]
            } else {
                "⁻"
            }
        })
        .collect()
}
