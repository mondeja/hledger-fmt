pub mod errors;
#[cfg(test)]
mod tests;

use errors::SyntaxError;

/// A journal file
pub type JournalFile = Vec<JournalCstNode>;

/// Each node in a journal file
#[derive(Debug, PartialEq)]
pub enum JournalCstNode {
    /// An empty line
    EmptyLine {
        /// Line number in the file (1-indexed)
        lineno: usize,
    },

    /// Multiline comment
    MultilineComment {
        /// The comment content
        content: String,
        /// Starting line number in the file (1-indexed)
        lineno_start: usize,
        /// Ending line number in the file (1-indexed)
        lineno_end: usize,
    },

    SingleLineComment(SingleLineComment),

    /// Directives group
    DirectivesGroup {
        /// Directives in the group
        content: Vec<DirectiveNode>,
        /// Maximum length of the directive name + content
        max_name_content_len: usize,
    },

    /// A transaction.
    ///
    /// Can be a transaction, a auto posting rule, a balance assertion, etc.
    ///
    /// The syntax is:
    ///
    /// ```text
    /// <title>  ; comment
    ///     <entry-name>  <entry-value>  ; comment
    /// ```
    Transaction {
        /// Transaction title
        title: String,
        /// Transaction title comment
        title_comment: Option<SingleLineComment>,
        /// Transaction entries
        entries: Vec<TransactionNode>,
        /// Indent of the first transaction entry
        first_entry_indent: usize,
        /// Maximum length of the entry names
        max_entry_name_len: usize,
        /// Maximum length of the entry values
        max_entry_value_len: usize,
        /// Maximum length of units before decimal mark in entries
        max_entry_units_len: usize,
        /// Maximum length of decimal mark in entries
        max_entry_decimal_len: usize,
        /// Maximum length of part after decimal mark in entries
        max_entry_after_decimal_len: usize,
    },
}

/// Decimal mark
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum DecimalMark {
    /// '.'
    Period = b'.',
    /// ','
    Comma = b',',
}

/// Prefix of a single line comment
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum CommentPrefix {
    /// '#'
    Hash = b'#',
    /// ';'
    Semicolon = b';',
}

/// A single line comment (starting with '#' or ';')
#[derive(Debug, Clone, PartialEq)]
pub struct SingleLineComment {
    /// The comment content
    pub content: String,
    /// The comment prefix ('#' or ';')
    pub prefix: CommentPrefix,
    /// Line number in the file (1-indexed)
    lineno: usize,
    /// Column number in the file (1-indexed)
    pub colno: usize,
}

/// A directive
#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    /// The directive name
    pub name: String,
    /// The directive content
    pub content: String,
    /// Comment associated with the directive
    pub comment: Option<SingleLineComment>,
}

/// A directive or a single line comment
#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveNode {
    Directive(Directive),
    SingleLineComment(SingleLineComment),
}

/// A transaction entry
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionEntry {
    /// Entry name
    pub name: String,
    /// Entry value
    pub value: String,
    /// Length of the units before the decimal mark
    pub value_units_len: usize,
    /// Length of the decimal part of the value
    pub value_decimal_len: usize,
    /// Comment associated with the entry
    pub comment: Option<SingleLineComment>,
}

/// A transaction entry or a single line comment
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionNode {
    TransactionEntry(TransactionEntry),
    SingleLineComment(SingleLineComment),
}

/// Current state of the parser
#[derive(Debug, PartialEq)]
enum ParserState {
    /// Start state
    Start,
    /// Inside a multiline comment
    MultilineComment,
}

/// Temporary data used by the parser
struct ParserTempData {
    /// Line number where the current multiline comment started
    multiline_comment_start_lineno: usize,
    /// Content of the current multiline comment
    multiline_comment_content: String,
    /// Directives group content
    directives_group_content: Vec<DirectiveNode>,
    /// Maximum length of the directive names + contents
    directives_group_max_name_content_len: usize,
    /// Transaction title
    transaction_title: String,
    /// Transaction title comment
    transaction_title_comment: Option<SingleLineComment>,
    /// Transaction entries
    transaction_entries: Vec<TransactionNode>,
    /// If the current transaction has entries (ignoring comments)
    transaction_has_no_comment_entries: bool,
    /// Indent of the first transaction entry
    first_entry_indent: usize,
    /// Maximum length of the entry names
    max_entry_name_len: usize,
    /// Maximum length of the entry values
    max_entry_value_len: usize,
    /// Maximum length of units before decimal mark in entries
    max_entry_units_len: usize,
    /// Maximum length of decimal mark in entries
    max_entry_decimal_len: usize,
    /// Maximum length of part after decimal mark in entries
    max_entry_after_decimal_len: usize,
}

