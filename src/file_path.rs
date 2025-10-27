use smallvec::SmallVec;

#[derive(Clone)]
pub(crate) struct FilePath(SmallVec<[u8; 128]>);

impl FilePath {
    pub fn new() -> Self {
        FilePath(SmallVec::new())
    }

    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        self.as_os_str().to_string_lossy()
    }

    fn as_os_str(&self) -> &std::ffi::OsStr {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            std::ffi::OsStr::from_bytes(&self.0)
        }

        #[cfg(windows)]
        {
            std::ffi::OsStr::new(std::str::from_utf8_lossy(&self.0).as_ref())
        }
    }
}

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_os_str().to_string_lossy())
    }
}

impl std::fmt::Debug for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_os_str().to_string_lossy())
    }
}

impl From<&str> for FilePath {
    fn from(s: &str) -> Self {
        FilePath(SmallVec::from_slice(s.as_bytes()))
    }
}

impl From<&[u8]> for FilePath {
    fn from(bytes: &[u8]) -> Self {
        FilePath(SmallVec::from_slice(bytes))
    }
}

impl From<u8> for FilePath {
    fn from(s: u8) -> Self {
        FilePath(SmallVec::from_slice(&[s]))
    }
}

impl AsRef<[u8]> for FilePath {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<std::ffi::OsStr> for FilePath {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.as_os_str()
    }
}

impl AsRef<std::path::Path> for FilePath {
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(self.as_os_str())
    }
}

impl From<std::path::PathBuf> for FilePath {
    fn from(path_buf: std::path::PathBuf) -> Self {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            FilePath(SmallVec::from_slice(path_buf.as_os_str().as_bytes()))
        }

        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            let wide: Vec<u16> = path_buf.as_os_str().encode_wide().collect();
            let bytes: Vec<u8> = wide.iter().flat_map(|w| w.to_le_bytes()).collect();
            FilePath(SmallVec::from_slice(&bytes))
        }
    }
}

impl From<FilePath> for std::path::PathBuf {
    fn from(file_path: FilePath) -> Self {
        std::path::PathBuf::from(file_path.as_os_str())
    }
}
