all: install-tools check docs lint test deny semver

install-tools:
    sh -c 'command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny --locked'
    sh -c 'command -v cargo-semver-checks >/dev/null 2>&1 || cargo install cargo-semver-checks --locked'
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
