#[doc(hidden)]
pub mod cli;
mod common;
mod formatter;
mod parser;

pub use parser::errors::SyntaxError;

/// Format an hledger journal file content.
pub fn format_journal(content: &str) -> Result<String, SyntaxError> {
    let parsed = parser::parse_content(content)?;
    let format_opts = formatter::FormatContentOptions::new().with_estimated_length(content.len());
    let formatted = formatter::format_content_with_options(&parsed, &format_opts);
    Ok(formatted)
}
