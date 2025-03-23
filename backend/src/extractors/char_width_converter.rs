pub fn halfwidth_to_fullwidth(c: char) -> char {
    match c {
        'A'..'Z' | 'a'..'z' => char_from_u32_with_default(c as u32 + 0xFF21 - 0x41, c),
        _ => c,
    }
}

pub fn char_from_u32_with_default(i: u32, def: char) -> char {
    char::from_u32(i).unwrap_or(def)
}
