use zerocopy::{FromBytes, Immutable, KnownLayout};

/// Treats the incoming buffer as being NUL terminated string and converts to UTF8 with lossy encoding
pub fn get_string(buffer: &mut [u8]) -> (String, &mut [u8]) {
    let len = memchr::memchr(0, buffer).expect("NUL terminated string");
    let (out, rest) = buffer.split_at_mut(len);
    let string = String::from_utf8_lossy(out).into_owned();
    (string, &mut rest[1..])
}

/// Get a slice of typed T. Return [Option::None] if there are not enough bytes. Also return
/// the remaining buffer
#[cfg(feature = "abi-7-16")]
pub fn get_vec<T: FromBytes + Immutable + KnownLayout + Copy>(buffer: &[u8], count: usize) -> Vec<T> {
    let elements = <[T]>::ref_from_prefix_with_elems(buffer, count).unwrap().0;
    elements.to_vec()
}
