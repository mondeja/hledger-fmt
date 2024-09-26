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
            content: vec![DirectiveNode::Directive(Directive {
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
            content: vec![
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
            content: vec![DirectiveNode::Directive(Directive {
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
            content: vec![DirectiveNode::Directive(Directive {
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
            content: vec![DirectiveNode::Directive(Directive {
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
            content: vec![DirectiveNode::Directive(Directive {
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
            content: vec![DirectiveNode::Directive(Directive {
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
            content: vec![DirectiveNode::Directive(Directive {
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
            content: vec![DirectiveNode::Directive(Directive {
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
            content: vec![DirectiveNode::Directive(Directive {
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
            content: vec![DirectiveNode::Directive(Directive {
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
            content: vec![
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
            content: vec![
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

/*
#[test]
fn transaction_with_tabbed_entry() {
    assert_journal(
        "2021-01-01 my-title\n\texpenses:rent\t$1000",
        vec![JournalCstNode::Transaction {
            title: "2021-01-01 my-title".to_string(),
            title_comment: None,
            entries: vec![TransactionNode::TransactionEntry(
                TransactionEntry {
                    name: "expenses:rent".to_string(),
                    value: "$1000".to_string(),
                    comment: None,
                    value_units_len: 5,
                    value_decimal_len: 0,
                },
            )],
            first_entry_indent: 4,
            max_entry_name_len: 13,
            max_entry_value_len: 5,
            max_entry_units_len: 5,
            max_entry_decimal_len: 0,
            max_entry_after_decimal_len: 0,
        }],
    );
}

#[test]
fn periodic_transaction() {
    assert_journal(
        //          two spaces inside title for periodic transactions
        r#"~ monthly  set budget goals
  (expenses:rent)      $1000
  (expenses:food)       $500"#,
        vec![JournalCstNode::Transaction {
            title: "~ monthly  set budget goals".to_string(),
            title_comment: None,
            max_entry_units_len: 5,
            entries: vec![
                TransactionNode::TransactionEntry(TransactionEntry {
                    name: "(expenses:rent)".to_string(),
                    value: "$1000".to_string(),
                    comment: None,
                    value_units_len: 5,
                    value_decimal_len: 0,
                }),
                TransactionNode::TransactionEntry(TransactionEntry {
                    name: "(expenses:food)".to_string(),
                    value: "$500".to_string(),
                    comment: None,
                    value_units_len: 4,
                    value_decimal_len: 0,
                }),
            ],
            first_entry_indent: 2,
            max_entry_name_len: 15,
            max_entry_value_len: 5,
            max_entry_decimal_len: 0,
            max_entry_after_decimal_len: 0,
        }],
    )
}

#[test]
fn auto_posting_rule() {
    assert_journal(
        r#"= revenues:consulting
    liabilities:tax:2024:us          *0.25  ; Add a tax liability & expense
    expenses:tax:2024:us            *-0.25  ; for 25% of the revenue."#,
        vec![JournalCstNode::Transaction {
            title: "= revenues:consulting".to_string(),
            title_comment: None,
            entries: vec![
                TransactionNode::TransactionEntry(TransactionEntry {
                    name: "liabilities:tax:2024:us".to_string(),
                    value: "*0.25".to_string(),
                    value_units_len: 3,
                    value_decimal_len: 2,
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " Add a tax liability & expense".to_string(),
                        lineno: 2,
                        colno: 45,
                    }),
                }),
                TransactionNode::TransactionEntry(TransactionEntry {
                    name: "expenses:tax:2024:us".to_string(),
                    value: "*-0.25".to_string(),
                    value_units_len: 4,
                    value_decimal_len: 2,
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " for 25% of the revenue.".to_string(),
                        lineno: 3,
                        colno: 45,
                    }),
                }),
            ],
            first_entry_indent: 4,
            max_entry_name_len: 23,
            max_entry_value_len: 6,
            max_entry_units_len: 4,
            max_entry_decimal_len: 2,
            max_entry_after_decimal_len: 0,
        }],
    )
}

#[test]
fn full_transaction() {
    assert_journal(
        r#"2024-01-01 opening balances         ; At the start, declare pre-existing balances this way.
    assets:savings          $10000  ; Account names can be anything. lower case is easy to type.
    assets:checking          $1000  ; assets, liabilities, equity, revenues, expenses are common.
    liabilities:credit card  $-500  ; liabilities, equity, revenues balances are usually negative.
    equity:start                    ; One amount can be left blank. $-10500 is inferred here.
                                    ; Some of these accounts we didn't declare above,
                                    ; so -s/--strict would complain."#,
        vec![JournalCstNode::Transaction {
            title: "2024-01-01 opening balances".to_string(),
            title_comment: Some(SingleLineComment {
                prefix: CommentPrefix::Semicolon,
                content: " At the start, declare pre-existing balances this way.".to_string(),
                lineno: 1,
                colno: 37,
            }),
            entries: vec![
                TransactionNode::TransactionEntry(TransactionEntry {
                    name: "assets:savings".to_string(),
                    value: "$10000".to_string(),
                    value_units_len: 6,
                    value_decimal_len: 0,
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " Account names can be anything. lower case is easy to type."
                            .to_string(),
                        lineno: 2,
                        colno: 37,
                    }),
                }),
                TransactionNode::TransactionEntry(TransactionEntry {
                    name: "assets:checking".to_string(),
                    value: "$1000".to_string(),
                    value_units_len: 5,
                    value_decimal_len: 0,
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " assets, liabilities, equity, revenues, expenses are common."
                            .to_string(),
                        lineno: 3,
                        colno: 37,
                    }),
                }),
                TransactionNode::TransactionEntry(TransactionEntry {
                    name: "liabilities:credit card".to_string(),
                    value: "$-500".to_string(),
                    value_units_len: 5,
                    value_decimal_len: 0,
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " liabilities, equity, revenues balances are usually negative."
                            .to_string(),
                        lineno: 4,
                        colno: 37,
                    }),
                }),
                TransactionNode::TransactionEntry(TransactionEntry {
                    name: "equity:start".to_string(),
                    value: "".to_string(),
                    value_units_len: 0,
                    value_decimal_len: 0,
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " One amount can be left blank. $-10500 is inferred here."
                            .to_string(),
                        lineno: 5,
                        colno: 37,
                    }),
                }),
                TransactionNode::SingleLineComment(SingleLineComment {
                    content: " Some of these accounts we didn't declare above,".to_string(),
                    prefix: CommentPrefix::Semicolon,
                    lineno: 6,
                    colno: 37,
                }),
                TransactionNode::SingleLineComment(SingleLineComment {
                    content: " so -s/--strict would complain.".to_string(),
                    prefix: CommentPrefix::Semicolon,
                    lineno: 7,
                    colno: 37,
                }),
            ],
            first_entry_indent: 4,
            max_entry_name_len: 23,
            max_entry_value_len: 6,
            max_entry_units_len: 6,
            max_entry_decimal_len: 0,
            max_entry_after_decimal_len: 0,
        }],
    )
}

#[test]
fn assert_balance_transaction() {
    assert_journal(
        r#"2024-01-15 assert some account balances on this date
    assets:savings                    $0                   = $10000
"#,
        vec![JournalCstNode::Transaction {
            title: "2024-01-15 assert some account balances on this date".to_string(),
            title_comment: None,
            entries: vec![TransactionNode::TransactionEntry(
                TransactionEntry {
                    name: "assets:savings".to_string(),
                    value: "$0                   = $10000".to_string(),
                    value_units_len: 2,
                    value_decimal_len: 0,
                    comment: None,
                },
            )],
            first_entry_indent: 4,
            max_entry_name_len: 14,
            max_entry_value_len: 29,
            max_entry_units_len: 2,
            max_entry_decimal_len: 0,
            max_entry_after_decimal_len: 27,
        }],
    )
}
*/