#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test the parser with arbitrary bytes
    let _ = hledger_fmt::format_journal_bytes(data);
});
