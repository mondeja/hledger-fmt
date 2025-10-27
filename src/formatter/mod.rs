#[cfg(test)]
mod tests;

use crate::parser::{
    Directive, DirectiveNode, JournalCstNode, JournalFile, SingleLineComment, TransactionNode,
};

const TRANSACTION_ENTRY_VALUE_SPACING: usize = 2;

#[derive(Default)]
pub(crate) struct FormatContentOptions {
    estimated_length: usize,
}

impl FormatContentOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_estimated_length(mut self, estimated_length: usize) -> Self {
        self.estimated_length = estimated_length;
        self
    }
}

#[cfg(test)]
fn format_content(nodes: &JournalFile) -> Vec<u8> {
    format_content_with_options(nodes, &FormatContentOptions::default())
}

pub(crate) fn format_content_with_options(
    nodes: &JournalFile,
    opts: &FormatContentOptions,
) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(opts.estimated_length);
    format_nodes(nodes, &mut buffer);
    buffer
}

fn format_nodes(nodes: &JournalFile, buffer: &mut Vec<u8>) {
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
                spaces::extend(buffer, *indent);
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
                            ..
                        }) => {
                            buffer.extend_from_slice(name);
                            buffer.push(b' ');
                            buffer.extend_from_slice(content);

                            if let Some(comment) = comment {
                                spaces::extend(
                                    buffer,
                                    2 + *max_name_content_len
                                        - name.chars_count()
                                        - content.chars_count(),
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
                            spaces::extend(buffer, *max_name_content_len + 3);
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

                for entry in entries {
                    match entry {
                        TransactionNode::TransactionEntry(inner) => {
                            let e = inner.as_ref();

                            if let Some(ref comment) = e.comment {
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
                                );

                                let comment_separation = if !e.value_second_separator.is_empty() {
                                    TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_third_part_after_decimals_len
                                        - e.value_third_part_after_decimals.chars_count()
                                } else if !e.value_first_separator.is_empty() {
                                    TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_second_part_after_decimals_len
                                        - e.value_second_part_after_decimals.chars_count()
                                } else {
                                    TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_first_part_after_decimals_len
                                        - e.value_first_part_after_decimals.chars_count()
                                };

                                let entry_line_chars_count =
                                    crate::byte_str::utf8_chars_count(&entry_line_buffer);

                                buffer.append(&mut entry_line_buffer);

                                let title_chars_count = title.chars_count();

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
                                );
                            }
                            buffer.push(b'\n');
                        }
                        TransactionNode::SingleLineComment(SingleLineComment {
                            content,
                            prefix,
                            ..
                        }) => {
                            spaces::extend(buffer, *first_entry_indent);
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
    first_entry_indent: usize,
    max_entry_name_len: usize,
    max_entry_value_first_part_before_decimals_len: usize,
    max_entry_value_first_part_after_decimals_len: usize,
    max_entry_value_first_separator_len: usize,
    max_entry_value_second_part_before_decimals_len: usize,
    max_entry_value_second_part_after_decimals_len: usize,
    max_entry_value_second_separator_len: usize,
    max_entry_value_third_part_before_decimals_len: usize,
) {
    spaces::extend(buffer, first_entry_indent);
    buffer.extend_from_slice(&entry.name);
    if !entry.value_first_part_before_decimals.is_empty() {
        let name_len = entry.name.chars_count();
        let before_decimals_len = entry.value_first_part_before_decimals.chars_count();
        let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING + max_entry_name_len - name_len
            + max_entry_value_first_part_before_decimals_len
            - before_decimals_len;
        spaces::extend(buffer, n_spaces);
    }
    buffer.extend_from_slice(&entry.value_first_part_before_decimals);
    buffer.extend_from_slice(&entry.value_first_part_after_decimals);

    if !entry.value_first_separator.is_empty() {
        let after_decimals_len = entry.value_first_part_after_decimals.chars_count();
        let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
            + max_entry_value_first_part_after_decimals_len
            - after_decimals_len;
        spaces::extend(buffer, n_spaces);
    }
    buffer.extend_from_slice(&entry.value_first_separator);
    if !entry.value_second_part_before_decimals.is_empty() {
        let value_first_separator_len = entry.value_first_separator.len();
        let value_second_part_before_decimals_len =
            entry.value_second_part_before_decimals.chars_count();
        let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING + max_entry_value_first_separator_len
            - value_first_separator_len
            + max_entry_value_second_part_before_decimals_len
            - value_second_part_before_decimals_len;
        spaces::extend(buffer, n_spaces);
    }
    buffer.extend_from_slice(&entry.value_second_part_before_decimals);
    buffer.extend_from_slice(&entry.value_second_part_after_decimals);

    if !entry.value_second_separator.is_empty() {
        let value_second_part_after_decimals_len =
            entry.value_second_part_after_decimals.chars_count();
        let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
            + max_entry_value_second_part_after_decimals_len
            - value_second_part_after_decimals_len;
        spaces::extend(buffer, n_spaces);
    }
    buffer.extend_from_slice(&entry.value_second_separator);
    if !entry.value_third_part_before_decimals.is_empty() {
        let value_second_separator_len = entry.value_second_separator.len();
        let value_third_part_before_decimals_len =
            entry.value_third_part_before_decimals.chars_count();
        let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING + max_entry_value_second_separator_len
            - value_second_separator_len
            + max_entry_value_third_part_before_decimals_len
            - value_third_part_before_decimals_len;
        spaces::extend(buffer, n_spaces);
    }
    buffer.extend_from_slice(&entry.value_third_part_before_decimals);
    buffer.extend_from_slice(&entry.value_third_part_after_decimals);
}

mod spaces {
    /// Generate a compile-time array of spaces of size N,
    /// to avoid allocating strings at runtime using " ".repeat(N).
    const fn make_spaces<const N: usize>() -> [u8; N] {
        [b' '; N]
    }

    const SPACES_64: [u8; 64] = make_spaces::<64>();

    pub fn extend(buffer: &mut Vec<u8>, n: usize) {
        if n <= 64 {
            buffer.extend_from_slice(&SPACES_64[..n]);
        } else {
            let old_len = buffer.len();
            buffer.resize(old_len + n, b' ');
        }
    }
}
