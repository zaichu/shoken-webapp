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
mod tests {
    use super::*;

    #[test]
    fn test_halfwidth_to_fullwidth_lowercase() {
        assert_eq!(halfwidth_to_fullwidth('a'), 'ａ');
        assert_eq!(halfwidth_to_fullwidth('z'), 'ｚ');
    }

    #[test]
    fn test_halfwidth_to_fullwidth_uppercase() {
        assert_eq!(halfwidth_to_fullwidth('A'), 'Ａ');
        assert_eq!(halfwidth_to_fullwidth('Z'), 'Ｚ');
    }

    #[test]
    fn test_halfwidth_to_fullwidth_non_letter() {
        assert_eq!(halfwidth_to_fullwidth('1'), '1');
        assert_eq!(halfwidth_to_fullwidth('!'), '!');
        assert_eq!(halfwidth_to_fullwidth('あ'), 'あ');
    }

    #[test]
    fn test_char_from_u32_with_default_valid() {
        assert_eq!(char_from_u32_with_default(0x41, 'X'), 'A');
    }

    #[test]
    fn test_char_from_u32_with_default_invalid() {
        assert_eq!(char_from_u32_with_default(0x110000, 'X'), 'X');
    }
}
