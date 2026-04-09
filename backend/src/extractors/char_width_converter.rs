pub fn halfwidth_to_fullwidth(c: char) -> char {
    match c {
        'A'..='Z' => char_from_u32_with_default(c as u32 - 0x0041 + 0xFF21, c),
        'a'..='z' => char_from_u32_with_default(c as u32 - 0x0061 + 0xFF41, c),
        _ => c,
    }
}

pub fn char_from_u32_with_default(i: u32, def: char) -> char {
    char::from_u32(i).unwrap_or(def)
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;
    #[test]
    fn test_halfwidth_to_fullwidth() { for (input, expected) in [('a', 'ａ'), ('z', 'ｚ'), ('A', 'Ａ'), ('Z', 'Ｚ'), ('1', '1'), ('!', '!'), ('あ', 'あ')] { assert_eq!(halfwidth_to_fullwidth(input), expected); } for (input, default, expected) in [(0x41, 'X', 'A'), (0x110000, 'X', 'X')] { assert_eq!(char_from_u32_with_default(input, default), expected); } }
}
