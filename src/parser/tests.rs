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
            content: "content".into(),
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
                    indent: 4,
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
                        indent: 4,
                    }),
                }),
                DirectiveNode::Directive(Directive {
                    name: "account".into(),
                    content: "bar".into(),
                    comment: Some(SingleLineComment {
                        prefix: CommentPrefix::Semicolon,
                        content: " bar comment".into(),
                        indent: 4,
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
                        indent: 1, // TODO: idents for directive comments are not used in the formatter
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

#[test]
fn all_directives() {
    assert_journal(
        r#"account Assets:Bank:Checking
commodity $
decimal-mark ,
payee Foo Bar
tag foo bar
include /path/to/file
P foobarbaz
apply account Expenses:Food
D 2024-01-01
Y 2024
apply fixed
apply tag Important
assert Assets:Bank:Checking >= $0
capture Transactions2024
check Assets:Bank:Checking
define MyDefinition
bucket / A Expenses:Food
end apply fixed
end apply tag
end apply year
end tag
eval foobar
expr foobar
python print('Hello, World!')
value $100.00
--command-line-flags foobar
"#,
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
                DirectiveNode::Directive(Directive {
                    name: "payee".into(),
                    content: "Foo Bar".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "tag".into(),
                    content: "foo bar".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "include".into(),
                    content: "/path/to/file".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "P".into(),
                    content: "foobarbaz".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "apply account".into(),
                    content: "Expenses:Food".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "D".into(),
                    content: "2024-01-01".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "Y".into(),
                    content: "2024".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "apply fixed".into(),
                    content: "".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "apply tag".into(),
                    content: "Important".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "assert".into(),
                    content: "Assets:Bank:Checking >= $0".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "capture".into(),
                    content: "Transactions2024".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "check".into(),
                    content: "Assets:Bank:Checking".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "define".into(),
                    content: "MyDefinition".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "bucket / A".into(),
                    content: "Expenses:Food".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "end apply fixed".into(),
                    content: "".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "end apply tag".into(),
                    content: "".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "end apply year".into(),
                    content: "".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "end tag".into(),
                    content: "".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "eval".into(),
                    content: "foobar".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "expr".into(),
                    content: "foobar".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "python".into(),
                    content: "print('Hello, World!')".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "value".into(),
                    content: "$100.00".into(),
                    comment: None,
                }),
                DirectiveNode::Directive(Directive {
                    name: "--command-line-flags".into(),
                    content: "foobar".into(),
                    comment: None,
                }),
            ],
            max_name_content_len: 32,
        }],
    )
}

#[test]
fn parse_empty() {
    assert_journal("", vec![]);
}

// Regression tests for bugs found via fuzzing
// Each test demonstrates a panic/crash that occurred before the fix

#[test]
fn regression_entry_name_with_closing_paren() {
    // DEMONSTRATES FIX: src/parser/mod.rs:751-753
    // 
    // Bug: slice index panic when entry_name_end < entry_name_start
    // 
    // BEFORE FIX (line 746):
    //   let entry_name = ByteStr::from(&line[entry_name_start..entry_name_end]);
    //
    // Panic occurred: thread 'main' panicked at src/parser/mod.rs:746:41:
    //   slice index starts at 1 but ends at 0
    //
    // AFTER FIX (lines 751-753):
    //   let entry_name_end = entry_name_end.max(entry_name_start);
    //   let entry_name = ByteStr::from(&line[entry_name_start..entry_name_end]);
    //
    // This test would PANIC without the fix and PASSES with the fix.
    
    let content = r#"2015-10-16 bought food
 ) expenses:food        $10
  assets:cash
"#;
    let result = parse_content(content.as_bytes());
    assert!(result.is_ok(), "PANIC before fix: slice index starts at 1 but ends at 0");
}

#[test]
fn regression_single_char_whitespace_line() {
    // DEMONSTRATES FIX: src/parser/mod.rs:255-263
    //
    // Bug: out-of-bounds access with single-character whitespace lines
    //
    // BEFORE FIX (lines 255-258):
    //   let all_whitespace = last_byte.is_ascii_whitespace()
    //       && unsafe { line.get_unchecked(1..line_length - 1) }
    //           .iter()
    //           .all(|&b| b.is_ascii_whitespace());
    //
    // Panic occurred: accessing line[1..0] is invalid for single-char line
    //
    // AFTER FIX (lines 255-263):
    //   let all_whitespace = if line_length < 2 {
    //       true
    //   } else {
    //       last_byte.is_ascii_whitespace()
    //           && unsafe { line.get_unchecked(1..line_length - 1) }
    //               .iter()
    //               .all(|&b| b.is_ascii_whitespace())
    //   };
    //
    // This test would PANIC without the fix and PASSES with the fix.
    
    let content = " \n";
    let result = parse_content(content.as_bytes());
    assert!(result.is_ok(), "PANIC before fix: out-of-bounds access line[1..0]");
    
    let content = "\t\n";
    let result = parse_content(content.as_bytes());
    assert!(result.is_ok(), "PANIC before fix: out-of-bounds access line[1..0]");
}

#[test]
fn regression_commodity_directive_bounds() {
    // DEMONSTRATES FIX: src/parser/mod.rs:1074
    //
    // Bug: off-by-one error accessing index 9 with length check >= 9
    //
    // BEFORE FIX (line 1067):
    //   if line_length >= 9 && *l.get_unchecked(0) == b'c' {
    //       // ... checks indices 0-8 ...
    //       && *l.get_unchecked(9) == b' '  // BUG: accessing index 9!
    //
    // Panic occurred: thread 'main' panicked at src/parser/mod.rs:1084:19:
    //   unsafe precondition(s) violated: slice::get_unchecked requires that 
    //   the index is within the slice
    //
    // AFTER FIX (line 1074):
    //   if line_length >= 10 && *l.get_unchecked(0) == b'c' {
    //
    // This test would PANIC without the fix and PASSES with the fix.
    
    let content = "commodity";  // exactly 9 characters, no space
    let result = parse_content(content.as_bytes());
    assert!(result.is_ok(), "PANIC before fix: unsafe precondition violated at index 9");
    
    let content = "commodity ";  // 10 characters with space - valid directive
    let result = parse_content(content.as_bytes());
    assert!(result.is_ok());
}

#[test]
fn regression_various_edge_cases() {
    // Additional edge cases exercising the fixes above
    
    // Tests entry_name fix with complex spacing
    let content = r#"2015-10-16 test
   )   foo  $10
  assets:cash
"#;
    let result = parse_content(content.as_bytes());
    assert!(result.is_ok());
    
    // Tests tab-indented entries
    let content = "2015-10-16 test\n\tassets:cash  $10\n\texpenses:food\n";
    let result = parse_content(content.as_bytes());
    assert!(result.is_ok());
}
