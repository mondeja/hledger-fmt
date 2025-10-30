#[cfg(test)]
mod tests;
use crate::Vec;

use crate::parser::{
    Directive, DirectiveNode, JournalCstNode, JournalFile, SingleLineComment, TransactionNode,
};

pub struct FormatJournalOptions {
    estimated_length: usize,
    entry_spacing: usize,
}

impl Default for FormatJournalOptions {
    fn default() -> Self {
        Self {
            estimated_length: 1024,
            entry_spacing: {
                let compile_time_value = option_env!("HLEDGER_FMT_ENTRY_SPACING")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(2);

                #[cfg(feature = "env")]
                {
                    std::env::var("HLEDGER_FMT_ENTRY_SPACING")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(compile_time_value)
                }

                #[cfg(not(feature = "env"))]
                {
                    compile_time_value
                }
            },
        }
    }
}

impl FormatJournalOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_estimated_length(mut self, estimated_length: usize) -> Self {
        self.estimated_length = estimated_length;
        self
    }

    pub fn with_entry_spacing(mut self, entry_spacing: usize) -> Self {
        self.entry_spacing = entry_spacing;
        self
    }

    #[must_use]
    pub fn entry_spacing(&self) -> usize {
        self.entry_spacing
    }
}

#[cfg(test)]
fn format_content(nodes: &JournalFile) -> Vec<u8> {
    format_content_with_options(nodes, &FormatJournalOptions::default())
}

pub(crate) fn format_content_with_options(
    nodes: &JournalFile,
    opts: &FormatJournalOptions,
) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(opts.estimated_length);
    format_nodes(nodes, &mut buffer, opts.entry_spacing);
    buffer
}

