pub(crate) enum FilePathOrStdin {
    FilePath(std::path::PathBuf),
    Stdin,
}

impl FilePathOrStdin {
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        match self {
            FilePathOrStdin::FilePath(p) => p.to_string_lossy(),
            FilePathOrStdin::Stdin => std::borrow::Cow::Borrowed("<stdin>"),
        }
    }
}

impl From<&str> for FilePathOrStdin {
    fn from(s: &str) -> Self {
        if s == "-" {
            FilePathOrStdin::Stdin
        } else {
            FilePathOrStdin::FilePath(std::path::PathBuf::from(s))
        }
    }
}

impl From<char> for FilePathOrStdin {
    fn from(c: char) -> Self {
        if c == '-' {
            FilePathOrStdin::Stdin
        } else {
            FilePathOrStdin::FilePath(std::path::PathBuf::from(c.to_string()))
        }
    }
}

impl AsRef<std::path::Path> for FilePathOrStdin {
    fn as_ref(&self) -> &std::path::Path {
        match self {
            FilePathOrStdin::FilePath(path_buf) => path_buf.as_ref(),
            FilePathOrStdin::Stdin => unreachable!(),
        }
    }
}

impl std::fmt::Display for FilePathOrStdin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilePathOrStdin::FilePath(path_buf) => write!(f, "{}", path_buf.display()),
            FilePathOrStdin::Stdin => write!(f, "[STDIN]"),
        }
    }
}

impl From<std::path::PathBuf> for FilePathOrStdin {
    fn from(p: std::path::PathBuf) -> Self {
        FilePathOrStdin::FilePath(p)
    }
}

// clone but not explicit
impl From<&FilePathOrStdin> for FilePathOrStdin {
    fn from(p: &FilePathOrStdin) -> Self {
        match p {
            FilePathOrStdin::FilePath(pb) => FilePathOrStdin::FilePath(pb.clone()),
            FilePathOrStdin::Stdin => FilePathOrStdin::Stdin,
        }
    }
}
