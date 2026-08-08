# Reproducibility Matrix (Phase 0)

This document tracks deterministic validation evidence for RELI Phase 0.

## Required Commands

1. cargo build --locked
2. cargo test --locked
3. cargo run --locked --bin validate_phase0

## Environment Matrix

- ubuntu-latest (GitHub Actions)
- macos-latest (GitHub Actions)

## Local Baseline (2026-08-07)

- Host OS: Linux
- Command status:
  - cargo build --locked: pass
  - cargo test --locked: pass
  - cargo run --locked --bin validate_phase0: pass

## CI Evidence Collection

CI workflow:

- .github/workflows/ci.yml

Expected acceptance rule:

- Phase 0 reproducibility requirement is satisfied when both matrix environments pass all required commands.

## Notes

- Any reproducibility failure must include:
  - full command output
  - rustc and cargo versions
  - schema and fixture revision identifiers
