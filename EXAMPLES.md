# Formatting guide

<!-- markdownlint-disable no-inline-html line-length -->

hledger-fmt tries to format files in a way that make them look consistent and tidy.
This document contains commented examples about how hledger-fmt formats files.

## Directives

- Commodities are grouped if they are not separated by blank lines.
- Comments at the end of commodity declaration lines are aligned,
  adjusting to the minimum of 2 spaces of separation from the longest value.
- Comments at the end of directives group are aligned too.

```hledger
; Declare commodities/currencies and their decimal mark, digit grouping,
; number of decimal places...
commodity 10.000000000€  ; Euro
commodity 10.000000000$  ; Dollar
commodity 80.00Kg        ; weight in Kg
                         ; Comments at the end are aligned too.
```

## Transactions

- The indentation of transaction entries is defined by the indentation of the first entry.
- Entry values in transactions are aligned to the decimal point.
- The minimum space between entry values can be defined with the environment variable
  `HLEDGER_FMT_ENTRY_SPACING` (default: 2), see [Configuration].
- Entry comments are aligned to the longest value, with at least 2 spaces of separation.
- When possible, transaction title and entry comments are aligned. When not, the title
  comment is indented with 2 spaces of separation and entry comments are aligned in a group.
- Transactions are separated by blank lines.

[Configuration]: https://github.com/mondeja/hledger-fmt#configuration

```hledger
             ; The original indent of comments before transactions is preserved.
2023-05-25 trip to the supermarket
  expenses     $10.06               ; Entry comments are aligned with a future title's comment.
  assets       $-1                  ; 2 spaces is the minimum possible separation for comments.
  ; Single line comments are aligned with the indentation of entries.
  assets    $-1000
  ; Comments at the end of a transaction entry are aligned too.

; Different transactions can ident entries differently:
= revenues:consulting  ; The title comment is indented with 2 spaces and not aligned with entries.
    liabilities:tax:2024:us   *0.25  ; Add a tax liability & expense for 25% of the revenue.
    expenses:tax:2024:us     *-0.25

2024-01-15 buy some shares, in two lots  ; Cost
    assets:investments:2024-01-15       2.0 AAAA  @   $1.50  ; @  means per-unit cost
    assets:investments:2024-01-15-02    3.0 AAAA  @@  $4     ; @@ means total cost
    ; Two spaces is the default spacing between  ^   ^    entry values.
    assets:checking                   $-7

; With HLEDGER_FMT_ENTRY_SPACING=5
2024-01-15 buy some shares, in two lots  ; Cost
    assets:investments:2024-01-15       2.0 AAAA     @      $1.50  ; @  means per-unit cost
    assets:investments:2024-01-15-02    3.0 AAAA     @@     $4     ; @@ means total cost
    ; Five spaces as custom spacing between       ^      ^      entry values.
    assets:checking                   $-7
```

### More examples

```hledger
2024-01-15 assert some account balances on this date
    assets:savings                    $0                    =  $10000
    assets:checking                   $0                    =    $493
    assets:bank:gold                   0 gold               =     -10 gold
    assets:pouch                       0 gold               =       4 gold
    assets:pouch                       0 "Chocolate Frogs"  =       3 "Chocolate Frogs"
    assets:investments:2024-01-15      0.0 AAAA             =       2.0 AAAA             @   $1.50
    assets:investments:2024-01-15-02   0.0 AAAA             =       3.0 AAAA             @@  $4
    liabilities:credit card           $0                    =   $-500

2024-01-15 hello  ; a comment
    assets:checking     10000,00€          @   32543.000345€  ==*  $56424324€   ; posting
    assets:checking     10000€             @   32543.000345€  ==*  $56424324€   ; comments
    expenses:food         $10.010000       @@    $33.3        =           56€   ; are
    foo              50000000.0000000000€  @@  65579€         ==         $78.7  ; aligned

2016-01-01 opening balances
    assets:Lloyds:current             £650.00  =   £650.00
    assets:Lloyds:savings             £500     =   £500
    assets:house                     £1000.00  =  £1000.00
    equity:opening/closing balances

2025-09-21 * Example transaction
    a.bankaccount       -2 049,44
    e.expanse              116,99
    e.someotherexpense  $1 018,99
    e.anotherexpense         1,99
    e.thirdexpense
```
