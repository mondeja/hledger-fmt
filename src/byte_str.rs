#[cfg_attr(any(test, feature = "tracing"), derive(PartialEq))]
pub struct ByteStr<'a> {
    bytes: &'a [u8],
}

impl<'a> std::ops::Deref for ByteStr<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.bytes
    }
}

impl<'a> AsRef<[u8]> for ByteStr<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.bytes
    }
}

impl<'a> From<&'a [u8]> for ByteStr<'a> {
    #[inline(always)]
    fn from(bytes: &'a [u8]) -> Self {
        ByteStr { bytes }
    }
}

impl<'a> ByteStr<'a> {
    /// Returns the number of characters, handling UTF-8 correctly
    pub fn chars_count(&self) -> usize {
        utf8_chars_count(self.bytes)
    }
}

#[cfg(test)]
impl<'a> From<&'a str> for ByteStr<'a> {
    #[inline(always)]
    fn from(s: &'a str) -> Self {
        ByteStr {
            bytes: s.as_bytes(),
        }
    }
}

#[cfg(any(test, feature = "tracing"))]
impl<'a> std::fmt::Debug for ByteStr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(self.bytes) {
            Ok(s) => write!(f, "{:?}", s),
            Err(_) => write!(f, "{:?}", self.bytes),
        }
    }
}

/// Counts the number of UTF-8 characters in a byte slice.
///
/// This function iterates over the bytes of the UTF-8 encoded slice `buf`
/// and counts only the bytes that are **not continuation bytes**, i.e.,
/// the bytes that start a new UTF-8 character.
///
/// In UTF-8 encoding:
/// - ASCII characters (U+0000..U+007F) are single bytes (0xxxxxxx).
/// - Multibyte characters start with bytes that have the high bits
///   110xxxxx, 1110xxxx, or 11110xxx.
/// - Continuation bytes always have the form 10xxxxxx.
///
/// Therefore, the function counts all bytes that are **not of the form 10xxxxxx**
/// to determine the number of Unicode characters in the slice.
#[inline]
pub(crate) fn utf8_chars_count(buf: &[u8]) -> usize {
    let mut count = 0;
    let len = buf.len();
    let ptr = buf.as_ptr();

    for i in 0..len {
        // SAFETY: i < len
        let byte = unsafe { *ptr.add(i) };
        if byte & 0b1100_0000 != 0b1000_0000 {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::utf8_chars_count;

    #[test]
    fn test_utf8_chars_count() {
        let test_cases = vec![
            (b"hello", 5),
            (b"h\xE9llo", 5), // 'é' is a 2-byte character
        ];

        for (input, expected) in test_cases {
            let count = utf8_chars_count(input);
            assert_eq!(
                count, expected,
                "utf8_chars_count({:?}) = {}, expected {}",
                input, count, expected
            );
        }
    }
}
