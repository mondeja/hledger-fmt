use crate::parser::{errors::*, *};

fn assert_journal(content: &str, expected: Vec<JournalCstNode>) {
    let journal = parse_content(content.as_bytes());
    assert_eq!(journal, Ok(expected));
}

fn assert_journal_err(content: &str, expected: SyntaxError) {
    let journal = parse_content(content.as_bytes());
    assert_eq!(journal, Err(expected));
}

#[test]
fn single_line_comment_hash() {
    assert_journal(
        "# comment",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment".into(),
            prefix: CommentPrefix::Hash,
            indent: 0,
        })],
    );
}

#[test]
fn single_line_comment_semicolon() {
    assert_journal(
        "; comment",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment".into(),
            prefix: CommentPrefix::Semicolon,
            indent: 0,
        })],
    );
}

#[test]
fn single_line_comment_indented() {
    assert_journal(
        "  # comment",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment".into(),
            prefix: CommentPrefix::Hash,
            indent: 2,
        })],
    );

    assert_journal(
        "    ; comment # foo ; bar",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment # foo ; bar".into(),
            prefix: CommentPrefix::Semicolon,
            indent: 4,
        })],
    );
}

#[test]
fn single_line_comment_tab_indented_hash() {
    assert_journal(
        "\t# comment",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment".into(),
            prefix: CommentPrefix::Hash,
            // a tab is the second column in the line
            indent: 1,
        })],
    );
}

#[test]
fn single_line_comment_tab_indented_semicolon() {
    assert_journal(
        "\t\t; comment # foo ; bar",
        vec![JournalCstNode::SingleLineComment(SingleLineComment {
            content: " comment # foo ; bar".into(),
            prefix: CommentPrefix::Semicolon,
            indent: 2,
        })],
    );
}

#[test]
fn multiline_comment() {
    assert_journal(
        "comment\ncontent\nend comment",
        vec![JournalCstNode::MultilineComment {
            content: "content\n".into(),
        }],
    );
}

#[test]
fn multiline_comment_not_ended() {
    // Hledger v1.40 traits not ended multiline comments as a multiline comment
    assert_journal(
        "comment\ncontent",
        vec![JournalCstNode::MultilineComment {
            content: "content\n".into(),
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
                name: "account".into(),
                content: "bank".into(),
                comment: Some(SingleLineComment {
                    prefix: CommentPrefix::Semicolon,
                    content: " comment".into(),
                    indent: 16,
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
                    name: "account".into(),
                    content: "foo".into(),
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " foo comment".into(),
                        indent: 19,
                    }),
                }),
                DirectiveNode::Directive(Directive {
                    name: "account".into(),
                    content: "bar".into(),
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " bar comment".into(),
                        indent: 23,
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
                name: "account".into(),
                content: "Assets:Bank:Checking".into(),
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
                name: "account".into(),
                content: "Assets Bank:Checking".into(),
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
                name: "account".into(),
                content: "Assets:Bank:Checking".into(),
                comment: Some(SingleLineComment {
                    prefix: CommentPrefix::Semicolon,
                    content: " comment".into(),
                    indent: 1, // TODO: idents for directive comments are not used in the formatter
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
                name: "commodity".into(),
                content: "$".into(),
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
                name: "decimal-mark".into(),
                content: ",".into(),
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
                name: "payee".into(),
                content: "Foo Bar".into(),
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
                name: "tag".into(),
                content: "foo bar".into(),
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
                name: "include".into(),
                content: "/path/to/file".into(),
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
                name: "P".into(),
                content: "foobarbaz".into(),
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
                    name: "account".into(),
                    content: "Assets:Bank:Checking".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "commodity".into(),
                    content: "$".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "decimal-mark".into(),
                    content: ",".into(),
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
                    name: "account".into(),
                    content: "foo:bar".into(),
                    comment: None,
                }),
                DirectiveNode::SingleLineComment(SingleLineComment {
                    content: " comment".into(),
                    prefix: CommentPrefix::Semicolon,
                    indent: 0,
                }),
                DirectiveNode::Directive(Directive {
                    name: "commodity".into(),
                    content: "$100.00".into(),
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " comment".into(),
                        indent: 19,
                    }),
                }),
                DirectiveNode::SingleLineComment(SingleLineComment {
                    content: " other comment".into(),
                    prefix: CommentPrefix::Hash,
                    indent: 2,
                }),
            ],
            max_name_content_len: 16,
        }],
    )
}

#[test]
fn subdirective() {
    assert_journal(
        "account Assets:Bank:Checking\n  subdirective foo bar",
        vec![JournalCstNode::DirectivesGroup {
            nodes: vec![
                DirectiveNode::Directive(Directive {
                    name: "account".into(),
                    content: "Assets:Bank:Checking".into(),
                    comment: None,
                }),
                DirectiveNode::Subdirective("subdirective foo bar".into()),
            ],
            max_name_content_len: 27,
        }],
    );
}
