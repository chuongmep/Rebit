# Contributing to Rebit

## Development prerequisites

1. Rust stable toolchain (rustfmt and clippy enabled).
2. GitHub access with CODEOWNERS review path.
3. Ability to run workspace checks locally.

## Quick start

1. Clone repository.
2. Run: `cargo check --workspace`
3. Run: `cargo test --workspace`
4. Run: `cargo clippy --workspace --all-targets -- -D warnings`
5. Run: `cargo fmt --all -- --check`

## Branch strategy

1. Create a short-lived feature branch from main.
2. Keep PRs focused on one concern.
3. Rebase before requesting final review.

## Required for every PR

1. Linked issue or RFC reference.
2. Tests for changed behavior.
3. API-change notes if public interfaces changed.
4. Performance note for hot-path changes.

## Review rules

1. CODEOWNERS approval is mandatory.
2. Public API changes need one cross-domain reviewer.
3. Architecture-impacting changes require ADR update.

## Commit style

Use conventional style:

- feat: new capability
- fix: behavior correction
- perf: performance optimization
- refactor: code improvements without behavior change
- docs: documentation only
- test: tests only
- ci: pipeline or automation changes

## Definition of done

1. CI checks pass.
2. Tests and docs updated.
3. Observability and rollback considerations documented.
