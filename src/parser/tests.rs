use crate::parser::{errors::*, *};

fn assert_journal(content: &str, expected: Vec<JournalCstNode>) {
    let journal = parse_content(content);
    assert_eq!(journal, Ok(expected));
}

fn assert_journal_err(content: &str, expected: SyntaxError) {
    let journal = parse_content(content);
    assert_eq!(journal, Err(expected));
}

#[test]
fn single_line_comment_hash() {
    assert_journal(
        "# comment",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment".to_string(),
            prefix: CommentPrefix::Hash,
            lineno: 1,
            colno: 1,
        })],
    );
}

#[test]
fn single_line_comment_semicolon() {
    assert_journal(
        "; comment",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment".to_string(),
            prefix: CommentPrefix::Semicolon,
            lineno: 1,
            colno: 1,
        })],
    );
}

#[test]
fn single_line_comment_indented() {
    assert_journal(
        "  # comment",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment".to_string(),
            prefix: CommentPrefix::Hash,
            lineno: 1,
            colno: 3,
        })],
    );

    assert_journal(
        "    ; comment # foo ; bar",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment # foo ; bar".to_string(),
            prefix: CommentPrefix::Semicolon,
            lineno: 1,
            colno: 5,
        })],
    );
}

#[test]
fn single_line_comment_tab_indented_hash() {
    assert_journal(
        "\t# comment",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment".to_string(),
            prefix: CommentPrefix::Hash,
            lineno: 1,
            // a tab is the second column in the line
            colno: 2,
        })],
    );
}

#[test]
fn single_line_comment_tab_indented_semicolon() {
    assert_journal(
        "\t\t; comment # foo ; bar",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment # foo ; bar".to_string(),
            prefix: CommentPrefix::Semicolon,
            lineno: 1,
            colno: 3,
        })],
    );
}

#[test]
fn multiline_comment() {
    assert_journal(
        "comment\ncontent\nend comment",
        vec![JournalCstNode::MultilineComment {
            content: "content\n".to_string(),
            lineno_start: 1,
            lineno_end: 3,
        }],
    );
}

#[test]
fn multiline_comment_not_ended() {
    // Hledger v1.40 traits not ended multiline comments as a multiline comment
    assert_journal(
        "comment\ncontent",
        vec![JournalCstNode::MultilineComment {
            content: "content\n".to_string(),
            lineno_start: 1,
            lineno_end: 2,
        }],
    );
}

#[test]
fn indented_text() {
    // Hledger v1.40 raises next error parsing indented text:
    //
    // ```
    //   |
    // 1 |   foo
    //   |   ^
    // unexpected 'f'
    // expecting newline
    // ```
    //
    // The error message is not totally accurate, because a newline is not
    // mandatory, a comment prefix is also valid.
    assert_journal_err(
        "  foo",
        SyntaxError {
            lineno: 1,
            colno_start: 3,
            colno_end: 4,
            message: "Unexpected character 'f'".to_string(),
            expected: "'#', ';' or newline",
        },
    );
}

#[test]
fn directive_with_tabbed_comment() {
    assert_journal(
        "account bank\t; comment",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![DirectiveNode::Directive(Directive {
                name: "account".to_string(),
                content: "bank".to_string(),
                comment: Some(SingleLineComment {
                    prefix: CommentPrefix::Semicolon,
                    content: " comment".to_string(),
                    colno: 17,
                    lineno: 1,
                }),
            })],
            max_name_content_len: 11,
        }],
    );
}

#[test]
fn directives_with_multiple_tabbed_comments() {
    assert_journal(
        "account foo\t\t; foo comment\naccount bar\t\t\t; bar comment",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![
                DirectiveNode::Directive(Directive {
                    name: "account".to_string(),
                    content: "foo".to_string(),
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " foo comment".to_string(),
                        colno: 20,
                        lineno: 1,
                    }),
                }),
                DirectiveNode::Directive(Directive {
                    name: "account".to_string(),
                    content: "bar".to_string(),
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " bar comment".to_string(),
                        colno: 24,
                        lineno: 2,
                    }),
                }),
            ],
            max_name_content_len: 10,
        }],
    );
}

