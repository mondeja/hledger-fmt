use crate::{Box, Vec};

pub mod errors;
#[cfg(test)]
mod tests;
use crate::format;

use crate::byte_str::ByteStr;
use errors::SyntaxError;

/// A journal file
pub type JournalFile<'a> = Vec<JournalCstNode<'a>>;

/// Each node in a journal file
#[cfg_attr(any(test, feature = "tracing"), derive(Debug, PartialEq))]
pub enum JournalCstNode<'a> {
    /// An empty line
    EmptyLine,

    /// Multiline comment
    MultilineComment {
        /// The comment content
        content: ByteStr<'a>,
    },

    SingleLineComment(IndentedComment<'a>),

    /// Directives group
    DirectivesGroup {
        /// Directives in the group
        nodes: Vec<DirectiveNode<'a>>,
        /// Maximum length of the directive name + content (u16 max: 65,535 - more than sufficient)
        max_name_content_len: u16,
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
        title: ByteStr<'a>,
        /// Transaction title comment
        title_comment: Option<InlineComment<'a>>,
        /// Transaction entries
        entries: Vec<TransactionNode<'a>>,
        /// Indent of the first transaction entry (u16 max: 65,535 - more than sufficient)
        first_entry_indent: u16,
        /// Maximum length of the entry names (u16 max: 65,535 - more than sufficient)
        max_entry_name_len: u16,

        max_entry_value_first_part_before_decimals_len: u16,
        max_entry_value_first_part_after_decimals_len: u16,
        max_entry_value_first_separator_len: u16,
        max_entry_value_second_part_before_decimals_len: u16,
        max_entry_value_second_part_after_decimals_len: u16,
        max_entry_value_second_separator_len: u16,
        max_entry_value_third_part_before_decimals_len: u16,
        max_entry_value_third_part_after_decimals_len: u16,
    },
}

/// Prefix of a single line comment
#[repr(u8)]
#[derive(Clone, Copy)]
#[cfg_attr(any(test, feature = "tracing"), derive(Debug, PartialEq))]
pub enum CommentPrefix {
    /// '#'
    Hash = b'#',
    /// ';'
    Semicolon = b';',
}

impl CommentPrefix {
    #[inline]
    fn from_byte(byte: u8) -> Self {
        if byte == b'#' {
            CommentPrefix::Hash
        } else {
            CommentPrefix::Semicolon
        }
    }
}

/// A single line comment without indentation (inline comment)
#[cfg_attr(any(test, feature = "tracing"), derive(Debug, PartialEq))]
pub struct InlineComment<'a> {
    /// The comment content
    pub content: ByteStr<'a>,
    /// The comment prefix ('#' or ';')
    pub prefix: CommentPrefix,
}

/// A single line comment that starts at the beginning of a line and tracks indentation
#[cfg_attr(any(test, feature = "tracing"), derive(Debug, PartialEq))]
pub struct IndentedComment<'a> {
    /// The comment content
    pub content: ByteStr<'a>,
    /// The column number where the comment starts (u16 covers practical indent widths)
    pub indent: u16,
    /// The comment prefix ('#' or ';')
    pub prefix: CommentPrefix,
}

/// A directive
#[cfg_attr(any(test, feature = "tracing"), derive(Debug, PartialEq))]
pub struct Directive<'a> {
    /// The directive name
    pub name: ByteStr<'a>,
    /// The directive content
    pub content: ByteStr<'a>,
    /// Comment associated with the directive
    pub comment: Option<InlineComment<'a>>,
    /// Cached character count for name
    pub(crate) name_chars_count: u16,
    /// Cached character count for content
    pub(crate) content_chars_count: u16,
}

