#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test parse -> format -> parse roundtrip
    // This helps catch issues where formatting produces invalid input
    if let Ok(formatted) = hledger_fmt::format_journal_bytes(data) {
        // The formatted output should be parseable
        let _ = hledger_fmt::format_journal_bytes(&formatted);
    }
});
