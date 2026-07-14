#![no_main]

use api::SseParser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut parser = SseParser::new();
    // Must not panic on any input; errors are acceptable.
    let _ = parser.push(data);
    let _ = parser.finish();
});
