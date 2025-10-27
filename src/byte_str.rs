#[cfg_attr(any(test, feature = "tracing"), derive(PartialEq))]
pub struct ByteStr<'a>(&'a [u8]);

impl<'a> std::ops::Deref for ByteStr<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> AsRef<[u8]> for ByteStr<'a> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'a> From<&'a [u8]> for ByteStr<'a> {
    #[inline(always)]
    fn from(bytes: &'a [u8]) -> Self {
        ByteStr(bytes)
    }
}

impl<'a> ByteStr<'a> {
    /// Returns the number of characters, handling UTF-8 correctly
    pub fn chars_count(&self) -> usize {
        if self.0.iter().any(|&b| b >= 0x80) {
            std::str::from_utf8(self.0)
                .map(|s| s.chars().count())
                .unwrap_or(self.0.len())
        } else {
            self.0.len()
        }
    }
}

#[cfg(test)]
impl<'a> From<&'a str> for ByteStr<'a> {
    #[inline(always)]
    fn from(s: &'a str) -> Self {
        ByteStr(s.as_bytes())
    }
}

#[cfg(any(test, feature = "tracing"))]
impl<'a> std::fmt::Debug for ByteStr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(self.0) {
            Ok(s) => write!(f, "{:?}", s),
            Err(_) => write!(f, "{:?}", self.0),
        }
    }
}
