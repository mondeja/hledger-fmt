use crate::{formatter::format_content, parser::parse_content};
use similar::{ChangeTag, TextDiff};

/// Add a newline to the end of the string if it doesn't have one.
///
/// The formatter adds a newline at the end if the provided content doesn't have one.
fn with_ending_newline(s: &str) -> String {
    if s.ends_with('\n') {
        s.to_string()
    } else {
        format!("{}\n", s)
    }
}

fn assert_raw_format(content: &str, expected: &str) {
    let parsed = parse_content(content).unwrap();
    let formatted = format_content(&parsed);
    assert_eq!(formatted, expected, "{}", {
        let expected_as_string = expected.to_string();
        let diff = TextDiff::from_lines(&expected_as_string, &formatted);

        let mut lines = String::from("\n");
        for change in diff.iter_all_changes() {
            let line = match change.tag() {
                ChangeTag::Delete => format!("- {}", change),
                ChangeTag::Insert => format!("+ {}", change),
                ChangeTag::Equal => format!("  {}", change),
            };
            lines.push_str(&line);
        }

        lines
    });
}

fn assert_format(content: &str, expected: &str) {
    assert_raw_format(content, &with_ending_newline(expected));
}

fn assert_noop_format(content: &str) {
    assert_format(content, &with_ending_newline(content));
}

#[test]
fn empty() {
    // empty content does not add a newline
    assert_raw_format("", "");
}

#[test]
fn single_line_comment_hash() {
    assert_noop_format("# comment");
}

#[test]
fn single_line_comment_semicolon() {
    assert_noop_format("; comment");
}

#[test]
fn indented_single_line_comment() {
    // must not be dedented
    assert_noop_format("  ; comment");
}

#[test]
fn commodities() {
    assert_noop_format(
        r#"
; Declare commodities/currencies and their decimal mark, digit grouping,
; number of decimal places..
commodity 10.000000000€  ; Euro
commodity 10.000000000$  ; Dollar
commodity 80.00Kg        ; weight in Kg
"#,
    );
}

#[test]
fn multiline_comment() {
    assert_noop_format("comment\nfoo\nbar\nend comment");
}

#[test]
fn empty_newlines_are_preserved() {
    assert_noop_format("\n\n\n\n");
}

#[test]
fn single_directive_with_comment() {
    // move comment to two spaces of separation
    assert_format("decimal-mark .      ; comment", "decimal-mark .  ; comment");
}

#[test]
fn directives_group_with_comments() {
    // align comments in group of directives
    assert_format(
        "account cash:foo:bar:baz     ; comment\ntag foo  ; comment",
        "account cash:foo:bar:baz  ; comment\ntag foo                   ; comment",
    );
}

#[test]
fn directive_group_with_trailing_comments() {
    // comments that are in the newline just after a directive group
    // are aligned with the directive group comments
    assert_format(
        r#"account cash:foo:bar:baz     ; comment
tag foo  ; comment
; trailing comment"#,
        r#"account cash:foo:bar:baz  ; comment
tag foo                   ; comment
                          ; trailing comment"#,
    );
}

#[test]
fn one_directive_compounds_a_directives_group() {
    // just one directive creates a directive group
    assert_format(
        r#"
tag foo      ; comment
; trailing comment"#,
        r#"
tag foo  ; comment
         ; trailing comment"#,
    );
}

#[test]
fn comment_before_directives_group() {
    // comments in lines just before directive groups are not aligned
    assert_format(
        r#"; comment
tag foo      ; comment
; trailing comment"#,
        r#"; comment
tag foo  ; comment
         ; trailing comment"#,
    );
}

#[test]
fn transaction_comment_with_title_comment() {
    // transaction titles and entry comments are aligned
    assert_format(
        r#"
2021-01-01 title  ; comment
  account  10  ; entry comment"#,
        r#"
2021-01-01 title  ; comment
  account  10     ; entry comment
"#,
    )
}

#[test]
fn transaction_comments() {
    // even when there is no title, entry comments are aligned
    // with a possible title future comment
    assert_format(
        r#"
2023-05-25 trip to the supermarket
    expenses             $10.06   ; hello
    assets              $-1  ; hello
    assets              $-1000
"#,
        r#"
2023-05-25 trip to the supermarket
    expenses     $10.06             ; hello
    assets       $-1                ; hello
    assets    $-1000
"#,
    );
}

