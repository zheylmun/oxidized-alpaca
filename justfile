all: check docs lint test

check:
    cargo check

docs:
    cargo test --doc
    cargo doc --no-deps

lint:
    cargo clippy --all-targets --all-features --tests -- -Dclippy::all

test:
    cargo test env
    cargo test -- --skip env
