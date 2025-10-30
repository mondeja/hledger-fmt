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
    let parsed = parse_content(content.as_bytes()).unwrap();
    let buffer = format_content(&parsed);
    let formatted = String::from_utf8_lossy(&buffer).to_string();
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
fn full_transaction_1() {
    assert_format(
        r#"
2024-01-01 opening balances         ; At the start, declare pre-existing balances this way.
    assets:savings          $10000  ; Account names can be anything. lower case is easy to type.
    assets:checking          $1000  ; assets, liabilities, equity, revenues, expenses are common.
    liabilities:credit card  $-500  ; liabilities, equity, revenues balances are usually negative.
    equity:start                    ; One amount can be left blank. $-10500 is inferred here.
                                    ; Some of these accounts we didn't declare above,
                                    ; so -s/--strict would complain.
"#,
        r#"
2024-01-01 opening balances  ; At the start, declare pre-existing balances this way.
    assets:savings           $10000  ; Account names can be anything. lower case is easy to type.
    assets:checking           $1000  ; assets, liabilities, equity, revenues, expenses are common.
    liabilities:credit card   $-500  ; liabilities, equity, revenues balances are usually negative.
    equity:start             ; One amount can be left blank. $-10500 is inferred here.
    ; Some of these accounts we didn't declare above,
    ; so -s/--strict would complain.
"#,
    );
}

#[test]
fn full_transaction_2() {
    assert_format(
        r#"2024-01-03 ! (12345) pay rent
    ; Additional transaction comment lines, indented.
    ; There can be a ! or * after the date meaning "pending" or "cleared".
    ; There can be a parenthesised (code) after the date/status.
                                    ; Amounts' sign shows direction of flow.
    assets:checking          $-500  ; Minus means removed from this account (credit).
    expenses:rent             $500  ; Plus means added to this account (debit).
"#,
        r#"2024-01-03 ! (12345) pay rent
    ; Additional transaction comment lines, indented.
    ; There can be a ! or * after the date meaning "pending" or "cleared".
    ; There can be a parenthesised (code) after the date/status.
    ; Amounts' sign shows direction of flow.
    assets:checking  $-500     ; Minus means removed from this account (credit).
    expenses:rent     $500     ; Plus means added to this account (debit).
"#,
    )
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
    assets:investments:2024-01-15       2.0 AAAA  @   $1.50  ; @  means per-unit cost
    assets:investments:2024-01-15-02    3.0 AAAA  @@  $4     ; @@ means total cost
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
fn balance_transaction() {
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
    assets:investments:2024-01-15      0.0 AAAA             =       2.0 AAAA             @   $1.50
    assets:investments:2024-01-15-02   0.0 AAAA             =       3.0 AAAA             @@  $4
    liabilities:credit card           $0                    =   $-500
"#,
    );
}

#[test]
fn complex_transaction() {
    assert_format(
        r#"
2024-01-15 hello  ; a comment
    assets:checking   10000,00€  @ 32543.000345€  ==*  $56424324€  ; posting
    assets:checking  10000€  @ 32543.000345€  ==*  $56424324€  ; comments
    expenses:food      $10.010000 @@  $33.3  = 56€   ; must be
    foo  50000000.0000000000€   @@ 65579€  == $78.7   ; aligned
"#,
        r#"
2024-01-15 hello  ; a comment
    assets:checking     10000,00€          @   32543.000345€  ==*  $56424324€   ; posting
    assets:checking     10000€             @   32543.000345€  ==*  $56424324€   ; comments
    expenses:food         $10.010000       @@    $33.3        =           56€   ; must be
    foo              50000000.0000000000€  @@  65579€         ==         $78.7  ; aligned
"#,
    );
}

// https://github.com/mondeja/hledger-fmt/issues/13
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
    assets:investments:2024-01-15     0.0 AAAA  =  2.0 AAAA  @   $1.50
    assets:investments:2024-01-15-02  0.0 AAAA  =  3.0 AAAA  @@  $4
"#,
    );
}

// https://github.com/mondeja/hledger-fmt/issues/25
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

// https://github.com/mondeja/hledger-fmt/issues/27
#[test]
fn issue_27() {
    assert_format(
        r#"2024-01-02 exchange imaginary currency
    income:cash    EUR -100 @@ USDT 120
    assets:cash    USDT 120
"#,
        r#"2024-01-02 exchange imaginary currency
    income:cash  EUR -100  @@  USDT 120
    assets:cash  USDT 120
"#,
    )
}

// https://github.com/mondeja/hledger-fmt/issues/32
#[test]
fn unicode_in_entry_name() {
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
"#,
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
"#,
    );

    assert_format(
        r#"2025-10-10        Description after multiple spaces ; comment
    assets:A   10 EUR
    assets:B  -10 EUR
"#,
        r#"2025-10-10        Description after multiple spaces  ; comment
    assets:A   10 EUR
    assets:B  -10 EUR
"#,
    );
}

// https://github.com/mondeja/hledger-fmt/issues/32
#[test]
fn transaction_with_multiple_currency_formatting() {
    assert_noop_format(
        r#"2025-01-01 Example transaction
    assets:acc1  £10,000.00
    assets:acc2    1.000,00€
    assets:acc3    £1000,00
    assets:acc3    £1000€
    equity
"#,
    )
}