#[test]
fn transaction_comments_alignments() {
    // comments are aligned with the longest entry
    //
    // TODO: this extreme case is not handled by the current implementation
    assert_format(
        r#"
2023-05-25 trip to the supermarket  ; foo bar
    expenses             $10.06   ; hello
    assets              $-1  ; hello
    assets              $-100000000000,000000000    ; hello
"#,
        r#"
2023-05-25 trip to the supermarket  ; foo bar
    expenses             $10.06     ; hello
    assets               $-1        ; hello
    assets    $-100000000000,000000000  ; hello
"#,
    );
}

#[test]
fn transaction_single_line_comments() {
    // single line comments inside translations are aligned with the indentation
    assert_format(
        r#"
2021-01-01 title  ; comment
      ; Explain this translation
  account  10  ; entry comment
    ; Another comment"#,
        r#"
2021-01-01 title  ; comment
  ; Explain this translation
  account  10     ; entry comment
  ; Another comment
"#,
    );
}

#[test]
fn comment_before_transaction() {
    // comments before transactions are not aligned
    assert_format(
        r#"
; comment
2021-01-01 title  ; comment
  account  10  ; entry comment"#,
        r#"
; comment
2021-01-01 title  ; comment
  account  10     ; entry comment
"#,
    );
}

#[test]
fn periodic_transaction() {
    assert_format(
        r#"
~ monthly  set budget goals  ; <- Note, 2 spaces before the description.
    (expenses:rent)      $1000  ; <- Note, 2+ spaces before comment.
    (expenses:food)       $500
"#,
        r#"
~ monthly  set budget goals  ; <- Note, 2 spaces before the description.
    (expenses:rent)  $1000   ; <- Note, 2+ spaces before comment.
    (expenses:food)   $500
"#,
    );
}

#[test]
fn auto_posting_rule() {
    assert_format(
        r#"
= revenues:consulting
    liabilities:tax:2024:us          *0.25  ; Add a tax liability & expense
    expenses:tax:2024:us            *-0.25  ; for 25% of the revenue.
"#,
        r#"
= revenues:consulting
    liabilities:tax:2024:us   *0.25  ; Add a tax liability & expense
    expenses:tax:2024:us     *-0.25  ; for 25% of the revenue.
"#,
    );
}

#[test]
fn two_full_transactions() {
    assert_format(
        r#"
2024-01-01 opening balances         ; At the start, declare pre-existing balances this way.
    assets:savings          $10000  ; Account names can be anything. lower case is easy to type.
    assets:checking          $1000  ; assets, liabilities, equity, revenues, expenses are common.
    liabilities:credit card  $-500  ; liabilities, equity, revenues balances are usually negative.
    equity:start                    ; One amount can be left blank. $-10500 is inferred here.
                                    ; Some of these accounts we didn't declare above,
                                    ; so -s/--strict would complain.

2024-01-03 ! (12345) pay rent
    ; Additional transaction comment lines, indented.
    ; There can be a ! or * after the date meaning "pending" or "cleared".
    ; There can be a parenthesised (code) after the date/status.
                                    ; Amounts' sign shows direction of flow.
    assets:checking          $-500  ; Minus means removed from this account (credit).
    expenses:rent             $500  ; Plus means added to this account (debit).
"#,
        r#"
2024-01-01 opening balances  ; At the start, declare pre-existing balances this way.
    assets:savings            $10000  ; Account names can be anything. lower case is easy to type.
    assets:checking            $1000  ; assets, liabilities, equity, revenues, expenses are common.
    liabilities:credit card    $-500  ; liabilities, equity, revenues balances are usually negative.
    equity:start             ; One amount can be left blank. $-10500 is inferred here.
    ; Some of these accounts we didn't declare above,
    ; so -s/--strict would complain.

2024-01-03 ! (12345) pay rent
    ; Additional transaction comment lines, indented.
    ; There can be a ! or * after the date meaning "pending" or "cleared".
    ; There can be a parenthesised (code) after the date/status.
    ; Amounts' sign shows direction of flow.
    assets:checking  $-500     ; Minus means removed from this account (credit).
    expenses:rent     $500     ; Plus means added to this account (debit).
"#,
    );
}

