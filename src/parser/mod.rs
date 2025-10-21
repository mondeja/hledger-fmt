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
        nodes: Vec<DirectiveNode>,
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

        max_entry_value_first_part_before_decimals_len: usize,
        max_entry_value_first_part_after_decimals_len: usize,
        max_entry_value_first_separator_len: usize,
        max_entry_value_second_part_before_decimals_len: usize,
        max_entry_value_second_part_after_decimals_len: usize,
        max_entry_value_second_separator_len: usize,
        max_entry_value_third_part_before_decimals_len: usize,
        max_entry_value_third_part_after_decimals_len: usize,
    },
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
    Subdirective(String), // includes comments after the subdirective content
    SingleLineComment(SingleLineComment),
}

/// A transaction entry
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionEntry {
    /// Entry name
    pub name: String,
    pub value_first_part_before_decimals: String,
    pub value_first_part_after_decimals: String,
    pub value_first_separator: String,
    pub value_second_part_before_decimals: String,
    pub value_second_part_after_decimals: String,
    pub value_second_separator: String,
    pub value_third_part_before_decimals: String,
    pub value_third_part_after_decimals: String,
    /// Comment associated with the entry
    pub comment: Option<SingleLineComment>,
}

/// A transaction entry or a single line comment
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionNode {
    TransactionEntry(Box<TransactionEntry>),
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
    /// Directives group nodes
    directives_group_nodes: Vec<DirectiveNode>,
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
    max_entry_value_first_part_before_decimals_len: usize,
    max_entry_value_first_part_after_decimals_len: usize,
    max_entry_value_first_separator_len: usize,
    max_entry_value_second_part_before_decimals_len: usize,
    max_entry_value_second_part_after_decimals_len: usize,
    max_entry_value_second_separator_len: usize,
    max_entry_value_third_part_before_decimals_len: usize,
    max_entry_value_third_part_after_decimals_len: usize,
}

