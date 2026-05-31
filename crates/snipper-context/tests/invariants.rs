#![cfg(feature = "lang-csharp")]
use proptest::prelude::*;
use snipper_context::{Backend as _, TreeSitterBackend};

proptest! {
    // INV-4: classify is pure — same inputs always produce the same output.
    #[test]
    fn inv4_classify_is_deterministic(
        source in ".{0,64}",
        offset in 0usize..=256usize,
    ) {
        let backend = TreeSitterBackend::csharp();
        let r1 = backend.classify(&source, offset);
        let r2 = backend.classify(&source, offset);
        prop_assert_eq!(r1.ok(), r2.ok());
    }
}
