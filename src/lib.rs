#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
pub(crate) use alloc::{boxed::Box, format, string::String, vec::Vec};

#[cfg(feature = "std")]
pub(crate) use std::{boxed::Box, format, string::String, vec::Vec};

mod byte_str;
#[doc(hidden)]
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod file_path;
mod formatter;
mod parser;
#[cfg(any(test, feature = "tracing"))]
mod tracing;

pub use formatter::FormatJournalOptions;
pub use parser::errors::SyntaxError;

/// Format an hledger journal string file content as a String.
pub fn format_journal(content: &str) -> Result<String, SyntaxError> {
    let buffer = format_journal_bytes(content.as_bytes())?;
    // SAFETY: The formatter only outputs valid UTF-8 since it only writes:
    // 1. Slices from the valid UTF-8 input
    // 2. ASCII characters (spaces, newlines, comment prefixes)
    let formatted = String::from_utf8(buffer)
        .expect("formatter should only produce valid UTF-8");
    Ok(formatted)
}

/// Format an hledger journal string file content as a String with specified options.
pub fn format_journal_with_options(
    content: &str,
    options: formatter::FormatJournalOptions,
) -> Result<String, SyntaxError> {
    let parsed: Vec<parser::JournalCstNode<'_>> = parser::parse_content(content.as_bytes())?;
    let merged_options = options.with_estimated_length(content.len());
    let formatted_bytes = formatter::format_content_with_options(&parsed, &merged_options);
    // SAFETY: The formatter only outputs valid UTF-8 since it only writes:
    // 1. Slices from the valid UTF-8 input
    // 2. ASCII characters (spaces, newlines, comment prefixes)
    let formatted = String::from_utf8(formatted_bytes)
        .expect("formatter should only produce valid UTF-8");
    Ok(formatted)
}

/// Format an hledger journal file content as bytes.
pub fn format_journal_bytes(content: &[u8]) -> Result<Vec<u8>, SyntaxError> {
    let parsed = parser::parse_content(content)?;
    let opts = formatter::FormatJournalOptions::new().with_estimated_length(content.len());
    Ok(formatter::format_content_with_options(&parsed, &opts))
}

/// Format an hledger journal file content as bytes with specified options.
pub fn format_journal_bytes_with_options(
    content: &[u8],
    options: formatter::FormatJournalOptions,
) -> Result<Vec<u8>, SyntaxError> {
    let parsed = parser::parse_content(content)?;
    let merged_options = options.with_estimated_length(content.len());
    let formatted = formatter::format_content_with_options(&parsed, &merged_options);
    Ok(formatted)
}

#[cfg(feature = "bench")]
pub fn format_parsed_journal(parsed: &parser::JournalFile) -> Result<Vec<u8>, SyntaxError> {
    let format_opts = formatter::FormatJournalOptions::new();
    let formatted = formatter::format_content_with_options(parsed, &format_opts);
    Ok(formatted)
}

#[cfg(feature = "bench")]
pub use parser::parse_content;
