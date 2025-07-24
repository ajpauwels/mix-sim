use std::borrow::Cow;

// Takes a string and converts it to an array of 32-bytes, panicking
// if the string is too long, and filling the rest of the array with
// zeros if it's too short
pub fn str_to_byte_array_32(s: &str) -> [u8; 32] {
    if s.len() > 32 {
        panic!("String \"{}\" is longer than 32 characters", s);
    } else {
        let s_bytes = s.as_bytes();
        let mut v = [0u8; 32];
        v[..s_bytes.len()].copy_from_slice(s_bytes);
        v
    }
}

// Accepts a slice of bytes, removes all trailing zero-bytes, and
// converts the remaining prefix into a string
//
// WARNING: Invalid UTF-8 characters in the prefix will be replaced
// with the U+FFFD (�) character
pub fn bytes_to_string_truncate_zeroes(bytes: &[u8]) -> Cow<'_, str> {
    match bytes.iter().rposition(|&b| b != 0u8) {
        Some(len) => String::from_utf8_lossy(&bytes[0..len + 1]),
        None => String::from_utf8_lossy(bytes),
    }
}