#[test]
fn account_directive() {
    assert_journal(
        "account Assets:Bank:Checking",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![DirectiveNode::Directive(Directive {
                name: "account".to_string(),
                content: "Assets:Bank:Checking".to_string(),
                comment: None,
            })],
            max_name_content_len: 27, // "account" (7) + "Assets:Bank:Checking" (20),
        }],
    );
}

#[test]
fn account_directive_with_whitespace() {
    assert_journal(
        "account Assets Bank:Checking",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![DirectiveNode::Directive(Directive {
                name: "account".to_string(),
                content: "Assets Bank:Checking".to_string(),
                comment: None,
            })],
            max_name_content_len: 27,
        }],
    );
}

#[test]
fn account_directive_with_comment() {
    assert_journal(
        "account Assets:Bank:Checking  ; comment",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![DirectiveNode::Directive(Directive {
                name: "account".to_string(),
                content: "Assets:Bank:Checking".to_string(),
                comment: Some(SingleLineComment {
                    prefix: CommentPrefix::Semicolon,
                    content: " comment".to_string(),
                    colno: 31,
                    lineno: 1,
                }),
            })],
            max_name_content_len: 27,
        }],
    );
}

#[test]
fn commodity_directive() {
    assert_journal(
        "commodity $",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![DirectiveNode::Directive(Directive {
                name: "commodity".to_string(),
                content: "$".to_string(),
                comment: None,
            })],
            max_name_content_len: 10,
        }],
    );
}

#[test]
fn decimal_mark_directive() {
    assert_journal(
        "decimal-mark ,",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![DirectiveNode::Directive(Directive {
                name: "decimal-mark".to_string(),
                content: ",".to_string(),
                comment: None,
            })],
            max_name_content_len: 13,
        }],
    );
}

#[test]
fn payee_directive() {
    assert_journal(
        "payee Foo Bar",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![DirectiveNode::Directive(Directive {
                name: "payee".to_string(),
                content: "Foo Bar".to_string(),
                comment: None,
            })],
            max_name_content_len: 12,
        }],
    );
}

#[test]
fn tag_directive() {
    assert_journal(
        "tag foo bar",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![DirectiveNode::Directive(Directive {
                name: "tag".to_string(),
                content: "foo bar".to_string(),
                comment: None,
            })],
            max_name_content_len: 10,
        }],
    );
}

#[test]
fn include_directive() {
    assert_journal(
        "include /path/to/file",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![DirectiveNode::Directive(Directive {
                name: "include".to_string(),
                content: "/path/to/file".to_string(),
                comment: None,
            })],
            max_name_content_len: 20,
        }],
    );
}

#[test]
fn p_directive() {
    assert_journal(
        "P foobarbaz",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![DirectiveNode::Directive(Directive {
                name: "P".to_string(),
                content: "foobarbaz".to_string(),
                comment: None,
            })],
            max_name_content_len: 10,
        }],
    );
}

#[test]
fn directives_group() {
    assert_journal(
        "account Assets:Bank:Checking\ncommodity $\ndecimal-mark ,",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![
                DirectiveNode::Directive(Directive {
                    name: "account".to_string(),
                    content: "Assets:Bank:Checking".to_string(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "commodity".to_string(),
                    content: "$".to_string(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "decimal-mark".to_string(),
                    content: ",".to_string(),
                    comment: None,
                }),
            ],
            max_name_content_len: 27,
        }],
    );
}

#[test]
fn directives_group_with_comments() {
    assert_journal(
        "account foo:bar\n; comment\ncommodity $100.00  ; comment\n  # other comment",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![
                DirectiveNode::Directive(Directive {
                    name: "account".to_string(),
                    content: "foo:bar".to_string(),
                    comment: None,
                }),
                DirectiveNode::SingleLineComment(SingleLineComment {
                    content: " comment".to_string(),
                    prefix: CommentPrefix::Semicolon,
                    lineno: 2,
                    colno: 1,
                }),
                DirectiveNode::Directive(Directive {
                    name: "commodity".to_string(),
                    content: "$100.00".to_string(),
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " comment".to_string(),
                        colno: 20,
                        lineno: 3,
                    }),
                }),
                DirectiveNode::SingleLineComment(SingleLineComment {
                    content: " other comment".to_string(),
                    prefix: CommentPrefix::Hash,
                    lineno: 4,
                    colno: 3,
                }),
            ],
            max_name_content_len: 16,
        }],
    )
}