/// A directive or a single line comment
#[cfg_attr(any(test, feature = "tracing"), derive(Debug, PartialEq))]
pub enum DirectiveNode<'a> {
    Directive(Directive<'a>),
    Subdirective(ByteStr<'a>), // includes comments after the subdirective content
    SingleLineComment(IndentedComment<'a>),
}

/// A transaction entry
#[cfg_attr(any(test, feature = "tracing"), derive(Debug, PartialEq))]
pub struct TransactionEntry<'a> {
    /// Entry name
    pub name: ByteStr<'a>,
    pub value_first_part_before_decimals: ByteStr<'a>,
    pub value_first_part_after_decimals: ByteStr<'a>,
    pub value_first_separator: ByteStr<'a>,
    pub value_second_part_before_decimals: ByteStr<'a>,
    pub value_second_part_after_decimals: ByteStr<'a>,
    pub value_second_separator: ByteStr<'a>,
    pub value_third_part_before_decimals: ByteStr<'a>,
    pub value_third_part_after_decimals: ByteStr<'a>,
    /// Comment associated with the entry
    pub comment: Option<InlineComment<'a>>,
    /// Cached character counts
    pub(crate) name_chars_count: u16,
    pub(crate) value_first_part_before_decimals_chars_count: u16,
    pub(crate) value_first_part_after_decimals_chars_count: u16,
    pub(crate) value_second_part_before_decimals_chars_count: u16,
    pub(crate) value_second_part_after_decimals_chars_count: u16,
    pub(crate) value_third_part_before_decimals_chars_count: u16,
    pub(crate) value_third_part_after_decimals_chars_count: u16,
}

/// A transaction entry or a single line comment
#[cfg_attr(any(test, feature = "tracing"), derive(Debug, PartialEq))]
pub enum TransactionNode<'a> {
    TransactionEntry(Box<TransactionEntry<'a>>),
    SingleLineComment(IndentedComment<'a>),
}

#[derive(Default)]
/// Temporary data used by the parser
struct ParserTempData<'a> {
    /// Location of the content of the current multiline comment
    multiline_comment_byte_start: usize,
    multiline_comment_byte_end: usize,
    /// Directives group nodes
    directives_group_nodes: Vec<DirectiveNode<'a>>,
    /// Maximum length of the directive names + contents (u16 max: 65,535 - more than sufficient)
    directives_group_max_name_content_len: u16,
    /// Transaction title
    transaction_title_byte_start: usize,
    transaction_title_byte_end: usize,
    /// Transaction title comment
    transaction_title_comment: Option<InlineComment<'a>>,
    /// Transaction entries
    transaction_entries: Vec<TransactionNode<'a>>,
    /// If the current transaction has entries (ignoring comments)
    transaction_has_no_comment_entries: bool,
    /// Indent of the first transaction entry (u16 max: 65,535 - more than sufficient)
    first_entry_indent: u16,
    /// Maximum length of the entry names (u16 max: 65,535 - more than sufficient)
    max_entry_name_len: u16,
    max_entry_value_first_part_before_decimals_len: u16,
    max_entry_value_first_part_after_decimals_len: u16,
    max_entry_value_first_separator_len: u16,
    max_entry_value_second_part_before_decimals_len: u16,
    max_entry_value_second_part_after_decimals_len: u16,
    max_entry_value_second_separator_len: u16,
    max_entry_value_third_part_before_decimals_len: u16,
    max_entry_value_third_part_after_decimals_len: u16,
    /// Reusable entry value parser to avoid allocations
    entry_value_parser: EntryValueParser,
}

impl<'a> ParserTempData<'a> {
    fn new() -> Self {
        Self::default()
    }
}

pub fn parse_content<'a>(bytes: &'a [u8]) -> Result<JournalFile<'a>, errors::SyntaxError> {
    #[cfg(any(test, feature = "tracing"))]
    {
        // TODO: if not using here, the `bytes` argument is propagared to all children
        // even when using skip(bytes)
        let _span = tracing::span!(
            tracing::Level::TRACE,
            "parse_content",
            bytes = format!("{}", crate::tracing::Utf8Slice(bytes))
        )
        .entered();
    }

    let mut inside_multiline_comment = false;
    let mut data = ParserTempData::new();
    // Start with a modest capacity; Vec grows as needed without huge upfront allocations.
    let mut journal = Vec::with_capacity(16);

    let mut lineno = 1;
    let mut byteno = 0;
    let bytes_length = bytes.len();
    while byteno < bytes_length {
        let newline_pos = memchr::memchr(b'\n', &bytes[byteno..]);

        let (line_end_including_newline, line_end) = match newline_pos {
            Some(pos) => {
                let end_with_newline = byteno + pos + 1;
                let end_without_newline =
                    if pos > 0 && *unsafe { bytes.get_unchecked(byteno + pos - 1) } == b'\r' {
                        byteno + pos - 1 // CRLF
                    } else {
                        byteno + pos // LF
                    };
                (end_with_newline, end_without_newline)
            }
            None => (bytes_length, bytes_length), // last line without newline
        };

        if line_end == byteno {
            // empty line
            process_empty_line(&mut journal, &mut data, bytes);
            byteno = line_end_including_newline;
            lineno += 1;
            continue;
        }

        let line = unsafe { bytes.get_unchecked(byteno..line_end) };

        #[cfg(any(test, feature = "tracing"))]
        {
            let _span = tracing::span!(
                tracing::Level::TRACE,
                "parse_content(line)",
                line = format!("{}", crate::tracing::Utf8Slice(line)),
                lineno,
                inside_multiline_comment,
            )
            .entered();
        }

        if inside_multiline_comment {
            if line == b"end comment" {
                save_multiline_comment(&mut data, &mut journal, bytes);
                inside_multiline_comment = false;
            } else {
                if data.multiline_comment_byte_start == 0 {
                    data.multiline_comment_byte_start = byteno;
                }
                data.multiline_comment_byte_end = line_end_including_newline;
            }

            byteno = line_end_including_newline;
            lineno += 1;
            continue;
        }

        let first_byte = unsafe { *line.get_unchecked(0) };
        if first_byte.is_ascii_whitespace() {
            let line_length = line.len();
            let last_byte = unsafe { *line.get_unchecked(line_length - 1) };
            let all_whitespace = if line_length < 2 {
                // Single character line that is whitespace
                true
            } else {
                last_byte.is_ascii_whitespace()
                    && unsafe { line.get_unchecked(1..line_length - 1) }
                        .iter()
                        .all(|&b| b.is_ascii_whitespace())
            };

            if all_whitespace {
                // empty line (only spaces or tabs)
                process_empty_line(&mut journal, &mut data, bytes);
            } else if data.transaction_title_byte_start == 0 && data.transaction_title_byte_end == 0
            {
                // probably single line comment that starts with a space,
                // but could be also a subdirective
                parse_single_line_comment_or_subdirective(line, lineno, &mut data, &mut journal)?;
            } else {
                // maybe inside transaction entry, but could be also a single
                // line comment inside a transaction group
                parse_transaction_entry(line, &mut data);
            }
        } else if first_byte == b'#' || first_byte == b';' {
            #[cfg(any(test, feature = "tracing"))]
            {
                let _span = tracing::span!(
                    tracing::Level::TRACE,
                    "parse_content(single line comment)",
                    "line[0]" = format!("{}", first_byte as char),
                )
                .entered();
            }
            // single line comment
            let prefix = CommentPrefix::from_byte(first_byte);

            let content = ByteStr::from(&line[1..]);
            let comment = IndentedComment {
                content,
                prefix,
                indent: 0,
            };

            if data.directives_group_nodes.is_empty()
                && data.transaction_title_byte_start == 0
                && data.transaction_title_byte_end == 0
            {
                journal.push(JournalCstNode::SingleLineComment(comment));
            } else if data.transaction_title_byte_start != 0 || data.transaction_title_byte_end != 0
            {
                data.transaction_entries
                    .push(TransactionNode::SingleLineComment(comment));
            } else {
                data.directives_group_nodes
                    .push(DirectiveNode::SingleLineComment(comment));
            }
        } else if let Some(name) = unsafe { maybe_start_with_directive(line) } {
            parse_directive(name, line, &mut data);
        } else if line != b"comment" {
            // starts transaction

            // if we are in a current transaction, save it adding a newline
            if data.transaction_title_byte_start != 0 || data.transaction_title_byte_end != 0 {
                process_empty_line(&mut journal, &mut data, bytes);
            }

            data.transaction_title_byte_start = byteno;
            parse_transaction_title(line, &mut data);
        } else {
            inside_multiline_comment = true;
        }

        byteno = line_end_including_newline;
        lineno += 1;
    }

    // Hledger v1.40 traits not ended multiline comments as a multiline comment
    if inside_multiline_comment {
        if data.multiline_comment_byte_start != 0 && data.multiline_comment_byte_end == 0 {
            data.multiline_comment_byte_end = byteno;
        }

        save_multiline_comment(&mut data, &mut journal, bytes);
    } else if !data.directives_group_nodes.is_empty() {
        save_directives_group_nodes(&mut data, &mut journal);
    } else if data.transaction_title_byte_start != 0 || data.transaction_title_byte_end != 0 {
        save_transaction(&mut data, &mut journal, bytes);
    }

    Ok(journal)
}

#[cfg_attr(
    any(test, feature = "tracing"),
    tracing::instrument(
        level = "trace",
        skip(journal, bytes),
        fields(
            data = %{
                let directives_count = data.directives_group_nodes.len();
                let transaction_title_byte_start = data.transaction_title_byte_start;
                let transaction_title_byte_end = data.transaction_title_byte_end;
                format!(
                    "[directives={directives_count} \
                    transaction_title_byte_start={transaction_title_byte_start}, \
                    transaction_title_byte_end={transaction_title_byte_end}]"
                )
            }
        )
    )
)]
fn process_empty_line<'a>(
    journal: &mut Vec<JournalCstNode<'a>>,
    data: &mut ParserTempData<'a>,
    bytes: &'a [u8],
) {
    if !data.directives_group_nodes.is_empty() {
        save_directives_group_nodes(data, journal);
    } else if data.transaction_title_byte_start != 0 || data.transaction_title_byte_end != 0 {
        save_transaction(data, journal, bytes);
    }
    journal.push(JournalCstNode::EmptyLine);
}

#[cfg_attr(
    any(test, feature = "tracing"),
    tracing::instrument(
        level = "trace",
        fields(
            line = %format!("{:?}", crate::tracing::Utf8Slice(line)),
            name = %format!("{:?}", crate::tracing::Utf8Slice(name)),
            data = %{
                let directives_count = data.directives_group_nodes.len();
                let directives_group_max_name_content_len = data.directives_group_max_name_content_len;
                format!("[directives={directives_count} \
                         directives_group_max_name_content_len={directives_group_max_name_content_len}]")
            }
        )
    )
)]
fn parse_directive<'a>(name: &'a [u8], line: &'a [u8], data: &mut ParserTempData<'a>) {
    let name_length = name.len();
    let line_length = line.len();
    let mut start = name_length;
    if start < line_length && is_directive_delimiter(line[start]) {
        while start < line_length && is_directive_delimiter(line[start]) {
            start += 1;
        }
    }
    let mut end = start;

    let mut prev_was_whitespace = false;
    let mut comment_colno_padding = 1;

    while end < line_length {
        let c = line[end];
        if c == b'\t' {
            comment_colno_padding = 4;
            break;
        }

        if c.is_ascii_whitespace() {
            if prev_was_whitespace {
                // double whitespace, end of content
                end -= 1;
                break;
            }
            prev_was_whitespace = true;
        } else {
            prev_was_whitespace = false;
        }
        end += 1;
    }
    let mut comment = None;
    if end != start {
        // it should be a comment
        comment = parse_inline_comment(line, line_length, comment_colno_padding, None);
    }

    let name = ByteStr::from(name);
    let content = ByteStr::from(&line[start..end]);
    save_directive(name, content, comment, data);
}

