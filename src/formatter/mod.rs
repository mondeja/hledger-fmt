#[cfg(test)]
mod tests;

use crate::{
    common::{leading_commodity_len_from_units, trailing_commodity_len_from_units},
    parser::{
        Directive, DirectiveNode, JournalCstNode, JournalFile, SingleLineComment, TransactionEntry,
        TransactionNode,
    },
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
                formatted.push_str(&format!("comment\n{content}end comment\n"));
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
                            formatted.push_str(&format!(
                                "{} {}{}\n",
                                name,
                                content,
                                match comment {
                                    Some(comment) => {
                                        format!(
                                            "{}{}{}",
                                            " ".repeat(
                                                2 + max_name_content_len
                                                    - name.chars().count()
                                                    - content.chars().count()
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
                max_entry_value_first_part_numeric_units_len,
                max_entry_value_first_part_decimal_len,
                max_entry_value_first_part_commodity_leading_len,
                max_entry_value_first_part_commodity_trailing_len,
                max_entry_value_first_separator_len,
                max_entry_value_second_part_decimal_len,
                max_entry_value_second_part_numeric_units_len,
                max_entry_value_second_separator_len,
                max_entry_value_third_part_decimal_len,
                max_entry_value_third_part_numeric_units_len,
            } => {
                formatted.push_str(&format!(
                    "{}{}\n",
                    title.trim(),
                    match title_comment {
                        Some(comment) => {
                            format!("  {}{}", comment.prefix as u8 as char, comment.content)
                        }
                        None => String::new(),
                    }
                ));

                for entry in entries {
                    match entry {
                        TransactionNode::TransactionEntry(inner) => {
                            let TransactionEntry {
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
                                ..
                            } = inner.as_ref();
                            let entry_line = format!(
                                "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                                " ".repeat(*first_entry_indent),
                                name,
                                if !value_first_part_units.is_empty()
                                    || !value_first_part_decimal.is_empty()
                                {
                                    " ".repeat(
                                        2 + max_entry_name_len - name.len()
                                            + max_entry_value_first_part_commodity_leading_len
                                            - leading_commodity_len_from_units(
                                                value_first_part_units,
                                            )
                                            + max_entry_value_first_part_numeric_units_len
                                            - value_first_part_numeric_units.chars().count(),
                                    )
                                } else {
                                    "".to_string()
                                },
                                value_first_part_units,
                                value_first_part_decimal,
                                if !value_first_separator.is_empty() {
                                    " ".repeat(
                                        3 + max_entry_value_first_part_decimal_len
                                            - value_first_part_decimal.chars().count()
                                            - trailing_commodity_len_from_units(
                                                value_first_part_units,
                                            ),
                                    )
                                } else {
                                    "".to_string()
                                },
                                value_first_separator,
                                if !value_second_part_units.is_empty()
                                    || !value_second_part_decimal.is_empty()
                                {
                                    " ".repeat(
                                        3 + max_entry_value_first_separator_len
                                            - value_first_separator.len()
                                            - leading_commodity_len_from_units(
                                                value_second_part_units,
                                            )
                                            + max_entry_value_second_part_numeric_units_len
                                            - value_second_part_numeric_units.chars().count(),
                                    )
                                } else {
                                    "".to_string()
                                },
                                value_second_part_units,
                                value_second_part_decimal,
                                if !value_second_separator.is_empty() {
                                    " ".repeat(if !value_first_separator.is_empty() {
                                        2 + max_entry_value_second_part_decimal_len
                                            - value_second_part_decimal.chars().count()
                                            - trailing_commodity_len_from_units(
                                                value_second_part_units,
                                            )
                                    } else {
                                        let pos =
                                            2 + max_entry_value_first_part_commodity_trailing_len;
                                        let neg = value_first_part_decimal.chars().count()
                                            + trailing_commodity_len_from_units(
                                                value_first_part_units,
                                            );
                                        if pos > neg {
                                            pos - neg
                                        } else {
                                            2
                                        }
                                    })
                                } else {
                                    "".to_string()
                                },
                                value_second_separator,
                                if !value_third_part_units.is_empty()
                                    || !value_third_part_decimal.is_empty()
                                {
                                    " ".repeat(
                                        3 + max_entry_value_second_separator_len
                                            - value_second_separator.len()
                                            - leading_commodity_len_from_units(
                                                value_third_part_units,
                                            )
                                            + max_entry_value_third_part_numeric_units_len
                                            - value_third_part_numeric_units.chars().count(),
                                    )
                                } else {
                                    "".to_string()
                                },
                                value_third_part_units,
                                value_third_part_decimal,
                            );

                            let comment_part = if let Some(comment) = comment {
                                let comment_separation = if !value_second_separator.is_empty() {
                                    2 + max_entry_value_third_part_decimal_len
                                        - value_third_part_decimal.chars().count()
                                        - trailing_commodity_len_from_units(value_third_part_units)
                                } else if !value_first_separator.is_empty() {
                                    2 + max_entry_value_second_part_decimal_len
                                        - value_second_part_decimal.chars().count()
                                        - trailing_commodity_len_from_units(value_second_part_units)
                                } else {
                                    2 + max_entry_value_first_part_decimal_len
                                        - value_first_part_decimal.chars().count()
                                };

                                format!(
                                    "{}{}{}",
                                    " ".repeat(
                                        if title.chars().count() + 2
                                            > entry_line.chars().count() + 2
                                        {
                                            title.chars().count() + 2 - entry_line.chars().count()
                                        } else {
                                            comment_separation
                                        }
                                    ),
                                    comment.prefix as u8 as char,
                                    comment.content
                                )
                            } else {
                                String::new()
                            };

                            formatted.push_str(&entry_line);
                            formatted.push_str(&comment_part);
                            formatted.push('\n');
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
