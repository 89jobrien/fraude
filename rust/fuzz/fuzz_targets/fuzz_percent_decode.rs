#![no_main]

use libfuzzer_sys::fuzz_target;
use runtime::oauth_fuzz;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Must not panic on any input; Err is acceptable.
        let _ = oauth_fuzz::percent_decode(s);

        // Round-trip invariant: encode then decode must equal original.
        let encoded = oauth_fuzz::percent_encode(s);
        let decoded = oauth_fuzz::percent_decode(&encoded);
        assert_eq!(decoded, Ok(s.to_string()));
    }
});