#[test]
fn transaction_with_payee_note() {
    assert_format(
        r#"
2024-01-02 Gringott's Bank | withdrawal  ; Description can be PAYEE | NOTE
    assets:bank:gold       -10 gold  ; foo
    assets:pouch            10 gold  ; bar
"#,
        r#"
2024-01-02 Gringott's Bank | withdrawal  ; Description can be PAYEE | NOTE
    assets:bank:gold  -10 gold           ; foo
    assets:pouch       10 gold           ; bar
"#,
    );
}

#[test]
fn transaction_with_shares() {
    assert_format(
        r#"
2024-01-15 buy some shares, in two lots                 ; Cost can be noted.
    assets:investments:2024-01-15     2.0 AAAA @ $1.50  ; @  means per-unit cost
    assets:investments:2024-01-15-02  3.0 AAAA @@ $4    ; @@ means total cost
                      ; ^ Per-lot subaccounts are sometimes useful.
    assets:checking                 $-7
"#,
        r#"
2024-01-15 buy some shares, in two lots  ; Cost can be noted.
    assets:investments:2024-01-15       2.0 AAAA   @    $1.50  ; @  means per-unit cost
    assets:investments:2024-01-15-02    3.0 AAAA   @@   $4     ; @@ means total cost
    ; ^ Per-lot subaccounts are sometimes useful.
    assets:checking                   $-7
"#,
    );
}

#[test]
fn transaction_without_postings() {
    assert_format(
        r#"
2024-02-01 note some event, or a transaction not yet fully entered, on this date
    ; Postings are not required.
; next comments are aligned with the first one
  ; foo
      ; bar
"#,
        r#"
2024-02-01 note some event, or a transaction not yet fully entered, on this date
    ; Postings are not required.
    ; next comments are aligned with the first one
    ; foo
    ; bar
"#,
    );
}

#[test]
fn date_formats_empty_transactions() {
    // example from the cheatsheet, transactions must be separated by a newline
    assert_format(
        r#"
; Some other date formats are allowed (but, consistent YYYY-MM-DD is useful).
2024.01.01
2024/1/1
2024-1-1
2024-01-1
2024-1-01
"#,
        r#"
; Some other date formats are allowed (but, consistent YYYY-MM-DD is useful).
2024.01.01

2024/1/1

2024-1-1

2024-01-1

2024-1-01"#,
    );
}

#[test]
fn separate_transactions() {
    // transactions must be separated by a newline
    assert_format(
        r#"
2015-10-16 bought food
  expenses:food  $10
  assets:cash
2015-10-17 bought tool
  expenses:food  $10
  assets:cash
"#,
        r#"
2015-10-16 bought food
  expenses:food  $10
  assets:cash

2015-10-17 bought tool
  expenses:food  $10
  assets:cash
"#,
    );
}

#[test]
fn assert_balance_transaction() {
    // assert balance transactions are formatted aligning the equal sign
    assert_format(
        r#"
2024-01-15 assert some account balances on this date
    ; Balances can be asserted in any transaction, with =, for extra error checking.
    ; Assertion txns like this one can be made with hledger close --assert --show-costs
    ;
    assets:savings                    $0                   = $10000
    assets:checking                   $0                   =   $493
    assets:bank:gold                   0 gold              =    -10 gold
    assets:pouch                       0 gold              =      4 gold
    assets:pouch                       0 "Chocolate Frogs" =      3 "Chocolate Frogs"
    assets:investments:2024-01-15      0.0 AAAA            =      2.0 AAAA @  $1.50
    assets:investments:2024-01-15-02   0.0 AAAA            =      3.0 AAAA @@ $4
    liabilities:credit card           $0                   =  $-500
"#,
        r#"
2024-01-15 assert some account balances on this date
    ; Balances can be asserted in any transaction, with =, for extra error checking.
    ; Assertion txns like this one can be made with hledger close --assert --show-costs
    ;
    assets:savings                    $0                    =  $10000
    assets:checking                   $0                    =    $493
    assets:bank:gold                   0 gold               =     -10 gold
    assets:pouch                       0 gold               =       4 gold
    assets:pouch                       0 "Chocolate Frogs"  =       3 "Chocolate Frogs"
    assets:investments:2024-01-15      0.0 AAAA             =       2.0 AAAA  @   $1.50
    assets:investments:2024-01-15-02   0.0 AAAA             =       3.0 AAAA  @@  $4
    liabilities:credit card           $0                    =   $-500
"#,
    );
}

