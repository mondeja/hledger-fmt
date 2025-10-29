#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Only fuzz valid UTF-8 strings for the format function
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = hledger_fmt::format_journal(s);
    }
});