impl ParserTempData {
    fn new() -> Self {
        Self {
            multiline_comment_start_lineno: 0,
            multiline_comment_content: String::new(),
            directives_group_content: Vec::new(),
            directives_group_max_name_content_len: 0,
            transaction_title: String::new(),
            transaction_title_comment: None,
            transaction_entries: Vec::new(),
            transaction_has_no_comment_entries: false,
            first_entry_indent: 0,
            max_entry_name_len: 0,
            max_entry_value_len: 0,
            max_entry_units_len: 0,
            max_entry_decimal_len: 0,
            max_entry_after_decimal_len: 0,
        }
    }
}

pub fn parse_content(content: &str) -> Result<JournalFile, errors::SyntaxError> {
    let mut state = ParserState::Start;
    let mut data = ParserTempData::new();
    let mut journal = Vec::new();

    for (lineno, line) in content.lines().enumerate() {
        let mut chars_iter = line.chars().enumerate();

        if line.is_empty() {
            process_empty_line(lineno + 1, &mut journal, &mut data);
            continue;
        }

        match state {
            ParserState::Start => {
                if let Some((colno, c)) = chars_iter.next() {
                    if c == '#' || c == ';' {
                        let prefix = if c == '#' {
                            CommentPrefix::Hash
                        } else {
                            CommentPrefix::Semicolon
                        };

                        let content = line[colno + 1..].to_string();
                        let comment = SingleLineComment {
                            content,
                            prefix,
                            lineno: lineno + 1,
                            colno: colno + 1,
                        };

                        if data.directives_group_content.is_empty()
                            && data.transaction_title.is_empty()
                        {
                            journal.push(JournalCstNode::SingleLineComment(comment));
                        } else if !data.transaction_title.is_empty() {
                            data.transaction_entries.push(
                                TransactionNode::SingleLineComment(comment),
                            );
                        } else {
                            data.directives_group_content
                                .push(DirectiveNode::SingleLineComment(comment));
                        }
                        state = ParserState::Start;
                    } else if colno == 0 && line == "comment" {
                        state = ParserState::MultilineComment;
                        data.multiline_comment_start_lineno = lineno + 1;
                        data.multiline_comment_content = String::new();
                    } else if colno == 0 && line.chars().map(char::is_whitespace).all(|x| x) {
                        process_empty_line(lineno + 1, &mut journal, &mut data);
                    } else if colno == 0
                        && (line.starts_with("account ")
                            || line.starts_with("commodity ")
                            || line.starts_with("decimal-mark ")
                            || line.starts_with("payee ")
                            || line.starts_with("tag ")
                            || line.starts_with("include ")
                            || line.starts_with("P ")
                            || line.starts_with("apply account ")
                            || line.starts_with("D ")
                            || line.starts_with("Y ")
                            // other Ledger directives
                            || line.starts_with("apply fixed ")
                            || line.starts_with("apply tag ")
                            || line.starts_with("assert ")
                            || line.starts_with("capture ")
                            || line.starts_with("check ")
                            || line.starts_with("define ")
                            || line.starts_with("bucket / A ")
                            || line.starts_with("end apply fixed")
                            || line.starts_with("end apply tag")
                            || line.starts_with("end apply year")
                            || line.starts_with("end tag")
                            || line.starts_with("eval")
                            || line.starts_with("expr")
                            || line.starts_with("python")  // 'python' CODE not supported
                            || line.starts_with("tag ")
                            || line.starts_with("value ")
                            || line.starts_with("--command-line-flags")
                        )
                    {
                        parse_directive(
                            line.split_whitespace().next().unwrap(),
                            &mut chars_iter,
                            lineno,
                            &mut data,
                        );
                    } else if colno == 0 && c.is_whitespace() {
                        if data.transaction_title.is_empty() {
                            // probably single line comment that starts with a space
                            let mut content = String::new();

                            let mut comment_prefix = None;
                            let mut colno = 0;
                            while let Some((coln, c)) = chars_iter.next() {
                                if comment_prefix.is_none() {
                                    if c == '#' {
                                        comment_prefix = Some(CommentPrefix::Hash);
                                        colno = coln + 1;
                                    } else if c == ';' {
                                        comment_prefix = Some(CommentPrefix::Semicolon);
                                        colno = coln + 1;
                                    } else if !c.is_whitespace() {
                                        return Err(SyntaxError {
                                            message: format!("Unexpected character {c:?}"),
                                            lineno: lineno + 1,
                                            colno_start: coln + 1,
                                            colno_end: coln + 2,
                                            expected: "'#', ';' or newline",
                                        });
                                    }
                                } else {
                                    content.push(c);
                                }
                            }

                            if let Some(prefix) = comment_prefix {
                                let comment = SingleLineComment {
                                    content,
                                    prefix,
                                    lineno: lineno + 1,
                                    colno,
                                };
                                if data.directives_group_content.is_empty() {
                                    journal.push(JournalCstNode::SingleLineComment(comment));
                                } else if !data.transaction_title.is_empty() {
                                    data.transaction_entries.push(
                                        TransactionNode::SingleLineComment(
                                            comment,
                                        ),
                                    );
                                } else {
                                    data.directives_group_content.push(
                                        DirectiveNode::SingleLineComment(comment),
                                    );
                                }
                            }
                        } else {
                            // inside transaction entry
                            let mut at_indent = c != '\t';
                            let mut indent = if at_indent { 1 } else { 4 };
                            let mut entry_name = String::new();
                            let mut prev_was_whitespace = c.is_whitespace();
                            let mut is_comment_only = false;
                            while let Some((coln, c)) = chars_iter.next() {
                                if at_indent {
                                    if c == '\t' {
                                        indent += 4;
                                    } else if c.is_whitespace() {
                                        indent += 1;
                                    } else if c == ';' || c == '#' {
                                        // transaction entry with empty value
                                        let comment = parse_inline_comment(
                                            &mut chars_iter,
                                            lineno,
                                            coln + 1,
                                            Some(if c == '#' {
                                                CommentPrefix::Hash
                                            } else {
                                                CommentPrefix::Semicolon
                                            }),
                                        );
                                        if comment.is_some() {
                                            is_comment_only = true;
                                            // if the first comment is indented with >=2 and first entry indent
                                            // is not setted, set it
                                            //
                                            // this is needed for transactions without entries, only comments
                                            if indent >= 2 && data.first_entry_indent == 0 {
                                                data.first_entry_indent = indent;
                                            }
                                            data.transaction_entries.push(
                                                TransactionNode::SingleLineComment(
                                                    comment.unwrap(),
                                                ),
                                            );
                                        }
                                        break;
                                    } else {
                                        at_indent = false;
                                        entry_name.push(c);
                                    }
                                } else {
                                    if c == '\t' {
                                        break;
                                    } else if c.is_whitespace() {
                                        if prev_was_whitespace {
                                            entry_name.pop(); // remove previous whitespace
                                            break;
                                        }
                                        prev_was_whitespace = true;
                                    } else {
                                        prev_was_whitespace = false;
                                    }
                                    entry_name.push(c);
                                }
                            }

                            if is_comment_only {
                                continue;
                            }

                            if data.first_entry_indent == 0 {
                                data.first_entry_indent = indent;
                            } else if !data.transaction_has_no_comment_entries {
                                // if the first entry is a comment, the indent is not
                                // properly setted so we need to set it here
                                data.first_entry_indent = indent;
                            }
                            data.max_entry_name_len = data.max_entry_name_len.max(entry_name.len());

                            let mut entry_value = String::new();
                            let mut inside_entry_value = false;
                            let mut comment = None;

                            while let Some((coln, c)) = chars_iter.next() {
                                if !inside_entry_value {
                                    if c == ';' || c == '#' {
                                        // transaction entry with empty value
                                        comment = parse_inline_comment(
                                            &mut chars_iter,
                                            lineno,
                                            coln + 1,
                                            Some(if c == '#' {
                                                CommentPrefix::Hash
                                            } else {
                                                CommentPrefix::Semicolon
                                            }),
                                        );
                                        break;
                                    } else if !c.is_whitespace() {
                                        inside_entry_value = true;
                                        entry_value.push(c);
                                        continue;
                                    }
                                } else if c == ';' || c == '#' {
                                    comment = parse_inline_comment(
                                        &mut chars_iter,
                                        lineno,
                                        coln + 1,
                                        Some(if c == '#' {
                                            CommentPrefix::Hash
                                        } else {
                                            CommentPrefix::Semicolon
                                        }),
                                    );
                                    break;
                                } else {
                                    entry_value.push(c);
                                }
                            }

                            entry_value = entry_value.trim_end().to_string();

                            data.max_entry_value_len =
                                data.max_entry_value_len.max(entry_value.len());

                            // first value number
                            let first_value_num = if entry_value.contains(' ') {
                                entry_value.split_once(' ').unwrap().0.to_string()
                            } else {
                                entry_value.clone()
                            };

                            let value_units_len =
                                if first_value_num.contains(',') || first_value_num.contains('.') {
                                    first_value_num.len()
                                        - first_value_num
                                            .split(|c| c == ',' || c == '.')
                                            .last()
                                            .unwrap()
                                            .len()
                                } else {
                                    first_value_num.len()
                                };

                            data.max_entry_units_len =
                                data.max_entry_units_len.max(value_units_len);
                            let value_decimal_len = first_value_num.len() - value_units_len;
                            data.max_entry_decimal_len =
                                data.max_entry_decimal_len.max(value_decimal_len);
                            let value_after_decimal_len =
                                entry_value.len() - value_units_len - value_decimal_len;
                            data.max_entry_after_decimal_len = data
                                .max_entry_after_decimal_len
                                .max(value_after_decimal_len);

                            data.transaction_has_no_comment_entries = true;
                            data.transaction_entries.push(
                                TransactionNode::TransactionEntry(
                                    TransactionEntry {
                                        name: entry_name,
                                        value: entry_value,
                                        value_units_len,
                                        value_decimal_len,
                                        comment,
                                    },
                                ),
                            );
                        }
                    } else if colno == 0 {
                        // starts transaction

                        // if we are in a current transaction, save it adding a newline
                        if !data.transaction_title.is_empty() {
                            process_empty_line(lineno, &mut journal, &mut data);
                        }

                        let mut transaction_title = String::new();
                        transaction_title.push(c);
                        let mut prev_was_whitespace = false;
                        let mut is_periodic = false;
                        while let Some((_, c)) = chars_iter.next() {
                            if c.is_whitespace() {
                                if prev_was_whitespace {
                                    // periodic transactions (starts with "~ ") allow two
                                    // spaces between the period and the title
                                    if transaction_title.starts_with("~ ") && !is_periodic {
                                        is_periodic = true;
                                    } else {
                                        transaction_title.pop(); // remove previous whitespace
                                        break;
                                    }
                                }
                                prev_was_whitespace = true;
                            } else {
                                prev_was_whitespace = false;
                            }
                            transaction_title.push(c);
                        }

                        data.transaction_title = transaction_title;

                        if !prev_was_whitespace {
                            // end of line
                            continue;
                        }

                        data.transaction_title_comment =
                            parse_inline_comment(&mut chars_iter, lineno, 1, None);
                    }
                }
            }
            ParserState::MultilineComment => {
                if line == "end comment" {
                    save_multiline_comment(&mut data, &mut journal, lineno + 1);
                    state = ParserState::Start;
                } else {
                    data.multiline_comment_content.push_str(line);
                    data.multiline_comment_content.push('\n');
                }
            }
        }
    }

    // Hledger v1.40 traits not ended multiline comments as a multiline comment
    if state == ParserState::MultilineComment {
        save_multiline_comment(&mut data, &mut journal, content.lines().count());
    }

    if !data.directives_group_content.is_empty() {
        save_directives_group_content(&mut data, &mut journal);
    } else if !data.transaction_title.is_empty() {
        save_transaction(&mut data, &mut journal);
    }

    Ok(journal)
}

