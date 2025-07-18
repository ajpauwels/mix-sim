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
