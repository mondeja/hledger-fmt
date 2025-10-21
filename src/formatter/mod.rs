#[cfg(test)]
mod tests;

use crate::parser::{
    Directive, DirectiveNode, JournalCstNode, JournalFile, SingleLineComment, TransactionEntry,
    TransactionNode,
};
use core::fmt::Write;

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
fn format_content(nodes: &JournalFile) -> String {
    format_content_with_options(nodes, &FormatContentOptions::default())
}

pub(crate) fn format_content_with_options(
    nodes: &JournalFile,
    opts: &FormatContentOptions,
) -> String {
    let mut formatted = String::with_capacity(opts.estimated_length);

    for node in nodes {
        match node {
            JournalCstNode::SingleLineComment(SingleLineComment {
                content,
                prefix,
                colno,
                ..
            }) => {
                _ = writeln!(
                    formatted,
                    "{}{}{}",
                    " ".repeat(*colno - 1),
                    *prefix as u8 as char,
                    content
                );
            }
            JournalCstNode::EmptyLine { .. } => {
                formatted.push('\n');
            }
            JournalCstNode::MultilineComment { content, .. } => {
                _ = writeln!(formatted, "comment\n{content}end comment");
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
                            _ = write!(formatted, "{name} {content}");

                            if let Some(comment) = comment {
                                _ = write!(
                                    formatted,
                                    "{}{}{}",
                                    " ".repeat(
                                        2 + max_name_content_len
                                            - name.chars().count()
                                            - content.chars().count()
                                    ),
                                    comment.prefix as u8 as char,
                                    comment.content
                                );
                            }

                            formatted.push('\n');
                        }
                        DirectiveNode::Subdirective(content) => {
                            _ = writeln!(formatted, "  {content}");
                        }
                        DirectiveNode::SingleLineComment(SingleLineComment {
                            content,
                            prefix,
                            ..
                        }) => {
                            _ = writeln!(
                                formatted,
                                "{}{}{}",
                                " ".repeat(*max_name_content_len + 3),
                                *prefix as u8 as char,
                                content
                            );
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
                _ = write!(formatted, "{}", title.trim());
                if let Some(comment) = title_comment {
                    _ = write!(
                        formatted,
                        "  {}{}",
                        comment.prefix as u8 as char, comment.content
                    );
                }
                formatted.push('\n');

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

                            let entry_line = format!(
                                "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                                " ".repeat(*first_entry_indent),
                                name,
                                if !value_first_part_before_decimals.is_empty() {
                                    let name_len = name.chars().count();
                                    let before_decimals_len =
                                        value_first_part_before_decimals.chars().count();
                                    let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_name_len
                                        - name_len
                                        + max_entry_value_first_part_before_decimals_len
                                        - before_decimals_len;
                                    " ".repeat(n_spaces)
                                } else {
                                    "".to_string()
                                },
                                value_first_part_before_decimals,
                                value_first_part_after_decimals,
                                if !value_first_separator.is_empty() {
                                    let after_decimals_len =
                                        value_first_part_after_decimals.chars().count();
                                    let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_first_part_after_decimals_len
                                        - after_decimals_len;
                                    " ".repeat(n_spaces)
                                } else {
                                    "".to_string()
                                },
                                value_first_separator,
                                if !value_second_part_before_decimals.is_empty() {
                                    let value_first_separator_len = value_first_separator.len();
                                    let value_second_part_before_decimals_len =
                                        value_second_part_before_decimals.chars().count();
                                    let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_first_separator_len
                                        - value_first_separator_len
                                        + max_entry_value_second_part_before_decimals_len
                                        - value_second_part_before_decimals_len;
                                    " ".repeat(n_spaces)
                                } else {
                                    "".to_string()
                                },
                                value_second_part_before_decimals,
                                value_second_part_after_decimals,
                                if !value_second_separator.is_empty() {
                                    let value_second_part_after_decimals_len =
                                        value_second_part_after_decimals.chars().count();
                                    let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_second_part_after_decimals_len
                                        - value_second_part_after_decimals_len;
                                    " ".repeat(n_spaces)
                                } else {
                                    "".to_string()
                                },
                                value_second_separator,
                                if !value_third_part_before_decimals.is_empty() {
                                    let value_second_separator_len = value_second_separator.len();
                                    let value_third_part_before_decimals_len =
                                        value_third_part_before_decimals.chars().count();
                                    let n_spaces = TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_second_separator_len
                                        - value_second_separator_len
                                        + max_entry_value_third_part_before_decimals_len
                                        - value_third_part_before_decimals_len;
                                    " ".repeat(n_spaces)
                                } else {
                                    "".to_string()
                                },
                                value_third_part_before_decimals,
                                value_third_part_after_decimals,
                            );

                            formatted.push_str(&entry_line);

                            if let Some(comment) = comment {
                                let comment_separation = if !value_second_separator.is_empty() {
                                    TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_third_part_after_decimals_len
                                        - value_third_part_after_decimals.chars().count()
                                } else if !value_first_separator.is_empty() {
                                    TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_second_part_after_decimals_len
                                        - value_second_part_after_decimals.chars().count()
                                } else {
                                    TRANSACTION_ENTRY_VALUE_SPACING
                                        + max_entry_value_first_part_after_decimals_len
                                        - value_first_part_after_decimals.chars().count()
                                };

                                let title_chars_count = title.chars().count();
                                let entry_line_chars_count = entry_line.chars().count();

                                _ = write!(
                                    formatted,
                                    "{}{}{}",
                                    " ".repeat(
                                        if title_chars_count + 2 > entry_line_chars_count + 2 {
                                            title_chars_count + 2 - entry_line_chars_count
                                        } else {
                                            comment_separation
                                        }
                                    ),
                                    comment.prefix as u8 as char,
                                    comment.content
                                );
                            };

                            formatted.push('\n');
                        }
                        TransactionNode::SingleLineComment(SingleLineComment {
                            content,
                            prefix,
                            ..
                        }) => {
                            _ = writeln!(
                                formatted,
                                "{}{}{}",
                                " ".repeat(*first_entry_indent),
                                *prefix as u8 as char,
                                content
                            );
                        }
                    }
                }
            }
        }
    }

    formatted
}
