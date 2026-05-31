verify: lint test fuzz-smoke

lint:
    markdownlint-cli2 "**/*.md"
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic
    cargo deny check
    typos

test:
    cargo test --workspace --all-features

fuzz-smoke:
    cargo +nightly fuzz run parse_context -- -max_total_time=60 -rss_limit_mb=2048
    cargo +nightly fuzz run receiver_range -- -max_total_time=60 -rss_limit_mb=2048
    cargo +nightly fuzz run render_template -- -max_total_time=60 -rss_limit_mb=2048