#[cfg_attr(
    any(test, feature = "tracing"),
    tracing::instrument(
        level = "trace",
        skip(data),
        fields(
            comment = %match &comment {
                Some(c) => format!("Some({:?})", crate::tracing::Utf8Slice(&c.content)),
                None => "None".to_string(),
            },
        )
    )
)]
fn save_directive<'a>(
    name: ByteStr<'a>,
    content: ByteStr<'a>,
    comment: Option<InlineComment<'a>>,
    data: &mut ParserTempData<'a>,
) {
    let content_len = content.chars_count();
    let name_length = name.chars_count();
    data.directives_group_nodes
        .push(DirectiveNode::Directive(Directive {
            name,
            content,
            comment,
            name_chars_count: name_length as u16,
            content_chars_count: content_len as u16,
        }));
    data.directives_group_max_name_content_len = data
        .directives_group_max_name_content_len
        .max((content_len + name_length) as u16);
}

#[cfg_attr(
    any(test, feature = "tracing"),
    tracing::instrument(
        level = "trace",
        fields(
            line = %format!("{:?}", crate::tracing::Utf8Slice(line)),
        )
    )
)]
#[inline]
fn parse_inline_comment<'a>(
    line: &'a [u8],
    line_length: usize,
    colno_padding: usize,
    from_comment_prefix: Option<CommentPrefix>,
) -> Option<InlineComment<'a>> {
    let (content_bytes, prefix) = if let Some(comment_prefix) = from_comment_prefix {
        (&line[colno_padding..line_length], comment_prefix)
    } else {
        // Use memchr2 to efficiently find comment markers
        let search_slice = &line[colno_padding..line_length];
        let pos = memchr::memchr2(b'#', b';', search_slice)?;
        let comment_prefix = CommentPrefix::from_byte(search_slice[pos]);
        let start = colno_padding + pos + 1;
        (&line[start..line_length], comment_prefix)
    };

    Some(InlineComment {
        content: ByteStr::from(content_bytes),
        prefix,
    })
}

#[cfg_attr(
    any(test, feature = "tracing"),
    tracing::instrument(
        level = "trace",
        skip(journal),
        fields(
            line = %format!("{:?}", crate::tracing::Utf8Slice(line)),
            data = %{
                let directives_count = data.directives_group_nodes.len();
                let entries_count = data.transaction_entries.len();
                format!("[directives={directives_count} entries={entries_count}]")
            }
        )
    )
)]
fn parse_single_line_comment_or_subdirective<'a>(
    line: &'a [u8],
    lineno: usize,
    data: &mut ParserTempData<'a>,
    journal: &mut Vec<JournalCstNode<'a>>,
) -> Result<(), SyntaxError> {
    let mut is_subdirective = false;
    let mut comment_prefix = None;
    let mut end = 0;
    let line_length = line.len();

    while end < line_length {
        let c = line[end];
        if c == b'#' {
            comment_prefix = Some(CommentPrefix::Hash);
            end += 1;
            break;
        } else if c == b';' {
            comment_prefix = Some(CommentPrefix::Semicolon);
            end += 1;
            break;
        } else if !c.is_ascii_whitespace() {
            // if we're inside a directives group, is a subdirective
            if !data.directives_group_nodes.is_empty() {
                is_subdirective = true;
                break;
            }

            return Err(SyntaxError {
                message: format!("Unexpected character {:?}", c as char),
                lineno,
                colno_start: end + 1,
                colno_end: end + 2,
                expected: "'#', ';' or newline",
            });
        }
        end += 1;
    }

    let content_start = end;
    let end = line_length;

    let content = ByteStr::from(&line[content_start..end]);
    if let Some(prefix) = comment_prefix {
        let comment = IndentedComment {
            content,
            prefix,
            indent: (content_start - 1) as u16,
        };
        if data.directives_group_nodes.is_empty() {
            journal.push(JournalCstNode::SingleLineComment(comment));
        } else if data.transaction_title_byte_start != 0 || data.transaction_title_byte_end != 0 {
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

    Ok(())
}

#[cfg_attr(
    any(test, feature = "tracing"),
    tracing::instrument(
        level = "trace",
        skip(bytes, journal),
        fields(
            data = %{
                let multiline_comment_byte_start = data.multiline_comment_byte_start;
                let multiline_comment_byte_end = data.multiline_comment_byte_end;
                format!(
                    "[multiline_comment_byte_start={multiline_comment_byte_start} \
                      multiline_comment_byte_end={multiline_comment_byte_end}]"
                )
            }
        )
    )
)]
fn save_multiline_comment<'a>(
    data: &mut ParserTempData<'a>,
    journal: &mut Vec<JournalCstNode<'a>>,
    bytes: &'a [u8],
) {
    let content =
        ByteStr::from(&bytes[data.multiline_comment_byte_start..data.multiline_comment_byte_end]);
    journal.push(JournalCstNode::MultilineComment { content });
    data.multiline_comment_byte_start = 0;
    data.multiline_comment_byte_end = 0;
}

#[cfg_attr(
    any(test, feature = "tracing"),
    tracing::instrument(
        level = "trace",
        skip(journal),
        fields(
            data = %{
                let directives_count = data.directives_group_nodes.len();
                let directives_group_max_name_content_len = data.directives_group_max_name_content_len;
                format!(
                    "[directives={directives_count} \
                     directives_group_max_name_content_len={directives_group_max_name_content_len}]"
                )
            }
        )
    )
)]
fn save_directives_group_nodes<'a>(
    data: &mut ParserTempData<'a>,
    journal: &mut Vec<JournalCstNode<'a>>,
) {
    journal.push(JournalCstNode::DirectivesGroup {
        nodes: core::mem::take(&mut data.directives_group_nodes),
        max_name_content_len: data.directives_group_max_name_content_len,
    });
    data.directives_group_max_name_content_len = 0;
}

