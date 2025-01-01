all: build docs lint test

build:
    cargo check

docs:
    cargo test --doc
    cargo doc --no-deps

lint:
    cargo clippy --all-targets --all-features --tests -- -Dclippy::all

test:
    cargo test env
    cargo test -- --skip env
