#[cfg(test)]
mod tests;

use crate::parser::{
    Directive, DirectiveNode, JournalCstNode, JournalFile, SingleLineComment, TransactionEntry,
    TransactionNode,
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
                max_entry_value_first_part_units_len,
                max_entry_value_first_part_numeric_units_len,
                max_entry_value_first_part_decimal_len,
                max_entry_value_first_separator_len,
                max_entry_value_second_part_units_len,
                max_entry_value_second_part_decimal_len,
                max_entry_value_second_part_numeric_units_len,
                max_entry_value_second_separator_len,
                max_entry_value_third_part_units_len,
                max_entry_value_third_part_decimal_len,
                max_entry_value_third_part_numeric_units_len,
            } => {
                let title_comment_padding = (title.len() + 2).max(
                    first_entry_indent
                        + max_entry_name_len
                        + 2
                        + max_entry_value_first_part_units_len
                        + max_entry_value_first_part_decimal_len
                        + max_entry_value_first_separator_len
                        + max_entry_value_second_part_units_len
                        + max_entry_value_second_part_decimal_len
                        + max_entry_value_second_separator_len
                        + max_entry_value_third_part_units_len
                        + max_entry_value_third_part_decimal_len
                        + 2,
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
                        TransactionNode::TransactionEntry(TransactionEntry {
                            name,
                            value_first_part_units,
                            value_first_part_numeric_units,
                            value_first_part_decimal,
                            value_first_separator,
                            value_second_part_units,
                            value_second_part_numeric_units,
                            value_second_part_decimal,
                            value_second_separator,
                            value_third_part_units,
                            value_third_part_numeric_units,
                            value_third_part_decimal,
                            comment,
                        }) => {
                            println!();

                            let mut first_part_numeric_units_trailing_com_len = 0;
                            for c in value_first_part_units.chars().rev() {
                                if c.is_digit(10) {
                                    break;
                                }
                                first_part_numeric_units_trailing_com_len += 1;
                            }

                            let mut first_part_numeric_units_leading_com_len = 0;
                            for c in value_first_part_units.chars() {
                                if c.is_digit(10) {
                                    break;
                                }
                                first_part_numeric_units_leading_com_len += 1;
                            }

                            let mut second_part_numeric_units_trailing_com_len = 0;
                            for c in value_second_part_units.chars().rev() {
                                if c.is_digit(10) {
                                    break;
                                }
                                second_part_numeric_units_trailing_com_len += 1;
                            }

                            let mut second_part_numeric_units_leading_com_len = 0;
                            for c in value_second_part_units.chars() {
                                if c.is_digit(10) {
                                    break;
                                }
                                second_part_numeric_units_leading_com_len += 1;
                            }

                            let mut third_part_numeric_units_leading_com_len = 0;
                            for c in value_third_part_units.chars() {
                                if c.is_digit(10) {
                                    break;
                                }
                                third_part_numeric_units_leading_com_len += 1;
                            }

                            /*
                            let separation = 2 + max_entry_value_first_part_decimal_len
                                - value_first_part_decimal.chars().count();
                            println!(
                                "2 + max_entry_value_first_part_decimal_len: 2 + {}",
                                max_entry_value_first_part_decimal_len
                            );
                            println!(
                                "- value_first_part_decimal.len(): -{}",
                                value_first_part_decimal.chars().count()
                            );
                            println!(
                                "separation: {}",
                                separation
                            );*/

                            formatted.push_str(&format!(
                                "{}{}{}{}{}{}{}{}{}{}{}{}{}\n",
                                " ".repeat(*first_entry_indent),
                                name,
                                " ".repeat(
                                    3 + max_entry_name_len  // 3 because of -leading
                                        - name.len()
                                        - first_part_numeric_units_leading_com_len
                                        + max_entry_value_first_part_numeric_units_len
                                        - value_first_part_numeric_units.chars().count()
                                ),
                                format!("{}{}", value_first_part_units, value_first_part_decimal),
                                " ".repeat(
                                    3 + max_entry_value_first_part_decimal_len
                                        - value_first_part_decimal.chars().count()
                                        - first_part_numeric_units_trailing_com_len
                                ),
                                value_first_separator,
                                " ".repeat(
                                    3 + max_entry_value_first_separator_len
                                        - value_first_separator.len()
                                        - second_part_numeric_units_leading_com_len
                                        + max_entry_value_second_part_numeric_units_len
                                        - value_second_part_numeric_units.chars().count()
                                ),
                                format!("{}{}", value_second_part_units, value_second_part_decimal),
                                " ".repeat(
                                    3 + max_entry_value_second_part_decimal_len
                                        - value_second_part_decimal.chars().count()
                                        - second_part_numeric_units_trailing_com_len
                                ),
                                value_second_separator,
                                " ".repeat(
                                    3 + max_entry_value_second_separator_len
                                        - value_second_separator.len()
                                        - third_part_numeric_units_leading_com_len
                                        + max_entry_value_third_part_numeric_units_len
                                        - value_third_part_numeric_units.chars().count()
                                ),
                                format!("{}{}", value_third_part_units, value_third_part_decimal),
                                match comment {
                                    Some(comment) => {
                                        let comment_separation = 2; /*title_comment_padding
                                                                    - value.len()
                                                                    - separation_with_value
                                                                    - name.len()
                                                                    - first_entry_indent;*/
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
                        TransactionNode::SingleLineComment(SingleLineComment {
                            content,
                            prefix,
                            ..
                        }) => {
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
