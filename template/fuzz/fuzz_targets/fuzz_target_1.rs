#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|_data: &[u8]| {
    // TODO: exercise your public API with randomized input.
    // Example:
    //   let _ = my_project_core::some_function(_data);
});