fn format_nodes(nodes: &JournalFile, buffer: &mut Vec<u8>, entry_spacing: usize) {
    #[cfg(any(test, feature = "tracing"))]
    {
        let span = tracing::span!(tracing::Level::TRACE, "format_nodes");
        let _enter = span.enter();
        tracing::trace!("nodes={:#?}", nodes);
    }

    for node in nodes {
        match node {
            JournalCstNode::SingleLineComment(SingleLineComment {
                content,
                prefix,
                indent,
                ..
            }) => {
                spaces::extend(buffer, *indent as usize);
                buffer.push(*prefix as u8);
                buffer.extend_from_slice(content);
                buffer.push(b'\n');
            }
            JournalCstNode::EmptyLine => {
                buffer.push(b'\n');
            }
            JournalCstNode::MultilineComment { content, .. } => {
                buffer.extend_from_slice(b"comment\n");
                buffer.extend_from_slice(content);
                buffer.extend_from_slice(b"end comment\n");
            }
            JournalCstNode::DirectivesGroup {
                nodes,
                max_name_content_len,
                ..
            } => {
                for node in nodes {
                    match node {
                        DirectiveNode::Directive(Directive {
                            name,
                            content,
                            comment,
                            name_chars_count,
                            content_chars_count,
                            ..
                        }) => {
                            buffer.extend_from_slice(name);
                            buffer.push(b' ');
                            buffer.extend_from_slice(content);

                            if let Some(comment) = comment {
                                spaces::extend(
                                    buffer,
                                    2 + *max_name_content_len as usize
                                        - *name_chars_count as usize
                                        - *content_chars_count as usize,
                                );
                                buffer.push(comment.prefix as u8);
                                buffer.extend_from_slice(&comment.content);
                            }
                            buffer.push(b'\n');
                        }
                        DirectiveNode::Subdirective(content) => {
                            spaces::extend(buffer, 2);
                            buffer.extend_from_slice(content);
                            buffer.push(b'\n');
                        }
                        DirectiveNode::SingleLineComment(SingleLineComment {
                            content,
                            prefix,
                            ..
                        }) => {
                            spaces::extend(buffer, *max_name_content_len as usize + 3);
                            buffer.push(*prefix as u8);
                            buffer.extend_from_slice(content);
                            buffer.push(b'\n');
                        }
                    }
                }
            }
            JournalCstNode::Transaction {
                title,
                title_comment,
                entries,
                first_entry_indent,
                max_entry_name_len,
                max_entry_value_first_part_before_decimals_len,
                max_entry_value_first_part_after_decimals_len,
                max_entry_value_first_separator_len,
                max_entry_value_second_part_before_decimals_len,
                max_entry_value_second_part_after_decimals_len,
                max_entry_value_second_separator_len,
                max_entry_value_third_part_before_decimals_len,
                max_entry_value_third_part_after_decimals_len,
            } => {
                buffer.extend_from_slice(title);
                if let Some(comment) = title_comment {
                    spaces::extend(buffer, 2);
                    buffer.push(comment.prefix as u8);
                    buffer.extend_from_slice(&comment.content);
                }
                buffer.push(b'\n');

                // Cache title_chars_count outside the loop since title doesn't change
                let title_chars_count = title.chars_count();

                for entry in entries {
                    match entry {
                        TransactionNode::TransactionEntry(inner) => {
                            let e = inner.as_ref();

                            if let Some(ref comment) = e.comment {
                                // Use cached chars_count values
                                let after_decimals_chars_count =
                                    if !e.value_second_separator.is_empty() {
                                        e.value_third_part_after_decimals_chars_count as usize
                                    } else if !e.value_first_separator.is_empty() {
                                        e.value_second_part_after_decimals_chars_count as usize
                                    } else {
                                        e.value_first_part_after_decimals_chars_count as usize
                                    };

                                let mut entry_line_buffer = Vec::with_capacity(e.name.len() + 32);
                                extend_entry(
                                    &mut entry_line_buffer,
                                    e,
                                    *first_entry_indent,
                                    *max_entry_name_len,
                                    *max_entry_value_first_part_before_decimals_len,
                                    *max_entry_value_first_part_after_decimals_len,
                                    *max_entry_value_first_separator_len,
                                    *max_entry_value_second_part_before_decimals_len,
                                    *max_entry_value_second_part_after_decimals_len,
                                    *max_entry_value_second_separator_len,
                                    *max_entry_value_third_part_before_decimals_len,
                                    entry_spacing,
                                );

                                let comment_separation = if !e.value_second_separator.is_empty() {
                                    entry_spacing
                                        + *max_entry_value_third_part_after_decimals_len as usize
                                        - after_decimals_chars_count
                                } else if !e.value_first_separator.is_empty() {
                                    entry_spacing
                                        + *max_entry_value_second_part_after_decimals_len as usize
                                        - after_decimals_chars_count
                                } else {
                                    entry_spacing
                                        + *max_entry_value_first_part_after_decimals_len as usize
                                        - after_decimals_chars_count
                                };

                                let entry_line_chars_count =
                                    crate::byte_str::utf8_chars_count(&entry_line_buffer);

                                buffer.append(&mut entry_line_buffer);

                                let n_spaces = if title_chars_count + 2 > entry_line_chars_count + 2
                                {
                                    title_chars_count + 2 - entry_line_chars_count
                                } else {
                                    comment_separation
                                };
                                spaces::extend(buffer, n_spaces);
                                buffer.push(comment.prefix as u8);
                                buffer.extend_from_slice(&comment.content);
                            } else {
                                extend_entry(
                                    buffer,
                                    e,
                                    *first_entry_indent,
                                    *max_entry_name_len,
                                    *max_entry_value_first_part_before_decimals_len,
                                    *max_entry_value_first_part_after_decimals_len,
                                    *max_entry_value_first_separator_len,
                                    *max_entry_value_second_part_before_decimals_len,
                                    *max_entry_value_second_part_after_decimals_len,
                                    *max_entry_value_second_separator_len,
                                    *max_entry_value_third_part_before_decimals_len,
                                    entry_spacing,
                                );
                            }
                            buffer.push(b'\n');
                        }
                        TransactionNode::SingleLineComment(SingleLineComment {
                            content,
                            prefix,
                            ..
                        }) => {
                            spaces::extend(buffer, *first_entry_indent as usize);
                            buffer.push(*prefix as u8);
                            buffer.extend_from_slice(content);
                            buffer.push(b'\n');
                        }
                    }
                }
            }
        }
    }

    #[cfg(any(test, feature = "tracing"))]
    {
        let span = tracing::span!(tracing::Level::TRACE, "format_nodes(formatted)");
        let _enter = span.enter();

        tracing::trace!("buffer=\"{}\"", String::from_utf8_lossy(buffer));
    }
}