impl ParserTempData {
    fn new() -> Self {
        Self {
            multiline_comment_start_lineno: 0,
            multiline_comment_content: String::new(),
            directives_group_nodes: Vec::new(),
            directives_group_max_name_content_len: 0,
            transaction_title: String::new(),
            transaction_title_comment: None,
            transaction_entries: Vec::new(),
            transaction_has_no_comment_entries: false,
            first_entry_indent: 0,
            max_entry_name_len: 0,
            max_entry_value_first_part_before_decimals_len: 0,
            max_entry_value_first_part_after_decimals_len: 0,
            max_entry_value_first_separator_len: 0,
            max_entry_value_second_part_before_decimals_len: 0,
            max_entry_value_second_part_after_decimals_len: 0,
            max_entry_value_second_separator_len: 0,
            max_entry_value_third_part_before_decimals_len: 0,
            max_entry_value_third_part_after_decimals_len: 0,
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

                        if data.directives_group_nodes.is_empty()
                            && data.transaction_title.is_empty()
                        {
                            journal.push(JournalCstNode::SingleLineComment(comment));
                        } else if !data.transaction_title.is_empty() {
                            data.transaction_entries
                                .push(TransactionNode::SingleLineComment(comment));
                        } else {
                            data.directives_group_nodes
                                .push(DirectiveNode::SingleLineComment(comment));
                        }
                    } else if colno == 0 && line == "comment" {
                        state = ParserState::MultilineComment;
                        data.multiline_comment_start_lineno = lineno + 1;
                        data.multiline_comment_content = String::with_capacity(128);
                    } else if colno == 0 && line.chars().all(char::is_whitespace) {
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
                            || line.starts_with("--command-line-flags"))
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
                            let mut content = String::with_capacity(128);
                            let mut is_subdirective = false;

                            let mut comment_prefix = None;
                            let mut colno = 0;
                            for (coln, c) in chars_iter.by_ref() {
                                if comment_prefix.is_some() {
                                    content.push(c);
                                    continue;
                                }

                                if is_subdirective {
                                    content.push(c);
                                    continue;
                                }

                                if c == '#' {
                                    comment_prefix = Some(CommentPrefix::Hash);
                                    colno = coln + 1;
                                } else if c == ';' {
                                    comment_prefix = Some(CommentPrefix::Semicolon);
                                    colno = coln + 1;
                                } else if !c.is_whitespace() {
                                    // if we're inside a directives group, is a subdirective
                                    if !data.directives_group_nodes.is_empty() {
                                        is_subdirective = true;
                                        content.push(c);
                                        continue;
                                    }

                                    return Err(SyntaxError {
                                        message: format!("Unexpected character {c:?}"),
                                        lineno: lineno + 1,
                                        colno_start: coln + 1,
                                        colno_end: coln + 2,
                                        expected: "'#', ';' or newline",
                                    });
                                }
                            }

                            if let Some(prefix) = comment_prefix {
                                let comment = SingleLineComment {
                                    content,
                                    prefix,
                                    lineno: lineno + 1,
                                    colno,
                                };
                                if data.directives_group_nodes.is_empty() {
                                    journal.push(JournalCstNode::SingleLineComment(comment));
                                } else if !data.transaction_title.is_empty() {
                                    data.transaction_entries
                                        .push(TransactionNode::SingleLineComment(comment));
                                } else {
                                    data.directives_group_nodes
                                        .push(DirectiveNode::SingleLineComment(comment));
                                }
                            } else if is_subdirective {
                                data.directives_group_nodes
                                    .push(DirectiveNode::Subdirective(content));
                            }
                        } else {
                            // maybe inside transaction entry
                            let mut at_indent = c != '\t';
                            let mut indent = if at_indent { 1 } else { 4 };
                            let mut entry_name = String::with_capacity(64);
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
                                        let maybe_comment = parse_inline_comment(
                                            &mut chars_iter,
                                            lineno,
                                            coln + 1,
                                            Some(if c == '#' {
                                                CommentPrefix::Hash
                                            } else {
                                                CommentPrefix::Semicolon
                                            }),
                                        );
                                        if let Some(comment) = maybe_comment {
                                            is_comment_only = true;
                                            // if the first comment is indented with >=2 and first entry indent
                                            // is not setted, set it
                                            //
                                            // this is needed for transactions without entries, only comments
                                            if indent >= 2 && data.first_entry_indent == 0 {
                                                data.first_entry_indent = indent;
                                            }
                                            data.transaction_entries
                                                .push(TransactionNode::SingleLineComment(comment));
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

                                        if c == ';' && entry_name.is_empty() {
                                            // inside comment
                                            let maybe_comment = parse_inline_comment(
                                                &mut chars_iter,
                                                lineno,
                                                coln + 1,
                                                Some(CommentPrefix::Semicolon),
                                            );
                                            if let Some(comment) = maybe_comment {
                                                is_comment_only = true;
                                                data.transaction_entries.push(
                                                    TransactionNode::SingleLineComment(comment),
                                                );
                                            }
                                            break;
                                        }
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
                            data.max_entry_name_len =
                                data.max_entry_name_len.max(entry_name.chars().count());

                            let mut entry_value = String::with_capacity(64);
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

                            // let coln = entry_value.chars().count()
                            //     + entry_name.chars().count()
                            //     + indent + 1;
                            entry_value = entry_value.trim_end().to_string();

                            let mut p = EntryValueParser::default();
                            // for development: to raise errors, pass lineno and coln
                            p.parse(&entry_value)?; // , lineno + 1, coln)?;

                            data.max_entry_value_first_part_before_decimals_len = data
                                .max_entry_value_first_part_before_decimals_len
                                .max(p.first_part_before_decimals.chars().count());
                            data.max_entry_value_first_part_after_decimals_len = data
                                .max_entry_value_first_part_after_decimals_len
                                .max(p.first_part_after_decimals.chars().count());

                            data.max_entry_value_first_separator_len = data
                                .max_entry_value_first_separator_len
                                .max(p.first_separator.len());

                            data.max_entry_value_second_part_before_decimals_len = data
                                .max_entry_value_second_part_before_decimals_len
                                .max(p.second_part_before_decimals.chars().count());
                            data.max_entry_value_second_part_after_decimals_len = data
                                .max_entry_value_second_part_after_decimals_len
                                .max(p.second_part_after_decimals.chars().count());

                            data.max_entry_value_second_separator_len = data
                                .max_entry_value_second_separator_len
                                .max(p.second_separator.len());

                            data.max_entry_value_third_part_before_decimals_len = data
                                .max_entry_value_third_part_before_decimals_len
                                .max(p.third_part_before_decimals.chars().count());
                            data.max_entry_value_third_part_after_decimals_len = data
                                .max_entry_value_third_part_after_decimals_len
                                .max(p.third_part_after_decimals.chars().count());

                            data.transaction_has_no_comment_entries = true;
                            data.transaction_entries
                                .push(TransactionNode::TransactionEntry(Box::new(
                                    TransactionEntry {
                                        name: entry_name,
                                        value_first_part_before_decimals: p
                                            .first_part_before_decimals,
                                        value_first_part_after_decimals: p
                                            .first_part_after_decimals,
                                        value_first_separator: p.first_separator,
                                        value_second_part_before_decimals: p
                                            .second_part_before_decimals,
                                        value_second_part_after_decimals: p
                                            .second_part_after_decimals,
                                        value_second_separator: p.second_separator,
                                        value_third_part_before_decimals: p
                                            .third_part_before_decimals,
                                        value_third_part_after_decimals: p
                                            .third_part_after_decimals,
                                        comment,
                                    },
                                )));
                        }
                    } else if colno == 0 {
                        // starts transaction

                        // if we are in a current transaction, save it adding a newline
                        if !data.transaction_title.is_empty() {
                            process_empty_line(lineno, &mut journal, &mut data);
                        }

                        let mut transaction_title = String::with_capacity(64);
                        transaction_title.push(c);
                        let mut comment_prefix = None;
                        for (_, c) in chars_iter.by_ref() {
                            if c == ';' {
                                comment_prefix = Some(CommentPrefix::Semicolon);
                                break;
                            } else if c == '#' {
                                comment_prefix = Some(CommentPrefix::Hash);
                                break;
                            }
                            transaction_title.push(c);
                        }

                        data.transaction_title = transaction_title.trim_end().to_string();

                        data.transaction_title_comment =
                            parse_inline_comment(&mut chars_iter, lineno, 1, comment_prefix);
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

    if !data.directives_group_nodes.is_empty() {
        save_directives_group_nodes(&mut data, &mut journal);
    } else if !data.transaction_title.is_empty() {
        save_transaction(&mut data, &mut journal);
    }

    Ok(journal)
}

fn process_empty_line(lineno: usize, journal: &mut Vec<JournalCstNode>, data: &mut ParserTempData) {
    if !data.directives_group_nodes.is_empty() {
        save_directives_group_nodes(data, journal);
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
    let name_chars_count = name.chars().count();
    let mut content = String::with_capacity(name_chars_count);
    let mut prev_was_whitespace = false;
    let mut last_colno = 0;
    for _ in 0..name.chars().count() {
        chars_iter.next();
    }
    let mut comment_colno_padding = 1;
    for (colno, c) in chars_iter.by_ref() {
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

    let content_len = content.chars().count();
    data.directives_group_nodes
        .push(DirectiveNode::Directive(Directive {
            name: name.to_string(),
            content,
            comment,
        }));
    data.directives_group_max_name_content_len = data
        .directives_group_max_name_content_len
        .max(content_len + name_chars_count);
}

fn parse_inline_comment(
    chars_iter: &mut impl Iterator<Item = (usize, char)>,
    lineno: usize,
    colno_padding: usize,
    from_comment_prefix: Option<CommentPrefix>,
) -> Option<SingleLineComment> {
    let mut comment_prefix = from_comment_prefix;
    let mut comment_content = String::with_capacity(128);
    let mut first_colno = colno_padding;
    for (colno, c) in chars_iter.by_ref() {
        if comment_prefix.is_none() {
            if c == '#' {
                comment_prefix = Some(CommentPrefix::Hash);
                first_colno += colno;
            } else if c == ';' {
                comment_prefix = Some(CommentPrefix::Semicolon);
                first_colno += colno;
            } else if c == '\t' {
                first_colno += 3;
            } else {
                continue;
            }
        } else {
            comment_content.push(c);
        }
    }
    comment_prefix.map(|prefix| SingleLineComment {
        content: comment_content,
        prefix,
        lineno: lineno + 1,
        colno: first_colno,
    })
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

fn save_directives_group_nodes(data: &mut ParserTempData, journal: &mut Vec<JournalCstNode>) {
    journal.push(JournalCstNode::DirectivesGroup {
        nodes: data.directives_group_nodes.clone(),
        max_name_content_len: data.directives_group_max_name_content_len,
    });
    data.directives_group_nodes.clear();
    data.directives_group_max_name_content_len = 0;
}

fn save_transaction(data: &mut ParserTempData, journal: &mut Vec<JournalCstNode>) {
    journal.push(JournalCstNode::Transaction {
        title: data.transaction_title.clone(),
        title_comment: data.transaction_title_comment.clone(),
        entries: data.transaction_entries.clone(),
        first_entry_indent: data.first_entry_indent,
        max_entry_name_len: data.max_entry_name_len,
        max_entry_value_first_part_before_decimals_len: data
            .max_entry_value_first_part_before_decimals_len,
        max_entry_value_first_part_after_decimals_len: data
            .max_entry_value_first_part_after_decimals_len,
        max_entry_value_first_separator_len: data.max_entry_value_first_separator_len,
        max_entry_value_second_part_before_decimals_len: data
            .max_entry_value_second_part_before_decimals_len,
        max_entry_value_second_part_after_decimals_len: data
            .max_entry_value_second_part_after_decimals_len,
        max_entry_value_second_separator_len: data.max_entry_value_second_separator_len,
        max_entry_value_third_part_before_decimals_len: data
            .max_entry_value_third_part_before_decimals_len,
        max_entry_value_third_part_after_decimals_len: data
            .max_entry_value_third_part_after_decimals_len,
    });
    data.transaction_title.clear();
    data.transaction_title_comment = None;
    data.transaction_entries.clear();
    data.transaction_has_no_comment_entries = false;
    data.first_entry_indent = 0;
    data.max_entry_name_len = 0;
    data.max_entry_value_first_part_before_decimals_len = 0;
    data.max_entry_value_first_part_after_decimals_len = 0;
    data.max_entry_value_first_separator_len = 0;
    data.max_entry_value_second_part_before_decimals_len = 0;
    data.max_entry_value_second_part_after_decimals_len = 0;
    data.max_entry_value_second_separator_len = 0;
    data.max_entry_value_third_part_before_decimals_len = 0;
    data.max_entry_value_third_part_after_decimals_len = 0;
}

/// Entry value parser
///
/// This parser is used to parse the value of a transaction entry.
///
/// A value can consist of one of the following:
///
/// - `N`  (amount)
/// - `N @ N`         (price per unit cost)
/// - `N @@ N`        (total price cost)
/// - `N sep N`       (balance assertion)
/// - `  = N`         (balance assignment)
/// - `N @ N sep N`   (price per unit cost and balance assertion)
/// - `N @@ N sep N`  (total price cost and balance assertion)
/// - `N sep N @ N`   (balance assertion and price per unit cost)
/// - `N sep N @@ N`  (balance assertion and total price cost)
///
/// Where:
///
/// - `N` is a number and optional commodity.
/// - `sep` is either `=`, `==` or `==*`.
/// - Rest of characters are literals.
///
/// In order to format the transaction entries, we must extract each part of the value
/// with their size in unit and decimal parts.
#[derive(Default, Debug)]
pub(crate) struct EntryValueParser {
    first_part_before_decimals: String,
    first_part_after_decimals: String,
    first_separator: String,
    second_part_before_decimals: String,
    second_part_after_decimals: String,
    second_separator: String,
    third_part_before_decimals: String,
    third_part_after_decimals: String,
}

#[derive(Debug)]
enum EntryValueParserState {
    FirstPartCommodityBefore,
    FirstPartNumber,
    FirstPartCommodityAfter,
    FirstSeparator,
    SecondPartCommodityBefore,
    SecondPartNumber,
    SecondPartCommodityAfter,
    SecondSeparator,
    ThirdPartCommodityBefore,
    ThirdPartNumber,
    ThirdPartCommodityAfter,
}

impl EntryValueParser {
    pub(crate) fn parse(&mut self, value: &str) -> Result<(), SyntaxError> {
        let chars = value.chars();
        let value_length = value.len();

        use EntryValueParserState::*;
        let mut state = FirstPartCommodityBefore;

        let mut current_spaces_in_a_row = 0;
        let mut current_commodity_is_quoted = false;
        let mut first_part_value = String::with_capacity(value_length);
        let mut second_part_value = String::with_capacity(value_length);
        let mut third_part_value = String::with_capacity(value_length);

        for c in chars {
            //println!("state: {:?}, c: {:?}", state, c);
            match state {
                FirstPartCommodityBefore => {
                    if c.is_whitespace() {
                        current_spaces_in_a_row += 1;
                        if current_commodity_is_quoted {
                            first_part_value.push(c);
                        } else if current_spaces_in_a_row > 1 {
                            // no commodity
                            state = FirstSeparator;
                            current_spaces_in_a_row = 0;
                            current_commodity_is_quoted = false;
                        } else {
                            // first space
                            first_part_value.push(c);
                        }
                    } else if c.is_ascii_digit() || c == '.' || c == ',' {
                        first_part_value.push(c);
                        state = FirstPartNumber;
                        current_spaces_in_a_row = 0;
                        current_commodity_is_quoted = false;
                    } else if c == '"' {
                        first_part_value.push(c);
                        if current_commodity_is_quoted {
                            state = FirstPartNumber;
                        }
                        current_commodity_is_quoted = true;
                    } else {
                        first_part_value.push(c);
                    }
                }
                FirstPartNumber => {
                    if c.is_ascii_digit() || c == '.' || c == ',' {
                        first_part_value.push(c);
                    } else if c == ' ' {
                        if !first_part_value.is_empty() {
                            first_part_value.push(c);
                            state = FirstPartCommodityAfter;
                        }
                    } else if c == '@' || c == '=' || c == '*' {
                        self.first_separator.push(c);
                        state = FirstSeparator;
                    } else if c == '"' {
                        first_part_value.push(c);
                        state = FirstPartCommodityAfter;
                    } else {
                        if c == '"' {
                            current_commodity_is_quoted = true;
                        }
                        first_part_value.push(c);
                        state = FirstPartCommodityAfter;
                        current_spaces_in_a_row = 0;
                    }
                }
                FirstPartCommodityAfter => {
                    if current_commodity_is_quoted {
                        if c == '"' {
                            first_part_value.push(c);
                            state = FirstSeparator;
                            current_commodity_is_quoted = false;
                        } else {
                            first_part_value.push(c);
                        }
                    } else if c.is_whitespace() {
                        state = FirstSeparator;
                        current_commodity_is_quoted = false;
                    } else if c == '@' || c == '*' || c == '=' {
                        self.first_separator.push(c);
                        state = FirstSeparator;
                        current_commodity_is_quoted = false;
                    } else if c == '"' {
                        first_part_value.push(c);
                        current_commodity_is_quoted = true;
                    } else {
                        // really numbers are forbidden by hledger, but don't care
                        first_part_value.push(c);
                    }
                }
                FirstSeparator => {
                    if c == '@' || c == '*' || c == '=' {
                        self.first_separator.push(c);
                    } else if !c.is_whitespace() {
                        second_part_value.push(c);
                        state = SecondPartCommodityBefore;
                        current_spaces_in_a_row = 0;
                    }
                }
                SecondPartCommodityBefore => {
                    if c.is_whitespace() {
                        current_spaces_in_a_row += 1;
                        if current_commodity_is_quoted {
                            second_part_value.push(c);
                        } else if current_spaces_in_a_row > 1 {
                            // no commodity
                            state = SecondSeparator;
                            current_spaces_in_a_row = 0;
                            current_commodity_is_quoted = false;
                        } else {
                            // first space
                            second_part_value.push(c);
                        }
                    } else if c.is_ascii_digit() || c == '.' || c == ',' {
                        second_part_value.push(c);
                        state = SecondPartNumber;
                        current_spaces_in_a_row = 0;
                        current_commodity_is_quoted = false;
                    } else if c == '"' {
                        second_part_value.push(c);
                        if current_commodity_is_quoted {
                            state = SecondPartNumber;
                        }
                        current_commodity_is_quoted = true;
                    } else {
                        second_part_value.push(c);
                    }
                }
                SecondPartNumber => {
                    if c.is_ascii_digit() || c == '.' || c == ',' {
                        second_part_value.push(c);
                    } else if c == ' ' {
                        second_part_value.push(c);
                        if !second_part_value.is_empty() {
                            state = SecondPartCommodityAfter;
                        }
                    } else if c == '=' || c == '*' || c == '@' {
                        self.second_separator.push(c);
                        state = SecondSeparator;
                        current_spaces_in_a_row = 0;
                    } else {
                        second_part_value.push(c);
                        state = SecondPartCommodityAfter;
                        current_spaces_in_a_row = 0;
                    }
                }
                SecondPartCommodityAfter => {
                    if current_commodity_is_quoted {
                        if c == '"' {
                            second_part_value.push(c);
                            state = SecondSeparator;
                            current_commodity_is_quoted = false;
                        } else {
                            second_part_value.push(c);
                        }
                    } else if c.is_whitespace() {
                        state = SecondSeparator;
                        current_commodity_is_quoted = false;
                    } else if c == '@' || c == '*' || c == '=' {
                        self.first_separator.push(c);
                        state = SecondSeparator;
                        current_commodity_is_quoted = false;
                    } else if c == '"' {
                        second_part_value.push(c);
                        current_commodity_is_quoted = true;
                    } else {
                        // really numbers are forbidden by hledger, but don't care
                        second_part_value.push(c);
                    }
                }
                SecondSeparator => {
                    if c == '=' || c == '*' || c == '@' {
                        self.second_separator.push(c);
                    } else if !c.is_whitespace() {
                        third_part_value.push(c);
                        state = ThirdPartCommodityBefore;
                    }
                }
                ThirdPartCommodityBefore => {
                    if c.is_whitespace() {
                        if current_spaces_in_a_row == 0 {
                            if current_commodity_is_quoted {
                                third_part_value.push(c);
                            }
                            current_spaces_in_a_row += 1;
                        } else {
                            // no commodity
                            //current_spaces_in_a_row = 0;
                            // end
                            break;
                        }
                    } else if c.is_ascii_digit() || c == '.' || c == ',' {
                        third_part_value.push(c);
                        state = ThirdPartNumber;
                    } else if c == '"' {
                        third_part_value.push(c);
                        if current_commodity_is_quoted {
                            state = ThirdPartNumber;
                        }
                        current_commodity_is_quoted = true;
                    } else {
                        third_part_value.push(c);
                    }
                }
                ThirdPartNumber => {
                    if c.is_ascii_digit() || c == '.' || c == ',' {
                        third_part_value.push(c);
                    } else if c == ' ' {
                        if !third_part_value.is_empty() {
                            state = ThirdPartCommodityAfter;
                        }
                    } else {
                        third_part_value.push(c);
                        state = ThirdPartCommodityAfter;
                        current_spaces_in_a_row = 0;
                    }
                }
                ThirdPartCommodityAfter => {
                    if current_commodity_is_quoted {
                        third_part_value.push(c);
                        if c == '"' {
                            // end
                            break;
                        }
                    } else if c.is_whitespace() {
                        // end
                        break;
                    } else {
                        // really numbers are forbidden by hledger, but don't care
                        third_part_value.push(c);
                    }
                }
            }
        }

        if first_part_value.ends_with(' ') {
            first_part_value.pop();
        }
        if second_part_value.ends_with(' ') {
            second_part_value.pop();
        }
        if third_part_value.ends_with(' ') {
            third_part_value.pop();
        }

        let (before_decimals, after_decimals) =
            split_value_in_before_decimals_after_decimals(&first_part_value);
        self.first_part_before_decimals = before_decimals;
        self.first_part_after_decimals = after_decimals;

        let (before_decimals, after_decimals) =
            split_value_in_before_decimals_after_decimals(&second_part_value);
        self.second_part_before_decimals = before_decimals;
        self.second_part_after_decimals = after_decimals;

        let (before_decimals, after_decimals) =
            split_value_in_before_decimals_after_decimals(&third_part_value);
        self.third_part_before_decimals = before_decimals;
        self.third_part_after_decimals = after_decimals;

        Ok(())
    }
}

fn split_value_in_before_decimals_after_decimals(value: &str) -> (String, String) {
    if let Some(pos) = value.rfind(['.', ',']) {
        let after = &value[pos + 1..];
        if after.len() == 3 && after.chars().all(|c| c.is_ascii_digit()) {
            return (value.to_string(), "".to_string());
        } else {
            let before = &value[..pos];
            let after = &value[pos..];
            return (before.to_string(), after.to_string());
        }
    }

    let mut idx = 0;
    for c in value.chars() {
        if c.is_ascii_digit() || c == ',' || c == '.' || c == '-' || c == '+' {
            idx += c.len_utf8();
        } else {
            break;
        }
    }

    let (before, after) = value.split_at(idx);
    if before.is_empty() {
        if after.ends_with(|c: char| c.is_ascii_digit()) {
            // case $-1
            (after.to_string(), before.to_string())
        } else {
            // case $453534€
            for c in after.chars().rev() {
                if c.is_ascii_digit() {
                    break;
                }
                idx += c.len_utf8();
            }
            let (before, after) = value.split_at(after.len() - idx);
            (before.to_string(), after.to_string())
        }
    } else {
        (before.to_string(), after.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::split_value_in_before_decimals_after_decimals;

    #[test]
    fn test_split_value_in_before_decimals_after_decimals() {
        let (before, after) = split_value_in_before_decimals_after_decimals("1000.50€");
        assert_eq!(before, "1000");
        assert_eq!(after, ".50€");

        let (before, after) = split_value_in_before_decimals_after_decimals("2000,75 USD");
        assert_eq!(before, "2000");
        assert_eq!(after, ",75 USD");

        let (before, after) = split_value_in_before_decimals_after_decimals("3000 JPY");
        assert_eq!(before, "3000");
        assert_eq!(after, " JPY");

        let (before, after) = split_value_in_before_decimals_after_decimals("4000");
        assert_eq!(before, "4000");
        assert_eq!(after, "");

        let (before, after) = split_value_in_before_decimals_after_decimals("4000.");
        assert_eq!(before, "4000");
        assert_eq!(after, ".");

        let (before, after) = split_value_in_before_decimals_after_decimals("5,000");
        assert_eq!(before, "5,000");
        assert_eq!(after, "");

        let (before, after) = split_value_in_before_decimals_after_decimals("$-1");
        assert_eq!(before, "$-1");
        assert_eq!(after, "");

        let (before, after) =
            split_value_in_before_decimals_after_decimals("$-100000000000,000000000");
        assert_eq!(before, "$-100000000000");
        assert_eq!(after, ",000000000");

        let (before, after) = split_value_in_before_decimals_after_decimals("100€");
        assert_eq!(before, "100");
        assert_eq!(after, "€");

        let (before, after) = split_value_in_before_decimals_after_decimals("0 gold");
        assert_eq!(before, "0");
        assert_eq!(after, " gold");

        let (before, after) =
            split_value_in_before_decimals_after_decimals("0 \"Chocolate Frogs\"");
        assert_eq!(before, "0");
        assert_eq!(after, " \"Chocolate Frogs\"");

        let (before, after) = split_value_in_before_decimals_after_decimals("$56424324€");
        assert_eq!(before, "$56424324");
        assert_eq!(after, "€");

        let (before, after) = split_value_in_before_decimals_after_decimals("-10 gold");
        assert_eq!(before, "-10");
        assert_eq!(after, " gold");

        let (before, after) = split_value_in_before_decimals_after_decimals("2.0 AAAA");
        assert_eq!(before, "2");
        assert_eq!(after, ".0 AAAA");

        let (before, after) = split_value_in_before_decimals_after_decimals("$1.50");
        assert_eq!(before, "$1");
        assert_eq!(after, ".50");
    }

    #[test]
    fn test_entry_value_parser_stock_lot() {
        let mut parser = super::EntryValueParser::default();
        parser.parse("0.0 AAAA  =  2.0 AAAA  @   $1.50").unwrap();

        assert_eq!(parser.first_part_before_decimals, "0");
        assert_eq!(parser.first_part_after_decimals, ".0 AAAA");
        assert_eq!(parser.first_separator, "=");
        assert_eq!(parser.second_part_before_decimals, "2");
        assert_eq!(parser.second_part_after_decimals, ".0 AAAA");
        assert_eq!(parser.second_separator, "@");
        assert_eq!(parser.third_part_before_decimals, "$1");
        assert_eq!(parser.third_part_after_decimals, ".50");
    }

    #[test]
    fn test_entry_value_parser_chocolate_balance() {
        let mut parser = super::EntryValueParser::default();
        parser
            .parse(r#"0 "Chocolate Frogs"  =       3 "Chocolate Frogs""#)
            .unwrap();

        assert_eq!(parser.first_part_before_decimals, "0");
        assert_eq!(parser.first_part_after_decimals, " \"Chocolate Frogs\"");
        assert_eq!(parser.first_separator, "=");
        assert_eq!(parser.second_part_before_decimals, "3");
        assert_eq!(parser.second_part_after_decimals, " \"Chocolate Frogs\"");
        assert_eq!(parser.second_separator, "");
        assert_eq!(parser.third_part_before_decimals, "");
        assert_eq!(parser.third_part_after_decimals, "");
    }
}