#[test]
fn complex_transaction() {
    assert_format(
        r#"
2024-01-15 hello  ; a comment
    assets:checking  10000€  @ 32543.000345€  ==*  $56424324€  ; comments
    expenses:food      $10.010000 @@  $33.3  = 56€   ; must be
    foo  50000000.0000000000€   @@ 65579€  == $78.7   ; aligned
"#,
        r#"
2024-01-15 hello  ; a comment
    assets:checking      10000€              @    32543.000345€  ==*  $56424324€   ; comments
    expenses:food          $10.010000        @@     $33.3        =           56€   ; must be
    foo               50000000.0000000000€   @@   65579€         ==         $78.7  ; aligned
"#,
    );
}

#[test]
fn issue_13() {
    assert_format(
        r#"
2022-01-01 SHELL OIL
    asset:checking           $-8.42 = $11373.17
    expense:transport:gas           $8.42
"#,
        r#"
2022-01-01 SHELL OIL
    asset:checking         $-8.42  =  $11373.17
    expense:transport:gas   $8.42
"#,
    );
}

#[test]
fn lots() {
    assert_format(
        r#"2024-01-15 foobar
    assets:investments:2024-01-15      0.0 AAAA            =      2.0 AAAA @  $1.50
    assets:investments:2024-01-15-02   0.0 AAAA            =      3.0 AAAA @@ $4
"#,
        r#"2024-01-15 foobar
    assets:investments:2024-01-15     0.0 AAAA  =   2.0 AAAA  @   $1.50
    assets:investments:2024-01-15-02  0.0 AAAA  =   3.0 AAAA  @@  $4
"#,
    );
}

#[test]
fn issue_25() {
    assert_format(
        r#"1/1/1 * transaction
	; vacation  $2350 hawaii flight
"#,
        r#"1/1/1 * transaction
; vacation  $2350 hawaii flight
"#,
    );
}

#[test]
fn issue_27() {
    assert_format(
        r#"2024-01-02 exchange imaginary currency
    income:cash    EUR -100 @@ USDT 120
    assets:cash    USDT 120
"#,
        r#"2024-01-02 exchange imaginary currency
    income:cash  EUR-100   @@   USDT120
    assets:cash  USDT120
"#,
    )
}

#[test]
fn issue_32_unicode() {
    assert_noop_format(
        r#"2025-09-23 * Zakupy
    a.ca      -106,98
    słodycze     6,99  ; one unicode char
    książki     77,77  ; two
    alkohol     22,22  ; none
"#,
    );
}

#[test]
fn subdirective() {
    assert_noop_format(
        r#"
commodity $
  note USD ; US Dollar
"#,
    );
}

#[test]
fn multiple_subdirectives_in_directives_group() {
    // Subdirective comments are not aligned. The rationale is that
    // currently, hledger ignores them.
    assert_format(
        r#"
account assets:bank:checking
  format subdirective  ; subdirective comments are not aligned
    ; this is a comment
tag foo
  hola;
  #another comment
account assets:bank:savings
  format subdirective  ; subdirective comments are not aligned
    ; this is another comment
tag foo
  hola;
"#,
        r#"
account assets:bank:checking
  format subdirective  ; subdirective comments are not aligned
                              ; this is a comment
tag foo
  hola;
                              #another comment
account assets:bank:savings
  format subdirective  ; subdirective comments are not aligned
                              ; this is another comment
tag foo
  hola;
"#,
    );
}

// https://github.com/mondeja/hledger-fmt/issues/40
#[test]
fn transaction_with_multi_spaced_description() {
    assert_noop_format(
        r#"2025-10-10  Description after two spaces
    assets:A   10 EUR
    assets:B  -10 EUR
"#
    );
}

#[test]
fn transaction_with_multi_spaced_description_and_valid_comment() {
    assert_format(
        r#"2025-10-10               Description after multiple spaces; comment
    assets:A   10 EUR
    assets:B  -10 EUR
"#,
        r#"2025-10-10               Description after multiple spaces  ; comment
    assets:A   10 EUR
    assets:B  -10 EUR
"#
    );

    assert_format(
        r#"2025-10-10        Description after multiple spaces ; comment
    assets:A   10 EUR
    assets:B  -10 EUR
"#,
        r#"2025-10-10        Description after multiple spaces  ; comment
    assets:A   10 EUR
    assets:B  -10 EUR
"#
    );
}
