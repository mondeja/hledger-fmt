#[doc(hidden)]
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod file_path;
mod formatter;
mod parser;
mod byte_str;
#[cfg(any(test, feature = "tracing"))]
mod tracing;

pub use parser::errors::SyntaxError;

/// Format an hledger journal string file content as a String.
pub fn format_journal(content: &str) -> Result<String, SyntaxError> {
    let buffer = format_journal_bytes(content.as_bytes())?;
    let formatted = String::from_utf8(buffer).unwrap();
    Ok(formatted)
}

/// Format an hledger journal file content as bytes.
pub fn format_journal_bytes(content: &[u8]) -> Result<Vec<u8>, SyntaxError> {
    let parsed = parser::parse_content(content)?;
    let opts = formatter::FormatContentOptions::new()
        .with_estimated_length(content.len());
    Ok(formatter::format_content_with_options(&parsed, &opts))
}

#[cfg(feature = "bench")]
pub fn format_parsed_journal(
    parsed: &parser::JournalFile,
) -> Result<Vec<u8>, SyntaxError> {
    let format_opts = formatter::FormatContentOptions::new();
    let formatted = formatter::format_content_with_options(parsed, &format_opts);
    Ok(formatted)
}

#[cfg(feature = "bench")]
pub use parser::parse_content;