fn process_empty_line(lineno: usize, journal: &mut Vec<JournalCstNode>, data: &mut ParserTempData) {
    if !data.directives_group_content.is_empty() {
        save_directives_group_content(data, journal);
    } else if !data.transaction_title.is_empty() {
        save_transaction(data, journal);
    }
    journal.push(JournalCstNode::EmptyLine { lineno });
}

fn parse_directive(
    name: &str,
    chars_iter: &mut impl Iterator<Item = (usize, char)>,
    lineno: usize,
    data: &mut ParserTempData,
) {
    let mut content = String::new();
    let mut prev_was_whitespace = false;
    let mut last_colno = 0;
    for _ in 0..name.len() {
        chars_iter.next();
    }
    let mut comment_colno_padding = 1;
    while let Some((colno, c)) = chars_iter.next() {
        if c == '\t' {
            last_colno = colno;
            comment_colno_padding = 4;
            break;
        }
        if c.is_whitespace() {
            if prev_was_whitespace {
                // double whitespace, end of content
                last_colno = colno;
                content.pop(); // remove previous whitespace
                break;
            }
            prev_was_whitespace = true;
        } else {
            prev_was_whitespace = false;
        }
        content.push(c);
    }
    let mut comment = None;
    if last_colno != 0 {
        // not end of line
        comment = parse_inline_comment(chars_iter, lineno, comment_colno_padding, None);
    }

    let content_len = content.len();
    data.directives_group_content
        .push(DirectiveNode::Directive(Directive {
            name: name.to_string(),
            content,
            comment,
        }));
    data.directives_group_max_name_content_len = data
        .directives_group_max_name_content_len
        .max(content_len + name.len());
}

