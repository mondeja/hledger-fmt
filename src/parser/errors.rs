/// Syntax error occurring while parsing journal files.
#[derive(Debug, PartialEq)]
pub struct SyntaxError {
    pub lineno: usize,
    pub colno_start: usize,
    pub colno_end: usize,
    pub message: String,
    pub expected: &'static str,
}

/// Generate an error context similar to what hledger does.
pub fn build_error_context(error: &SyntaxError, content: &str, file_path: &str) -> String {
    let lines = content.lines().collect::<Vec<&str>>();
    let mut context = format!(
        "hlegder-fmt error: {}:{}:{}:\n",
        file_path, error.lineno, error.colno_start
    );

    let lineno_len = format!("{}", error.lineno).len();
    if error.lineno > 1 {
        context.push_str(&format!(
            "{} | {}\n",
            " ".repeat(lineno_len),
            lines[error.lineno - 2]
        ));
    }
    context.push_str(&format!("{} | {}\n", error.lineno, lines[error.lineno - 1]));
    context.push_str(&format!(
        "{} | {}{}\n",
        " ".repeat(lineno_len),
        " ".repeat(error.colno_start - 1),
        "^".repeat(error.colno_end - error.colno_start)
    ));
    context.push_str(&format!("{}\nExpected {}", error.message, error.expected));
    context
}
