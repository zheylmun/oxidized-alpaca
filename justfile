build:
    cargo check

docs:
    cargo test --doc
    cargo doc --no-deps

lint:
    cargo clippy --all-targets --all-features --tests -- -Dclippy::pedantic -Dclippy::all

test:
    cargo test env
    cargo test -- --skip env
    cargo run --example tracing
    grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/cov/tests.lcov