fn parse_inline_comment(
    chars_iter: &mut impl Iterator<Item = (usize, char)>,
    lineno: usize,
    colno_padding: usize,
    from_comment_prefix: Option<CommentPrefix>,
) -> Option<SingleLineComment> {
    let mut comment_prefix = from_comment_prefix;
    let mut comment_content = String::new();
    let mut first_colno = colno_padding;
    while let Some((colno, c)) = chars_iter.next() {
        if comment_prefix.is_none() {
            if c == '#' {
                comment_prefix = Some(CommentPrefix::Hash);
                first_colno = colno + first_colno;
            } else if c == ';' {
                comment_prefix = Some(CommentPrefix::Semicolon);
                first_colno = colno + first_colno;
            } else if c == '\t' {
                first_colno += 3;
            } else {
                continue;
            }
        } else {
            comment_content.push(c);
        }
    }
    if let Some(prefix) = comment_prefix {
        Some(SingleLineComment {
            content: comment_content,
            prefix,
            lineno: lineno + 1,
            colno: first_colno,
        })
    } else {
        None
    }
}

fn save_multiline_comment(
    data: &mut ParserTempData,
    journal: &mut Vec<JournalCstNode>,
    lineno: usize,
) {
    journal.push(JournalCstNode::MultilineComment {
        content: data.multiline_comment_content.clone(),
        lineno_start: data.multiline_comment_start_lineno,
        lineno_end: lineno,
    });
    data.multiline_comment_content.clear();
    data.multiline_comment_start_lineno = 0;
}

