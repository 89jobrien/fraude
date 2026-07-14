#![no_main]

use libfuzzer_sys::fuzz_target;
use lsp::read_message_fuzz;

fuzz_target!(|data: &[u8]| {
    // Must not panic on any input; errors are acceptable.
    let _ = read_message_fuzz::read_message_sync(data);
});