// https://github.com/mondeja/hledger-fmt/issues/32
#[test]
fn space_as_thousands_separator() {
    assert_noop_format(
        r#"2025-09-21 * Example transaction
    a.bankaccount       -2 049,44
    e.expanse              116,99
    e.someotherexpense  $1 018,99
    e.anotherexpense         1,99
    e.thirdexpense
"#,
    )
}

// Real-world corpus file tests
// These tests verify that real-world journal files can be parsed and formatted correctly

#[test]
fn corpus_multicurrency_example() {
    // Test basic multicurrency transaction formatting
    assert_format(
        r#"2015-01-01 * Opening state 1
    Equity:Opening Balances                     -100.00 HRK
    Assets:Cash                                  100.00 HRK

2015-01-03 * Money exchange office
    Assets:Cash                                  -20 EUR @ 7.53 HRK
    Assets:Cash                                  150.60 HRK
"#,
        r#"2015-01-01 * Opening state 1
    Equity:Opening Balances  -100.00 HRK
    Assets:Cash               100.00 HRK

2015-01-03 * Money exchange office
    Assets:Cash  -20 EUR     @  7.53 HRK
    Assets:Cash  150.60 HRK
"#,
    );
}

#[test]
fn corpus_balance_assertions() {
    // Test balance assertions with multiple currencies
    assert_format(
        r#"2016-01-01 opening balances
    assets:Lloyds:current                   £650.00 = £650.00
    assets:Lloyds:savings                      £500 = £500
    assets:house                           £1000.00 = £1000.00
    equity:opening/closing balances
"#,
        r#"2016-01-01 opening balances
    assets:Lloyds:current             £650.00  =   £650.00
    assets:Lloyds:savings             £500     =   £500
    assets:house                     £1000.00  =  £1000.00
    equity:opening/closing balances
"#,
    );
}

#[test]
fn corpus_transaction_codes() {
    // Test transactions with codes in parentheses
    assert_format(
        r#"2016-03-30 (BGC) EMPLOYER INC
    assets:Lloyds:current         £664.72 = £1314.72
    income:employer

2016-04-02 (FOREIGN CCY) HLEDGER
    assets:Lloyds:current             £-6 = £1208.72
    expenses:donations        $7.68 @@ £6
"#,
        r#"2016-03-30 (BGC) EMPLOYER INC
    assets:Lloyds:current  £664.72  =  £1314.72
    income:employer

2016-04-02 (FOREIGN CCY) HLEDGER
    assets:Lloyds:current  £-6     =   £1208.72
    expenses:donations      $7.68  @@     £6
"#,
    );
}

#[test]
fn corpus_stock_trading() {
    // Test stock trading with lot prices
    assert_format(
        r#"2023-01-05 Buy AAPL
    Asset:Stocks                                  5 AAPL @ 160 USD
    Asset:Bank

2023-01-15 Sold Y.AAPL  ; cost_method:fifo
    ; commodity:Y.AAPL, qtty:3.00, price:163.00
    Asset:Bank                           489.00 USD
    Asset:Stocks            -3.0 AAPL @ 160 USD  ; buy_date:2023-01-05, base_cur:USD
    Revenue:Capital Gain                  -9.00 USD
"#,
        r#"2023-01-05 Buy AAPL
    Asset:Stocks  5 AAPL  @  160 USD
    Asset:Bank

2023-01-15 Sold Y.AAPL  ; cost_method:fifo
    ; commodity:Y.AAPL, qtty:3.00, price:163.00
    Asset:Bank            489.00 USD
    Asset:Stocks           -3.0 AAPL  @  160 USD  ; buy_date:2023-01-05, base_cur:USD
    Revenue:Capital Gain   -9.00 USD
"#,
    );
}

#[test]
fn corpus_price_directives() {
    // Test market price directives
    assert_noop_format(
        r#"P 2023-01-06 "AAPL" 129.42239379882812 USD
P 2023-01-09 "AAPL" 129.9515838623047 USD
P 2023-01-10 "AAPL" 130.53070068359375 USD
"#,
    );
}

#[test]
fn corpus_commodity_with_format() {
    // Test commodity declarations with format subdirectives
    assert_noop_format(
        r#"commodity ILS
  format ILS 9,999,999.00

commodity USD
  format USD 9,999,999.00
"#,
    );
}

#[test]
fn corpus_virtual_postings() {
    // Test virtual postings (parenthesized account names)
    assert_format(
        r#"2019/01/01 set initial assets balance
    (assets:banks:israel:boi:ils)    ILS 10,000.00

2016-12-31 pension valuation
    assets:pension:aviva                   = £308.27
    virtual:unrealized pnl
"#,
        r#"2019/01/01 set initial assets balance
    (assets:banks:israel:boi:ils)  ILS 10,000.00

2016-12-31 pension valuation
    assets:pension:aviva    = £308.27
    virtual:unrealized pnl
"#,
    );
}

#[test]
fn corpus_include_directive() {
    // Test include directives
    assert_noop_format(
        r#"; journal created 2021-09-22 by hledger

include ~/.task/hooks/task-timelog-hook/tw.timeclock

D 1.00 h
include ~/.task/hooks/task-timelog-hook/tw.timedot
"#,
    );
}

#[test]
fn corpus_custom_directives() {
    // Test custom directives (hledger-lots style)
    assert_noop_format(
        r#"#+hledger-lots avg_cost:false, check:true
#+hledger-lots no_desc:

#+args buy_aapl:bal desc:"Buy AAPL"
#+args aapl_cur:bal desc:"Buy AAPL" cur:{commodity}
"#,
    );
}