#[cfg_attr(
    any(test, feature = "tracing"),
    tracing::instrument(
        level = "trace",
        fields(
            line = %format!("{:?}", crate::tracing::Utf8Slice(line)),
            data = %{
                let first_entry_indent = data.first_entry_indent;
                let transaction_has_no_comment_entries = data.transaction_has_no_comment_entries;
                let entries_count = data.transaction_entries.len();
                format!(
                    "[first_entry_indent={first_entry_indent} \
                     transaction_has_no_comment_entries={transaction_has_no_comment_entries} \
                     entries={entries_count}]"
                )
            }
        )
    )
)]
fn parse_transaction_entry<'a>(line: &'a [u8], data: &mut ParserTempData<'a>) {
    let first_byte = unsafe { *line.get_unchecked(0) };
    let at_indent = first_byte != b'\t';
    let mut indent = if at_indent { 0 } else { 4 };
    let mut entry_name_start = 0;
    let mut entry_name_end = 0;
    let mut prev_was_whitespace = first_byte.is_ascii_whitespace();

    let line_length = line.len();
    let start = 0;
    let mut end = start;
    if at_indent {
        while end < line_length {
            let c = line[end];
            end += 1;
            if c == b'\t' {
                indent += 4;
            } else if c.is_ascii_whitespace() {
                indent += 1;
            } else if c == b';' || c == b'#' {
                // transaction entry with empty value
                let maybe_comment = parse_inline_comment(
                    line,
                    line_length,
                    end,
                    Some(if c == b'#' {
                        CommentPrefix::Hash
                    } else {
                        CommentPrefix::Semicolon
                    }),
                );
                if let Some(comment) = maybe_comment {
                    // if the first comment is indented with >=2 and first entry indent
                    // is not setted, set it
                    //
                    // this is needed for transactions without entries, only comments
                    if indent >= 2 && data.first_entry_indent == 0 {
                        data.first_entry_indent = indent as u16;
                    }
                    data.transaction_entries
                        .push(TransactionNode::SingleLineComment(IndentedComment {
                            content: comment.content,
                            prefix: comment.prefix,
                            indent: indent as u16,
                        }));
                    return; // is comment only
                }
                break;
            } else {
                entry_name_start = end - 1;
                entry_name_end = entry_name_start;
                break;
            }
        }
    } else {
        end += 1; // skip first tab
    }

    while end < line_length {
        let c = line[end];
        end += 1;
        if c == b'\t' {
            break;
        } else if c.is_ascii_whitespace() {
            if prev_was_whitespace {
                entry_name_end = entry_name_end.saturating_sub(1);
                break;
            }
            prev_was_whitespace = true;
        } else {
            prev_was_whitespace = false;
            if c == b';' && entry_name_end == 0 {
                // inside comment in transactions group
                let maybe_comment =
                    parse_inline_comment(line, line_length, end, Some(CommentPrefix::Semicolon));
                if let Some(comment) = maybe_comment {
                    data.transaction_entries
                        .push(TransactionNode::SingleLineComment(IndentedComment {
                            content: comment.content,
                            prefix: comment.prefix,
                            indent: indent as u16,
                        }));
                    return; // is comment only
                }
                break;
            }
        }
        entry_name_end = end;
    }

    // Ensure entry_name_end is not less than entry_name_start to avoid panic
    let entry_name_end = entry_name_end.max(entry_name_start);
    let entry_name = ByteStr::from(&line[entry_name_start..entry_name_end]);

    if data.first_entry_indent == 0 {
        data.first_entry_indent = indent as u16;
    } else if !data.transaction_has_no_comment_entries {
        // if the first entry is a comment, the indent is not
        // properly setted so we need to set it here
        data.first_entry_indent = indent as u16;
    }
    data.max_entry_name_len = data.max_entry_name_len.max(entry_name.chars_count() as u16);

    let mut inside_entry_value = false;
    let mut comment = None;
    let mut entry_value_start = end;
    let mut entry_value_end = entry_value_start;

    while end < line_length {
        let c = line[end];
        end += 1;
        if !inside_entry_value {
            if c == b';' || c == b'#' {
                // transaction entry with empty value
                comment = parse_inline_comment(
                    line,
                    line_length,
                    end,
                    Some(if c == b'#' {
                        CommentPrefix::Hash
                    } else {
                        CommentPrefix::Semicolon
                    }),
                );
                break;
            } else if !c.is_ascii_whitespace() {
                inside_entry_value = true;
                entry_value_start = end - 1;
                entry_value_end = entry_value_start;
            }
        } else if c == b';' || c == b'#' {
            comment = parse_inline_comment(
                line,
                line_length,
                end,
                Some(if c == b'#' {
                    CommentPrefix::Hash
                } else {
                    CommentPrefix::Semicolon
                }),
            );
            break;
        } else {
            entry_value_end = end;
        }
    }

    // let coln = entry_value.chars().count()
    //     + entry_name.chars().count()
    //     + indent + 1;
    let entry_value = &line[entry_value_start..entry_value_end];
    data.entry_value_parser.reset();
    let p = data.entry_value_parser.parse(entry_value);

    data.max_entry_value_first_part_before_decimals_len = data
        .max_entry_value_first_part_before_decimals_len
        .max(p.first_part_before_decimals.chars_count() as u16);
    data.max_entry_value_first_part_after_decimals_len = data
        .max_entry_value_first_part_after_decimals_len
        .max(p.first_part_after_decimals.chars_count() as u16);

    data.max_entry_value_first_separator_len = data
        .max_entry_value_first_separator_len
        .max(p.first_separator.len() as u16);

    data.max_entry_value_second_part_before_decimals_len = data
        .max_entry_value_second_part_before_decimals_len
        .max(p.second_part_before_decimals.chars_count() as u16);
    data.max_entry_value_second_part_after_decimals_len = data
        .max_entry_value_second_part_after_decimals_len
        .max(p.second_part_after_decimals.chars_count() as u16);

    data.max_entry_value_second_separator_len = data
        .max_entry_value_second_separator_len
        .max(p.second_separator.len() as u16);

    data.max_entry_value_third_part_before_decimals_len = data
        .max_entry_value_third_part_before_decimals_len
        .max(p.third_part_before_decimals.chars_count() as u16);
    data.max_entry_value_third_part_after_decimals_len = data
        .max_entry_value_third_part_after_decimals_len
        .max(p.third_part_after_decimals.chars_count() as u16);

    // Cache the character counts we just computed
    let name_chars_count = entry_name.chars_count() as u16;
    let value_first_part_before_decimals_chars_count =
        p.first_part_before_decimals.chars_count() as u16;
    let value_first_part_after_decimals_chars_count =
        p.first_part_after_decimals.chars_count() as u16;
    let value_second_part_before_decimals_chars_count =
        p.second_part_before_decimals.chars_count() as u16;
    let value_second_part_after_decimals_chars_count =
        p.second_part_after_decimals.chars_count() as u16;
    let value_third_part_before_decimals_chars_count =
        p.third_part_before_decimals.chars_count() as u16;
    let value_third_part_after_decimals_chars_count =
        p.third_part_after_decimals.chars_count() as u16;

    data.transaction_has_no_comment_entries = true;
    data.transaction_entries
        .push(TransactionNode::TransactionEntry(Box::new(
            TransactionEntry {
                name: entry_name,
                value_first_part_before_decimals: p.first_part_before_decimals,
                value_first_part_after_decimals: p.first_part_after_decimals,
                value_first_separator: p.first_separator,
                value_second_part_before_decimals: p.second_part_before_decimals,
                value_second_part_after_decimals: p.second_part_after_decimals,
                value_second_separator: p.second_separator,
                value_third_part_before_decimals: p.third_part_before_decimals,
                value_third_part_after_decimals: p.third_part_after_decimals,
                comment,
                name_chars_count,
                value_first_part_before_decimals_chars_count,
                value_first_part_after_decimals_chars_count,
                value_second_part_before_decimals_chars_count,
                value_second_part_after_decimals_chars_count,
                value_third_part_before_decimals_chars_count,
                value_third_part_after_decimals_chars_count,
            },
        )));
}

#[cfg_attr(
    any(test, feature = "tracing"),
    tracing::instrument(
        level = "trace",
        skip(data),
        fields(
            line = %format!("{:?}", crate::tracing::Utf8Slice(line)),
        )
    )
)]
fn parse_transaction_title<'a>(line: &'a [u8], data: &mut ParserTempData<'a>) {
    let line_length = line.len();
    let mut end = 0;
    let mut comment_prefix = None;
    // TODO: handle title + description to standarize?
    while end < line_length {
        let c = line[end];
        end += 1;
        if c == b';' {
            comment_prefix = Some(CommentPrefix::Semicolon);
            break;
        } else if c == b'#' {
            comment_prefix = Some(CommentPrefix::Hash);
            break;
        }
    }

    let original_end = end;

    if end < line_length {
        // go backwards to not include spaces in title
        let mut ends_with_space = line[end] == b' ' || line[end] == b'\t';
        while ends_with_space {
            end -= 1;
            ends_with_space = line[end - 1] == b' ' || line[end - 1] == b'\t';
        }
    }

    data.transaction_title_byte_end = data.transaction_title_byte_start + end;
    if comment_prefix.is_none() {
        return;
    }
    data.transaction_title_comment =
        parse_inline_comment(line, line_length, original_end, comment_prefix);
}

