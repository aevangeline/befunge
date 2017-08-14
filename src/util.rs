pub fn i64_to_char(num: i64) -> char {
    (num as u8) as char
}

pub fn char_to_i64(ch: char) -> i64 {
    (ch as u8) as i64
}