fn save_directives_group_content(data: &mut ParserTempData, journal: &mut Vec<JournalCstNode>) {
    journal.push(JournalCstNode::DirectivesGroup {
        content: data.directives_group_content.clone(),
        max_name_content_len: data.directives_group_max_name_content_len,
    });
    data.directives_group_content.clear();
    data.directives_group_max_name_content_len = 0;
}

fn save_transaction(data: &mut ParserTempData, journal: &mut Vec<JournalCstNode>) {
    journal.push(JournalCstNode::Transaction {
        title: data.transaction_title.clone(),
        title_comment: data.transaction_title_comment.clone(),
        entries: data.transaction_entries.clone(),
        first_entry_indent: data.first_entry_indent,
        max_entry_name_len: data.max_entry_name_len,
        max_entry_value_len: data.max_entry_value_len,
        max_entry_units_len: data.max_entry_units_len,
        max_entry_decimal_len: data.max_entry_decimal_len,
        max_entry_after_decimal_len: data.max_entry_after_decimal_len,
    });
    data.transaction_title.clear();
    data.transaction_title_comment = None;
    data.transaction_entries.clear();
    data.transaction_has_no_comment_entries = false;
    data.first_entry_indent = 0;
    data.max_entry_name_len = 0;
    data.max_entry_value_len = 0;
    data.max_entry_units_len = 0;
    data.max_entry_decimal_len = 0;
    data.max_entry_after_decimal_len = 0;
}
