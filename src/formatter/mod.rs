#[cfg(test)]
mod tests;

use crate::parser::{
    Directive, DirectiveNode, JournalCstNode, JournalFile, SingleLineComment, TransactionEntry,
    TransactionNode,
};

const TRANSACTION_ENTRY_VALUE_SPACING: u8 = 2;

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
                spaces::extend(buffer, *indent as usize);
                buffer.push(*prefix as u8);
                buffer.extend_from_slice(content.as_ref());
                buffer.push(b'\n');
            }
            JournalCstNode::EmptyLine => {
                buffer.push(b'\n');
            }
            JournalCstNode::MultilineComment { content, .. } => {
                buffer.extend_from_slice(b"comment\n");
                buffer.extend_from_slice(content.as_ref());
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
                            buffer.extend_from_slice(name.as_ref());
                            buffer.push(b' ');
                            buffer.extend_from_slice(content.as_ref());

                            if let Some(comment) = comment {
                                spaces::extend(
                                    buffer,
                                    (2 + *max_name_content_len) as usize
                                        - name.chars_count()
                                        - content.chars_count(),
                                );
                                buffer.push(comment.prefix as u8);
                                buffer.extend_from_slice(comment.content.as_ref());
                            }
                            buffer.push(b'\n');
                        }
                        DirectiveNode::Subdirective(content) => {
                            spaces::extend(buffer, 2);
                            buffer.extend_from_slice(content.as_ref());
                            buffer.push(b'\n');
                        }
                        DirectiveNode::SingleLineComment(SingleLineComment {
                            content,
                            prefix,
                            ..
                        }) => {
                            spaces::extend(buffer, (*max_name_content_len + 3) as usize);
                            buffer.push(*prefix as u8);
                            buffer.extend_from_slice(content.as_ref());
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
                buffer.extend_from_slice(title.as_ref());
                if let Some(comment) = title_comment {
                    spaces::extend(buffer, 2);
                    buffer.push(comment.prefix as u8);
                    buffer.extend_from_slice(comment.content.as_ref());
                }
                buffer.push(b'\n');

                for entry in entries {
                    match entry {
                        TransactionNode::TransactionEntry(inner) => {
                            let TransactionEntry {
                                name,
                                value_first_part_before_decimals,
                                value_first_part_after_decimals,
                                value_first_separator,
                                value_second_part_before_decimals,
                                value_second_part_after_decimals,
                                value_second_separator,
                                value_third_part_before_decimals,
                                value_third_part_after_decimals,
                                comment,
                            } = inner.as_ref();

                            let mut entry_line_buffer = Vec::new();

                            spaces::extend(&mut entry_line_buffer, *first_entry_indent as usize);
                            entry_line_buffer.extend_from_slice(name.as_ref());
                            if !value_first_part_before_decimals.is_empty() {
                                let name_len = name.chars_count() as u8;
                                let before_decimals_len =
                                    value_first_part_before_decimals.chars_count() as u8;
                                let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING + max_entry_name_len
                                    - name_len
                                    + max_entry_value_first_part_before_decimals_len
                                    - before_decimals_len;
                                spaces::extend(&mut entry_line_buffer, n_spaces as usize);
                            }
                            entry_line_buffer
                                .extend_from_slice(value_first_part_before_decimals.as_ref());
                            entry_line_buffer
                                .extend_from_slice(value_first_part_after_decimals.as_ref());

                            if !value_first_separator.is_empty() {
                                let after_decimals_len =
                                    value_first_part_after_decimals.chars_count() as u8;
                                let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
                                    + max_entry_value_first_part_after_decimals_len
                                    - after_decimals_len;
                                spaces::extend(&mut entry_line_buffer, n_spaces as usize);
                            }
                            entry_line_buffer.extend_from_slice(value_first_separator.as_ref());
                            if !value_second_part_before_decimals.is_empty() {
                                let value_first_separator_len = value_first_separator.len() as u8;
                                let value_second_part_before_decimals_len =
                                    value_second_part_before_decimals.chars_count() as u8;
                                let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
                                    + max_entry_value_first_separator_len
                                    - value_first_separator_len
                                    + max_entry_value_second_part_before_decimals_len
                                    - value_second_part_before_decimals_len;
                                spaces::extend(&mut entry_line_buffer, n_spaces as usize);
                            }
                            entry_line_buffer
                                .extend_from_slice(value_second_part_before_decimals.as_ref());
                            entry_line_buffer
                                .extend_from_slice(value_second_part_after_decimals.as_ref());

                            if !value_second_separator.is_empty() {
                                let value_second_part_after_decimals_len =
                                    value_second_part_after_decimals.chars_count() as u8;
                                let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
                                    + max_entry_value_second_part_after_decimals_len
                                    - value_second_part_after_decimals_len;
                                spaces::extend(&mut entry_line_buffer, n_spaces as usize);
                            }
                            entry_line_buffer.extend_from_slice(value_second_separator.as_ref());
                            if !value_third_part_before_decimals.is_empty() {
                                let value_second_separator_len = value_second_separator.len() as u8;
                                let value_third_part_before_decimals_len =
                                    value_third_part_before_decimals.chars_count() as u8;
                                let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
                                    + max_entry_value_second_separator_len
                                    - value_second_separator_len
                                    + max_entry_value_third_part_before_decimals_len
                                    - value_third_part_before_decimals_len;
                                spaces::extend(&mut entry_line_buffer, n_spaces as usize);
                            }
                            entry_line_buffer
                                .extend_from_slice(value_third_part_before_decimals.as_ref());
                            entry_line_buffer
                                .extend_from_slice(value_third_part_after_decimals.as_ref());

                            if let Some(comment) = comment {
                                let comment_separation = if !value_second_separator.is_empty() {
                                    TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_third_part_after_decimals_len
                                        - value_third_part_after_decimals.chars_count() as u8
                                } else if !value_first_separator.is_empty() {
                                    TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_second_part_after_decimals_len
                                        - value_second_part_after_decimals.chars_count() as u8
                                } else {
                                    TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_first_part_after_decimals_len
                                        - value_first_part_after_decimals.chars_count() as u8
                                };

                                let entry_line_chars_count =
                                    String::from_utf8_lossy(&entry_line_buffer).chars().count();

                                buffer.append(&mut entry_line_buffer);

                                let title_chars_count = title.chars_count();

                                let n_spaces = if title_chars_count + 2 > entry_line_chars_count + 2
                                {
                                    title_chars_count + 2 - entry_line_chars_count
                                } else {
                                    comment_separation as usize
                                };
                                spaces::extend(buffer, n_spaces);
                                buffer.push(comment.prefix as u8);
                                buffer.extend_from_slice(comment.content.as_ref());
                            } else {
                                buffer.append(&mut entry_line_buffer);
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
                            buffer.extend_from_slice(content.as_ref());
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
            buffer.extend(std::iter::repeat(b' ').take(n));
        }
    }
}
