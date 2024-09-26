#[cfg(test)]
mod tests;

use crate::parser::{
    Directive, DirectiveNode, JournalCstNode, JournalFile, SingleLineComment,
    TransactionEntry, TransactionNode,
};

pub fn format_content(nodes: &JournalFile) -> String {
    let mut formatted = String::new();

    for node in nodes {
        match node {
            JournalCstNode::SingleLineComment(SingleLineComment {
                content,
                prefix,
                colno,
                ..
            }) => {
                formatted.push_str(&format!(
                    "{}{}{}\n",
                    " ".repeat(*colno - 1),
                    *prefix as u8 as char,
                    content
                ));
            }
            JournalCstNode::EmptyLine { .. } => {
                formatted.push('\n');
            }
            JournalCstNode::MultilineComment { content, .. } => {
                formatted.push_str(&format!("comment\n{}end comment\n", content));
            }
            JournalCstNode::DirectivesGroup {
                content,
                max_name_content_len,
                ..
            } => {
                for node in content {
                    match node {
                        DirectiveNode::Directive(Directive {
                            name,
                            content,
                            comment,
                            ..
                        }) => {
                            formatted.push_str(&format!(
                                "{} {}{}\n",
                                name,
                                content,
                                match comment {
                                    Some(comment) => {
                                        format!(
                                            "{}{}{}",
                                            " ".repeat(
                                                *max_name_content_len - name.len() - content.len()
                                                    + 2
                                            ),
                                            comment.prefix as u8 as char,
                                            comment.content
                                        )
                                    }
                                    None => String::new(),
                                },
                            ));
                        }
                        DirectiveNode::SingleLineComment(SingleLineComment {
                            content,
                            prefix,
                            ..
                        }) => {
                            formatted.push_str(&format!(
                                "{}{}{}\n",
                                " ".repeat(*max_name_content_len + 3),
                                *prefix as u8 as char,
                                content
                            ));
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
                max_entry_value_len,
                max_entry_units_len,
                max_entry_decimal_len,
                max_entry_after_decimal_len,
            } => {
                let title_comment_padding = (title.len() + 2).max(
                    first_entry_indent
                        + max_entry_name_len
                        + 2
                        + max_entry_units_len
                        + max_entry_decimal_len
                        + max_entry_after_decimal_len
                        + 2
                        + if *max_entry_decimal_len > 0 { 1 } else { 0 },
                );
                formatted.push_str(&format!(
                    "{}{}\n",
                    title.trim(),
                    match title_comment {
                        Some(comment) => {
                            format!(
                                "{}{}{}",
                                " ".repeat(title_comment_padding - title.len()),
                                comment.prefix as u8 as char,
                                comment.content
                            )
                        }
                        None => String::new(),
                    }
                ));

                for entry in entries {
                    match entry {
                        TransactionNode::TransactionEntry(
                            TransactionEntry {
                                name,
                                value,
                                value_units_len,
                                value_decimal_len,
                                comment,
                            },
                        ) => {
                            let separation_with_value = if value.len() == 0 && comment.is_none() {
                                0  // no value nor comment, don't generate trailing spaces
                            } else {
                                max_entry_name_len - name.len()
                                    + max_entry_units_len
                                    + 2
                                    + if *value_decimal_len > 0 { 1 } else { 0 }
                                    - value_units_len
                            };
                            formatted.push_str(&format!(
                                "{}{}{}{}{}\n",
                                " ".repeat(*first_entry_indent),
                                name,
                                " ".repeat(separation_with_value),
                                value,
                                match comment {
                                    Some(comment) => {
                                        let comment_separation = title_comment_padding
                                            - value.len()
                                            - separation_with_value
                                            - name.len()
                                            - first_entry_indent;
                                        format!(
                                            "{}{}{}",
                                            " ".repeat(comment_separation),
                                            comment.prefix as u8 as char,
                                            comment.content
                                        )
                                    }
                                    None => String::new(),
                                },
                            ));
                        }
                        TransactionNode::SingleLineComment(
                            SingleLineComment {
                                content, prefix, ..
                            },
                        ) => {
                            formatted.push_str(&format!(
                                "{}{}{}\n",
                                " ".repeat(*first_entry_indent),
                                *prefix as u8 as char,
                                content
                            ));
                        }
                    }
                }
            }
        }
    }

    formatted
}
