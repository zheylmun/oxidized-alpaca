# Contributing to oxidized_alpaca

## Pre-commit hooks

The repo ships a `.pre-commit-config.yaml` that runs `cargo fmt`, `cargo clippy`,
and a Conventional Commits check against every commit. The same fmt/clippy
checks are enforced in CI; running them locally just shortens the feedback loop.

### Install the hook runner

Pick whichever runner you prefer — both consume the same config:

- **prek** (recommended, single Rust binary, no Python required):

  ```sh
  uv tool install prek
  # or: cargo install --locked prek
  ```

- **pre-commit** (Python):

  ```sh
  pipx install pre-commit
  # or: uv tool install pre-commit
  ```

### Wire the hooks into the repo

From the repo root:

```sh
prek install --install-hooks       # or: pre-commit install --install-hooks
```

This installs `pre-commit` and `commit-msg` git hooks into `.git/hooks/` and
pre-fetches the hook environments so the first commit isn't slow.

### What runs and when

| Stage      | Hook                       | What it checks                                   |
| ---------- | -------------------------- | ------------------------------------------------ |
| commit-msg | `conventional-pre-commit`  | Commit subject matches Conventional Commits      |
| pre-commit | `cargo-fmt`                | `cargo fmt --all -- --check`                     |
| pre-commit | `cargo-clippy`             | `cargo clippy --all-features --tests --benches -- -Dclippy::all` (mirrors CI) |

To run every hook against the working tree without committing:

```sh
prek run --all-files               # or: pre-commit run --all-files
```

## Commit messages

Commits must follow [Conventional Commits](https://www.conventionalcommits.org/).
The accepted types are the project default: `build`, `chore`, `ci`, `docs`,
`feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`. A trailing `!`
marks a breaking change; an optional `(scope)` is allowed.

Examples:

- `feat: add multi-leg options order entry point`
- `feat(orders)!: replace OrderClass enum with sealed trait`
- `fix: stop dropping symbols from multi-symbol pagination too eagerly`
- `chore: bump dependencies`
- `docs: clarify the numeric-type policy in the README`

The subject line should describe *what changed*; reserve the body for the
*why*. release-plz reads these markers to compute version bumps and to write
the CHANGELOG, so getting the type right matters at release time.