#[allow(clippy::too_many_arguments)]
fn extend_entry(
    buffer: &mut Vec<u8>,
    entry: &crate::parser::TransactionEntry,
    first_entry_indent: u16,
    max_entry_name_len: u16,
    max_entry_value_first_part_before_decimals_len: u16,
    max_entry_value_first_part_after_decimals_len: u16,
    max_entry_value_first_separator_len: u16,
    max_entry_value_second_part_before_decimals_len: u16,
    max_entry_value_second_part_after_decimals_len: u16,
    max_entry_value_second_separator_len: u16,
    max_entry_value_third_part_before_decimals_len: u16,
    entry_spacing: usize,
) {
    spaces::extend(buffer, first_entry_indent as usize);
    buffer.extend_from_slice(&entry.name);
    if !entry.value_first_part_before_decimals.is_empty() {
        let n_spaces = entry_spacing + max_entry_name_len as usize
            - entry.name_chars_count as usize
            + max_entry_value_first_part_before_decimals_len as usize
            - entry.value_first_part_before_decimals_chars_count as usize;
        spaces::extend(buffer, n_spaces);
    }
    buffer.extend_from_slice(&entry.value_first_part_before_decimals);
    buffer.extend_from_slice(&entry.value_first_part_after_decimals);

    if !entry.value_first_separator.is_empty() {
        let n_spaces = entry_spacing + max_entry_value_first_part_after_decimals_len as usize
            - entry.value_first_part_after_decimals_chars_count as usize;
        spaces::extend(buffer, n_spaces);
    }
    buffer.extend_from_slice(&entry.value_first_separator);
    if !entry.value_second_part_before_decimals.is_empty() {
        let value_first_separator_len = entry.value_first_separator.len();
        let n_spaces = entry_spacing + max_entry_value_first_separator_len as usize
            - value_first_separator_len
            + max_entry_value_second_part_before_decimals_len as usize
            - entry.value_second_part_before_decimals_chars_count as usize;
        spaces::extend(buffer, n_spaces);
    }
    buffer.extend_from_slice(&entry.value_second_part_before_decimals);
    buffer.extend_from_slice(&entry.value_second_part_after_decimals);

    if !entry.value_second_separator.is_empty() {
        let n_spaces = entry_spacing + max_entry_value_second_part_after_decimals_len as usize
            - entry.value_second_part_after_decimals_chars_count as usize;
        spaces::extend(buffer, n_spaces);
    }
    buffer.extend_from_slice(&entry.value_second_separator);
    if !entry.value_third_part_before_decimals.is_empty() {
        let value_second_separator_len = entry.value_second_separator.len();
        let n_spaces = entry_spacing + max_entry_value_second_separator_len as usize
            - value_second_separator_len
            + max_entry_value_third_part_before_decimals_len as usize
            - entry.value_third_part_before_decimals_chars_count as usize;
        spaces::extend(buffer, n_spaces);
    }
    buffer.extend_from_slice(&entry.value_third_part_before_decimals);
    buffer.extend_from_slice(&entry.value_third_part_after_decimals);
}

mod spaces {
    use crate::Vec;

    /// Generate a compile-time array of spaces of size N,
    /// to avoid allocating strings at runtime using " ".repeat(N).
    const fn make_spaces<const N: usize>() -> [u8; N] {
        [b' '; N]
    }

    const SPACES_64: [u8; 64] = make_spaces::<64>();

    pub fn extend(buffer: &mut Vec<u8>, n: usize) {
        // Fast paths for common values to avoid slice operations overhead
        match n {
            0 => (),                // No-op for zero spaces
            1 => buffer.push(b' '), // Direct push for single space
            2..=64 => {
                buffer.extend_from_slice(&SPACES_64[..n]);
            }
            _ => {
                let old_len = buffer.len();
                buffer.resize(old_len + n, b' ');
            }
        }
    }
}
