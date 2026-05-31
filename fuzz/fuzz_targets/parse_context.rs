#![no_main]
use libfuzzer_sys::fuzz_target;
use snippercontext::Backend as _;

#[derive(Debug, arbitrary::Arbitrary)]
struct Input {
    source: String,
    offset: usize,
}

fuzz_target!(|input: Input| {
    let backend = snippercontext::TreeSitterBackend::new();
    let _ = backend.classify(&input.source, input.offset);
});