#[cfg_attr(
    any(test, feature = "tracing"),
    tracing::instrument(
        level = "trace",
        skip(bytes, journal),
        fields(
            data = %{
                let transaction_title_byte_start = data.transaction_title_byte_start;
                let transaction_title_byte_end = data.transaction_title_byte_end;
                let entries_count = data.transaction_entries.len();
                format!(
                    "[transaction_title_byte_start={transaction_title_byte_start} \
                     transaction_title_byte_end={transaction_title_byte_end} \
                     entries={entries_count}]"
                )
            }
        )
    )
)]
fn save_transaction<'a>(
    data: &mut ParserTempData<'a>,
    journal: &mut Vec<JournalCstNode<'a>>,
    bytes: &'a [u8],
) {
    let title =
        ByteStr::from(&bytes[data.transaction_title_byte_start..data.transaction_title_byte_end]);
    journal.push(JournalCstNode::Transaction {
        title,
        title_comment: data.transaction_title_comment.take(),
        entries: core::mem::take(&mut data.transaction_entries),
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
    data.transaction_title_byte_start = 0;
    data.transaction_title_byte_end = 0;
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

#[inline(always)]
const fn is_directive_delimiter(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

/// Extremely performant function to check if a line starts with a directive
/// and return the name of the directive.
///
/// Supposes that:
///
/// - line is not empty
#[inline]
unsafe fn maybe_start_with_directive(line: &[u8]) -> Option<&[u8]> {
    /*
        "account "
        "commodity "
        "decimal-mark "
        "payee " (x)
        "tag "
        "include "
        "P "
        "apply account"
        "D "
        "Y "
                                 // other Ledger directives
        "apply fixed"
        "apply tag"
        "assert "
        "capture "
        "check "
        "define "
        "bucket / A "
        "end apply fixed"
        "end apply tag"
        "end apply year"
        "end tag"
        "eval "
        "expr "
        "python "
        "value "
        "--command-line-flags"  // longest directive, 20 chars
    */
    let l = line;
    let line_length = l.len();

    if line_length < 2 {
        return None;
    }

    let first_char = *l.get_unchecked(0);

    // "P ", "D ", "Y "
    if (first_char == b'P' || first_char == b'D' || first_char == b'Y')
        && *l.get_unchecked(1) == b' '
    {
        return Some(&line[0..1]);
    }

    if line_length < 4 {
        return None;
    }

    // "tag "
    if first_char == b't'
        && *l.get_unchecked(1) == b'a'
        && *l.get_unchecked(2) == b'g'
        && *l.get_unchecked(3) == b' '
    {
        return Some(&line[0..3]);
    }

    if line_length < 5 {
        return None;
    }

    // Group by first letter: 'e' for "expr", "eval", "end..."
    if first_char == b'e' {
        // "expr " / "eval "
        if (*l.get_unchecked(1) == b'x'
            && *l.get_unchecked(2) == b'p'
            && *l.get_unchecked(3) == b'r'
            && *l.get_unchecked(4) == b' ')
            || (*l.get_unchecked(1) == b'v'
                && *l.get_unchecked(2) == b'a'
                && *l.get_unchecked(3) == b'l'
                && *l.get_unchecked(4) == b' ')
        {
            return Some(&line[0..4]);
        }

        if line_length >= 7
            && *l.get_unchecked(1) == b'n'
            && *l.get_unchecked(2) == b'd'
            && *l.get_unchecked(3) == b' '
        {
            // "end tag"
            if *l.get_unchecked(4) == b't'
                && *l.get_unchecked(5) == b'a'
                && *l.get_unchecked(6) == b'g'
            {
                return Some(&line[0..7]);
            }

            if line_length >= 13
                && *l.get_unchecked(4) == b'a'
                && *l.get_unchecked(5) == b'p'
                && *l.get_unchecked(6) == b'p'
                && *l.get_unchecked(7) == b'l'
                && *l.get_unchecked(8) == b'y'
                && is_directive_delimiter(*l.get_unchecked(9))
            {
                // "end apply tag"
                if *l.get_unchecked(10) == b't'
                    && *l.get_unchecked(11) == b'a'
                    && *l.get_unchecked(12) == b'g'
                {
                    return Some(&line[0..13]);
                }

                // "end apply year"
                if line_length >= 14
                    && *l.get_unchecked(10) == b'y'
                    && *l.get_unchecked(11) == b'e'
                    && *l.get_unchecked(12) == b'a'
                    && *l.get_unchecked(13) == b'r'
                {
                    return Some(&line[0..14]);
                }

                // "end apply fixed"
                if line_length >= 15
                    && *l.get_unchecked(10) == b'f'
                    && *l.get_unchecked(11) == b'i'
                    && *l.get_unchecked(12) == b'x'
                    && *l.get_unchecked(13) == b'e'
                    && *l.get_unchecked(14) == b'd'
                {
                    return Some(&line[0..15]);
                }
            }
        }
    }

    if line_length < 6 {
        return None;
    }

    // Group by first letter: 'p' for "payee", "python"
    if first_char == b'p' {
        // "payee "
        if *l.get_unchecked(1) == b'a'
            && *l.get_unchecked(2) == b'y'
            && *l.get_unchecked(3) == b'e'
            && *l.get_unchecked(4) == b'e'
            && is_directive_delimiter(*l.get_unchecked(5))
        {
            return Some(&line[0..5]);
        }

        // "python "
        if line_length >= 7
            && *l.get_unchecked(1) == b'y'
            && *l.get_unchecked(2) == b't'
            && *l.get_unchecked(3) == b'h'
            && *l.get_unchecked(4) == b'o'
            && *l.get_unchecked(5) == b'n'
            && is_directive_delimiter(*l.get_unchecked(6))
        {
            return Some(&line[0..6]);
        }
    }

    // "value "
    if first_char == b'v'
        && *l.get_unchecked(1) == b'a'
        && *l.get_unchecked(2) == b'l'
        && *l.get_unchecked(3) == b'u'
        && *l.get_unchecked(4) == b'e'
        && is_directive_delimiter(*l.get_unchecked(5))
    {
        return Some(&line[0..5]);
    }

    if line_length < 7 {
        return None;
    }

    // Group by first letter: 'd' for "define", "decimal-mark"
    if first_char == b'd' && *l.get_unchecked(1) == b'e' {
        // "define "
        if *l.get_unchecked(2) == b'f'
            && *l.get_unchecked(3) == b'i'
            && *l.get_unchecked(4) == b'n'
            && *l.get_unchecked(5) == b'e'
            && is_directive_delimiter(*l.get_unchecked(6))
        {
            return Some(&line[0..6]);
        }

        // "decimal-mark "
        if line_length >= 13
            && *l.get_unchecked(2) == b'c'
            && *l.get_unchecked(3) == b'i'
            && *l.get_unchecked(4) == b'm'
            && *l.get_unchecked(5) == b'a'
            && *l.get_unchecked(6) == b'l'
            && *l.get_unchecked(7) == b'-'
            && *l.get_unchecked(8) == b'm'
            && *l.get_unchecked(9) == b'a'
            && *l.get_unchecked(10) == b'r'
            && *l.get_unchecked(11) == b'k'
            && is_directive_delimiter(*l.get_unchecked(12))
        {
            return Some(&line[0..12]);
        }
    }

    if line_length < 8 {
        return None;
    }

    // Group by first letter: 'a' for "account", "assert", "apply..."
    if first_char == b'a' {
        // "account "
        if *l.get_unchecked(1) == b'c'
            && *l.get_unchecked(2) == b'c'
            && *l.get_unchecked(3) == b'o'
            && *l.get_unchecked(4) == b'u'
            && *l.get_unchecked(5) == b'n'
            && *l.get_unchecked(6) == b't'
            && is_directive_delimiter(*l.get_unchecked(7))
        {
            return Some(&line[0..7]);
        }

        // "assert "
        if *l.get_unchecked(1) == b's'
            && *l.get_unchecked(2) == b's'
            && *l.get_unchecked(3) == b'e'
            && *l.get_unchecked(4) == b'r'
            && *l.get_unchecked(5) == b't'
            && is_directive_delimiter(*l.get_unchecked(6))
        {
            return Some(&line[0..6]);
        }

        // "apply..."
        if line_length >= 9
            && *l.get_unchecked(1) == b'p'
            && *l.get_unchecked(2) == b'p'
            && *l.get_unchecked(3) == b'l'
            && *l.get_unchecked(4) == b'y'
            && is_directive_delimiter(*l.get_unchecked(5))
        {
            // "apply tag"
            if *l.get_unchecked(6) == b't'
                && *l.get_unchecked(7) == b'a'
                && *l.get_unchecked(8) == b'g'
            {
                return Some(&line[0..9]);
            }

            // "apply fixed"
            if line_length >= 11
                && *l.get_unchecked(6) == b'f'
                && *l.get_unchecked(7) == b'i'
                && *l.get_unchecked(8) == b'x'
                && *l.get_unchecked(9) == b'e'
                && *l.get_unchecked(10) == b'd'
            {
                return Some(&line[0..11]);
            }

            // "apply account"
            if line_length >= 13
                && *l.get_unchecked(6) == b'a'
                && *l.get_unchecked(7) == b'c'
                && *l.get_unchecked(8) == b'c'
                && *l.get_unchecked(9) == b'o'
                && *l.get_unchecked(10) == b'u'
                && *l.get_unchecked(11) == b'n'
                && *l.get_unchecked(12) == b't'
            {
                return Some(&line[0..13]);
            }
        }
    }

    // Group by first letter: 'c' for "capture", "commodity", "check"
    if first_char == b'c' {
        // "capture "
        if *l.get_unchecked(1) == b'a'
            && *l.get_unchecked(2) == b'p'
            && *l.get_unchecked(3) == b't'
            && *l.get_unchecked(4) == b'u'
            && *l.get_unchecked(5) == b'r'
            && *l.get_unchecked(6) == b'e'
            && is_directive_delimiter(*l.get_unchecked(7))
        {
            return Some(&line[0..7]);
        }

        // "check "
        if *l.get_unchecked(1) == b'h'
            && *l.get_unchecked(2) == b'e'
            && *l.get_unchecked(3) == b'c'
            && *l.get_unchecked(4) == b'k'
            && is_directive_delimiter(*l.get_unchecked(5))
        {
            return Some(&line[0..5]);
        }

        // "commodity "
        if line_length >= 10
            && *l.get_unchecked(1) == b'o'
            && *l.get_unchecked(2) == b'm'
            && *l.get_unchecked(3) == b'm'
            && *l.get_unchecked(4) == b'o'
            && *l.get_unchecked(5) == b'd'
            && *l.get_unchecked(6) == b'i'
            && *l.get_unchecked(7) == b't'
            && *l.get_unchecked(8) == b'y'
            && is_directive_delimiter(*l.get_unchecked(9))
        {
            return Some(&line[0..9]);
        }
    }

    // "include "
    if first_char == b'i'
        && *l.get_unchecked(1) == b'n'
        && *l.get_unchecked(2) == b'c'
        && *l.get_unchecked(3) == b'l'
        && *l.get_unchecked(4) == b'u'
        && *l.get_unchecked(5) == b'd'
        && *l.get_unchecked(6) == b'e'
        && is_directive_delimiter(*l.get_unchecked(7))
    {
        return Some(&line[0..7]);
    }

    if line_length < 11 {
        return None;
    }

    // "bucket / A "
    if first_char == b'b'
        && *l.get_unchecked(1) == b'u'
        && *l.get_unchecked(2) == b'c'
        && *l.get_unchecked(3) == b'k'
        && *l.get_unchecked(4) == b'e'
        && *l.get_unchecked(5) == b't'
        && is_directive_delimiter(*l.get_unchecked(6))
        && *l.get_unchecked(7) == b'/'
        && is_directive_delimiter(*l.get_unchecked(8))
        && *l.get_unchecked(9) == b'A'
        && is_directive_delimiter(*l.get_unchecked(10))
    {
        return Some(&line[0..10]);
    }

    if line_length < 20 {
        return None;
    }

    // "--command-line-flags"
    if first_char == b'-'
        && *l.get_unchecked(1) == b'-'
        && *l.get_unchecked(2) == b'c'
        && *l.get_unchecked(3) == b'o'
        && *l.get_unchecked(4) == b'm'
        && *l.get_unchecked(5) == b'm'
        && *l.get_unchecked(6) == b'a'
        && *l.get_unchecked(7) == b'n'
        && *l.get_unchecked(8) == b'd'
        && *l.get_unchecked(9) == b'-'
        && *l.get_unchecked(10) == b'l'
        && *l.get_unchecked(11) == b'i'
        && *l.get_unchecked(12) == b'n'
        && *l.get_unchecked(13) == b'e'
        && *l.get_unchecked(14) == b'-'
        && *l.get_unchecked(15) == b'f'
        && *l.get_unchecked(16) == b'l'
        && *l.get_unchecked(17) == b'a'
        && *l.get_unchecked(18) == b'g'
        && *l.get_unchecked(19) == b's'
    {
        return Some(&line[0..20]);
    }

    None
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
#[derive(Default)]
pub(crate) struct EntryValueParser {
    // Using u16 for positions within entry value (max 65,535 chars - more than sufficient)
    first_part_value_start: u16,
    first_part_value_end: u16,
    first_separator_start: u16,
    first_separator_end: u16,
    second_part_value_start: u16,
    second_part_value_end: u16,
    second_separator_start: u16,
    second_separator_end: u16,
    third_part_value_start: u16,
    third_part_value_end: u16,
}

pub(crate) struct EntryValueParserReturn<'a> {
    pub first_part_before_decimals: ByteStr<'a>,
    pub first_part_after_decimals: ByteStr<'a>,
    pub first_separator: ByteStr<'a>,
    pub second_part_before_decimals: ByteStr<'a>,
    pub second_part_after_decimals: ByteStr<'a>,
    pub second_separator: ByteStr<'a>,
    pub third_part_before_decimals: ByteStr<'a>,
    pub third_part_after_decimals: ByteStr<'a>,
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
    /// Reset the parser state for reuse
    #[inline]
    fn reset(&mut self) {
        *self = Self::default();
    }

    #[inline(always)]
    fn is_number_char(c: u8) -> bool {
        c.is_ascii_digit() || c == b'.' || c == b','
    }

    #[inline(always)]
    fn is_separator_char(c: u8) -> bool {
        c == b'@' || c == b'*' || c == b'='
    }

    #[inline]
    fn update_first_part_value(&mut self, end: usize) {
        if self.first_part_value_start == 0 && self.first_part_value_end == 0 {
            self.first_part_value_start = (end - 1) as u16;
        }
        self.first_part_value_end = end as u16;
    }

    #[inline]
    fn is_first_part_value_empty(&self) -> bool {
        self.first_part_value_start == 0 && self.first_part_value_end == 0
    }

    #[inline]
    fn update_first_separator(&mut self, end: usize) {
        if self.first_separator_start == 0 && self.first_separator_end == 0 {
            self.first_separator_start = (end - 1) as u16;
        }
        self.first_separator_end = end as u16;
    }

    #[inline]
    fn update_second_part_value(&mut self, end: usize) {
        if self.second_part_value_start == 0 && self.second_part_value_end == 0 {
            self.second_part_value_start = (end - 1) as u16;
        }
        self.second_part_value_end = end as u16;
    }

    #[inline]
    fn is_second_part_value_empty(&self) -> bool {
        self.second_part_value_start == 0 && self.second_part_value_end == 0
    }

    #[inline]
    fn update_second_separator(&mut self, end: usize) {
        if self.second_separator_start == 0 && self.second_separator_end == 0 {
            self.second_separator_start = (end - 1) as u16;
        }
        self.second_separator_end = end as u16;
    }

    #[inline]
    fn update_third_part_value(&mut self, end: usize) {
        if self.third_part_value_start == 0 && self.third_part_value_end == 0 {
            self.third_part_value_start = (end - 1) as u16;
        }
        self.third_part_value_end = end as u16;
    }

    #[inline]
    fn is_third_part_value_empty(&self) -> bool {
        self.third_part_value_start == 0 && self.third_part_value_end == 0
    }

    pub(crate) fn parse<'a>(&mut self, value: &'a [u8]) -> EntryValueParserReturn<'a> {
        //let chars = value.chars();
        let value_length = value.len();

        use EntryValueParserState::*;
        let mut state = FirstPartCommodityBefore;

        let mut current_spaces_in_a_row = 0;
        let mut current_commodity_is_quoted = false;

        /*
        let mut first_part_value = String::with_capacity(value_length);
        let mut second_part_value = String::with_capacity(value_length);
        let mut third_part_value = String::with_capacity(value_length);
        */

        let mut end = 0;
        while end < value_length {
            let c = value[end];
            end += 1;
            match state {
                FirstPartCommodityBefore => {
                    if c.is_ascii_whitespace() {
                        current_spaces_in_a_row += 1;
                        if current_commodity_is_quoted {
                            self.update_first_part_value(end);
                        } else if current_spaces_in_a_row > 1 {
                            // no commodity
                            state = FirstSeparator;
                            current_spaces_in_a_row = 0;
                            current_commodity_is_quoted = false;
                        } else {
                            // first space
                            current_spaces_in_a_row = 1;
                        }
                    } else if Self::is_number_char(c) {
                        self.update_first_part_value(end);
                        state = FirstPartNumber;
                        current_spaces_in_a_row = 0;
                        current_commodity_is_quoted = false;
                    } else if c == b'"' {
                        self.update_first_part_value(end);
                        if current_commodity_is_quoted {
                            state = FirstPartNumber;
                        }
                        current_commodity_is_quoted = true;
                    } else {
                        self.update_first_part_value(end);
                    }
                }
                FirstPartNumber => {
                    if Self::is_number_char(c) {
                        self.update_first_part_value(end);
                    } else if c.is_ascii_whitespace() {
                        if self.is_first_part_value_empty() {
                            continue;
                        }
                        state = FirstPartCommodityAfter;
                    } else if Self::is_separator_char(c) {
                        self.update_first_separator(end);
                        state = FirstSeparator;
                    } else if c == b'"' {
                        self.update_first_part_value(end);
                        state = FirstPartCommodityAfter;
                    } else {
                        if c == b'"' {
                            current_commodity_is_quoted = true;
                        }
                        self.update_first_part_value(end);
                        state = FirstPartCommodityAfter;
                        current_spaces_in_a_row = 0;
                    }
                }
                FirstPartCommodityAfter => {
                    if current_commodity_is_quoted {
                        self.update_first_part_value(end);
                        if c == b'"' {
                            state = FirstSeparator;
                            current_commodity_is_quoted = false;
                        }
                    } else if c.is_ascii_whitespace() {
                        state = FirstSeparator;
                        current_commodity_is_quoted = false;
                    } else if Self::is_separator_char(c) {
                        self.update_first_separator(end);
                        state = FirstSeparator;
                        current_commodity_is_quoted = false;
                    } else if c == b'"' {
                        self.update_first_part_value(end);
                        current_commodity_is_quoted = true;
                    } else {
                        // really numbers are forbidden by hledger, but don't care
                        self.update_first_part_value(end);
                    }
                }
                FirstSeparator => {
                    if Self::is_separator_char(c) {
                        self.update_first_separator(end);
                    } else if !c.is_ascii_whitespace() {
                        self.update_second_part_value(end);
                        state = SecondPartCommodityBefore;
                        current_spaces_in_a_row = 0;
                    }
                }
                SecondPartCommodityBefore => {
                    if c.is_ascii_whitespace() {
                        current_spaces_in_a_row += 1;
                        if current_commodity_is_quoted || current_spaces_in_a_row < 2 {
                            self.update_second_part_value(end);
                        } else {
                            // no commodity
                            state = SecondSeparator;
                            current_spaces_in_a_row = 0;
                            current_commodity_is_quoted = false;
                        }
                    } else if Self::is_number_char(c) {
                        self.update_second_part_value(end);
                        state = SecondPartNumber;
                        current_spaces_in_a_row = 0;
                        current_commodity_is_quoted = false;
                    } else if c == b'"' {
                        self.update_second_part_value(end);
                        if current_commodity_is_quoted {
                            state = SecondPartNumber;
                        }
                        current_commodity_is_quoted = true;
                    } else {
                        self.update_second_part_value(end);
                    }
                }
                SecondPartNumber => {
                    if Self::is_number_char(c) {
                        self.update_second_part_value(end);
                    } else if c.is_ascii_whitespace() {
                        if self.is_second_part_value_empty() {
                            continue;
                        }
                        self.update_second_part_value(end);
                        state = SecondPartCommodityAfter;
                    } else if Self::is_separator_char(c) {
                        self.update_second_separator(end);
                        state = SecondSeparator;
                        current_spaces_in_a_row = 0;
                    } else {
                        self.update_second_part_value(end);
                        if c == b'"' {
                            current_commodity_is_quoted = true;
                        }
                        state = SecondPartCommodityAfter;
                        current_spaces_in_a_row = 0;
                    }
                }
                SecondPartCommodityAfter => {
                    if current_commodity_is_quoted {
                        self.update_second_part_value(end);
                        if c == b'"' {
                            state = SecondSeparator;
                            current_commodity_is_quoted = false;
                        }
                    } else if c.is_ascii_whitespace() {
                        state = SecondSeparator;
                        current_commodity_is_quoted = false;
                    } else if Self::is_separator_char(c) {
                        self.update_second_separator(end);
                        state = SecondSeparator;
                        current_commodity_is_quoted = false;
                    } else if c == b'"' {
                        self.update_second_part_value(end);
                        current_commodity_is_quoted = true;
                    } else {
                        // really numbers are forbidden by hledger, but don't care
                        self.update_second_part_value(end);
                    }
                }
                SecondSeparator => {
                    if Self::is_separator_char(c) {
                        self.update_second_separator(end);
                    } else if !c.is_ascii_whitespace() {
                        self.update_third_part_value(end);
                        state = ThirdPartCommodityBefore;
                    }
                }
                ThirdPartCommodityBefore => {
                    if c.is_ascii_whitespace() {
                        if current_spaces_in_a_row == 0 {
                            if current_commodity_is_quoted {
                                self.update_third_part_value(end);
                            }
                            current_spaces_in_a_row += 1;
                        } else {
                            // no commodity
                            //current_spaces_in_a_row = 0;
                            // end
                            break;
                        }
                    } else if Self::is_number_char(c) {
                        self.update_third_part_value(end);
                        state = ThirdPartNumber;
                    } else if c == b'"' {
                        self.update_third_part_value(end);
                        if current_commodity_is_quoted {
                            state = ThirdPartNumber;
                        }
                        current_commodity_is_quoted = true;
                    } else {
                        self.update_third_part_value(end);
                    }
                }
                ThirdPartNumber => {
                    if Self::is_number_char(c) {
                        self.update_third_part_value(end);
                    } else if c.is_ascii_whitespace() {
                        if self.is_third_part_value_empty() {
                            continue;
                        }
                        state = ThirdPartCommodityAfter;
                    } else {
                        self.update_third_part_value(end);
                        if c == b'"' {
                            current_commodity_is_quoted = true;
                        }
                        state = ThirdPartCommodityAfter;
                        current_spaces_in_a_row = 0;
                    }
                }
                ThirdPartCommodityAfter => {
                    if current_commodity_is_quoted {
                        self.update_third_part_value(end);
                        if c == b'"' {
                            // end
                            break;
                        }
                    } else if c.is_ascii_whitespace() {
                        // end
                        break;
                    } else {
                        // really numbers are forbidden by hledger, but don't care
                        self.update_third_part_value(end);
                    }
                }
            }
        }
        if self.first_part_value_end > 0 && value[self.first_part_value_end as usize - 1] == b' ' {
            self.first_part_value_end -= 1;
        }
        if self.second_part_value_end > 0 && value[self.second_part_value_end as usize - 1] == b' '
        {
            self.second_part_value_end -= 1;
        }
        if self.third_part_value_end > 0 && value[self.third_part_value_end as usize - 1] == b' ' {
            self.third_part_value_end -= 1;
        }

        let first_part_value =
            &value[self.first_part_value_start as usize..self.first_part_value_end as usize];
        let second_part_value =
            &value[self.second_part_value_start as usize..self.second_part_value_end as usize];
        let third_part_value =
            &value[self.third_part_value_start as usize..self.third_part_value_end as usize];

        let (first_part_value_before_decimals, first_part_value_after_decimals) =
            split_value_in_before_decimals_after_decimals(first_part_value);
        let (second_part_value_before_decimals, second_part_value_after_decimals) =
            split_value_in_before_decimals_after_decimals(second_part_value);
        let (third_part_value_before_decimals, third_part_value_after_decimals) =
            split_value_in_before_decimals_after_decimals(third_part_value);

        EntryValueParserReturn {
            first_part_before_decimals: ByteStr::from(first_part_value_before_decimals),
            first_part_after_decimals: ByteStr::from(first_part_value_after_decimals),
            first_separator: ByteStr::from(
                &value[self.first_separator_start as usize..self.first_separator_end as usize],
            ),
            second_part_before_decimals: ByteStr::from(second_part_value_before_decimals),
            second_part_after_decimals: ByteStr::from(second_part_value_after_decimals),
            second_separator: ByteStr::from(
                &value[self.second_separator_start as usize..self.second_separator_end as usize],
            ),
            third_part_before_decimals: ByteStr::from(third_part_value_before_decimals),
            third_part_after_decimals: ByteStr::from(third_part_value_after_decimals),
        }
    }
}

#[inline(always)]
fn split_value_in_before_decimals_after_decimals(value: &[u8]) -> (&[u8], &[u8]) {
    // Use memchr2 for faster decimal point search (rightmost position)
    if let Some(pos) = memchr::memrchr2(b'.', b',', value) {
        let after = &value[pos + 1..];
        // Fast path: check for thousands separator (3 digits after decimal)
        // Use unsafe for faster bounds-checked access since we know the length
        if after.len() == 3
            && unsafe {
                after.get_unchecked(0).is_ascii_digit()
                    && after.get_unchecked(1).is_ascii_digit()
                    && after.get_unchecked(2).is_ascii_digit()
            }
        {
            return (value, &[]);
        }
        let before = &value[..pos];
        let after = &value[pos..];
        return (before, after);
    }

    // Find the first non-numeric character using iterator
    let idx = value
        .iter()
        .position(|&c| !(c.is_ascii_digit() || c == b',' || c == b'.' || c == b'-' || c == b'+'))
        .unwrap_or(value.len());

    let (before, after) = value.split_at(idx);
    if before.is_empty() && !after.is_empty() {
        // SAFETY: We just checked that after is not empty
        if unsafe { *after.get_unchecked(after.len() - 1) }.is_ascii_digit() {
            // case $-1
            (after, before)
        } else {
            // case $453534€
            let trailing_non_digits = after
                .iter()
                .rev()
                .position(|&c| c.is_ascii_digit())
                .unwrap_or(after.len());
            let (before, after) = value.split_at(value.len() - trailing_non_digits);
            (before, after)
        }
    } else {
        (before, after)
    }
}

#[cfg(test)]
mod test {
    use super::{split_value_in_before_decimals_after_decimals, EntryValueParser};

    #[test]
    fn test_split_value_in_before_decimals_after_decimals() {
        let (before, after) = split_value_in_before_decimals_after_decimals("1000.50€".as_bytes());
        assert_eq!(before, b"1000");
        assert_eq!(after, ".50€".as_bytes());

        let (before, after) = split_value_in_before_decimals_after_decimals(b"2000,75 USD");
        assert_eq!(before, b"2000");
        assert_eq!(after, b",75 USD");

        let (before, after) = split_value_in_before_decimals_after_decimals(b"3000 JPY");
        assert_eq!(before, b"3000");
        assert_eq!(after, b" JPY");

        let (before, after) = split_value_in_before_decimals_after_decimals(b"4000");
        assert_eq!(before, b"4000");
        assert_eq!(after, b"");

        let (before, after) = split_value_in_before_decimals_after_decimals(b"4000.");
        assert_eq!(before, b"4000");
        assert_eq!(after, b".");

        let (before, after) = split_value_in_before_decimals_after_decimals(b"5,000");
        assert_eq!(before, b"5,000");
        assert_eq!(after, b"");

        let (before, after) = split_value_in_before_decimals_after_decimals("$-1".as_bytes());
        assert_eq!(before, "$-1".as_bytes());
        assert_eq!(after, b"");

        let (before, after) =
            split_value_in_before_decimals_after_decimals("$-100000000000,000000000".as_bytes());
        assert_eq!(before, "$-100000000000".as_bytes());
        assert_eq!(after, b",000000000");

        let (before, after) = split_value_in_before_decimals_after_decimals("100€".as_bytes());
        assert_eq!(before, b"100");
        assert_eq!(after, "€".as_bytes());

        let (before, after) = split_value_in_before_decimals_after_decimals(b"0 gold");
        assert_eq!(before, b"0");
        assert_eq!(after, b" gold");

        let (before, after) =
            split_value_in_before_decimals_after_decimals(b"0 \"Chocolate Frogs\"");
        assert_eq!(before, b"0");
        assert_eq!(after, b" \"Chocolate Frogs\"");

        let (before, after) =
            split_value_in_before_decimals_after_decimals("$56424324€".as_bytes());
        assert_eq!(before, "$56424324".as_bytes());
        assert_eq!(after, "€".as_bytes());

        let (before, after) = split_value_in_before_decimals_after_decimals(b"-10 gold");
        assert_eq!(before, b"-10");
        assert_eq!(after, b" gold");

        let (before, after) = split_value_in_before_decimals_after_decimals(b"2.0 AAAA");
        assert_eq!(before, b"2");
        assert_eq!(after, b".0 AAAA");

        let (before, after) = split_value_in_before_decimals_after_decimals("$1.50".as_bytes());
        assert_eq!(before, "$1".as_bytes());
        assert_eq!(after, b".50");
    }

    #[test]
    fn test_entry_value_parser_stock_lot() {
        let mut parser = EntryValueParser::default();
        let p = parser.parse("0.0 AAAA  =  2.0 AAAA  @   $1.50".as_bytes());

        assert_eq!(p.first_part_before_decimals, "0".as_bytes().into());
        assert_eq!(p.first_part_after_decimals, ".0 AAAA".as_bytes().into());
        assert_eq!(p.first_separator, "=".as_bytes().into());
        assert_eq!(p.second_part_before_decimals, "2".as_bytes().into());
        assert_eq!(p.second_part_after_decimals, ".0 AAAA".as_bytes().into());
        assert_eq!(p.second_separator, "@".as_bytes().into());
        assert_eq!(p.third_part_before_decimals, "$1".as_bytes().into());
        assert_eq!(p.third_part_after_decimals, ".50".as_bytes().into());
    }

    #[test]
    fn test_entry_value_parser_chocolate_balance() {
        let mut parser = EntryValueParser::default();
        let p = parser.parse(br#"0 "Chocolate Frogs"  =       3 "Chocolate Frogs""#);

        assert_eq!(p.first_part_before_decimals, "0".as_bytes().into());
        assert_eq!(
            p.first_part_after_decimals,
            " \"Chocolate Frogs\"".as_bytes().into()
        );
        assert_eq!(p.first_separator, "=".as_bytes().into());
        assert_eq!(p.second_part_before_decimals, "3".as_bytes().into());
        assert_eq!(
            p.second_part_after_decimals,
            " \"Chocolate Frogs\"".as_bytes().into()
        );
        assert_eq!(p.second_separator, "".as_bytes().into());
        assert_eq!(p.third_part_before_decimals, "".as_bytes().into());
        assert_eq!(p.third_part_after_decimals, "".as_bytes().into());
    }
}
