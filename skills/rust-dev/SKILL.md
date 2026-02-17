---
name: rust-dev
description: Use when building or refactoring Rust code in this repo, including workspace setup, crate boundaries, tests, linting, and performance-safe implementation details.
---

# Rust Dev

## Use This Skill When
- Creating or reorganizing Rust crates/modules.
- Implementing application logic, state management, async workers, or error handling.
- Adding or fixing tests, clippy findings, or formatting issues.

## Required Inputs
- Target files or modules.
- Functional requirement and acceptance criteria.

## Workflow
1. Confirm crate/module boundaries before coding.
2. Implement minimal change for one vertical slice.
3. Add or update unit/integration tests with the change.
4. Run `cargo fmt`, `cargo clippy`, and `cargo test`.
5. Report behavior changes and residual risk.

## Conventions
- Prefer explicit types at subsystem boundaries.
- Use `thiserror` for domain errors and `anyhow` only at app edges.
- Keep UI, app, engine, and catalog modules separated.
- Avoid hidden panics in production paths.

## Output Contract
- Compiling code.
- Tests for changed behavior.
- Short change summary with file references.
