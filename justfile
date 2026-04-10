all: check docs lint test deny semver

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

deny:
    cargo deny check

semver:
    cargo semver-checks
