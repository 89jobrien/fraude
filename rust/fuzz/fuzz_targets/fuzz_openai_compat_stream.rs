#![no_main]

use api::openai_compat_fuzz;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Must not panic on any input; errors are acceptable.
    let _ = openai_compat_fuzz::push_chunk(data);
});
