# AGENTS.md

## Architecture Memory (Clean Architecture)

This repository follows a strict Clean Architecture workspace layout:

- `/lite-room/crates/domain`
  - Pure business/domain rules only.
  - No IO/framework/database/UI dependencies.
- `/lite-room/crates/application`
  - Use-cases and ports (traits/interfaces).
  - Depends only on `domain`.
- `/lite-room/crates/adapters`
  - Implementations of application ports (sqlite/fs/image/presenters).
  - Depends on `application` and `domain`.
- `/lite-room/crates/drivers`
  - CLI/UI/runtime wiring and entrypoints.
  - Depends on `adapters`, `application`, and `domain` for DTO/read models.

## Dependency Rule (must never be violated)

Dependencies must point inward only:

`drivers -> adapters -> application -> domain`

Forbidden examples:

- `domain` importing anything from `application`, `adapters`, or `drivers`
- `application` importing from `adapters` or `drivers`
- `adapters` importing from `drivers`

## Change Policy

When implementing new behavior:

1. Add/adjust entities/value objects/errors in `domain`.
2. Add/adjust use-cases and port traits in `application`.
3. Implement ports in `adapters`.
4. Wire commands/UI/runtime flow in `drivers`.

Avoid placing business logic directly in `adapters` or `drivers`.

## Validation Commands

Run these after architectural changes:

- `cargo check -p lite-room-domain`
- `cargo check -p lite-room-application`
- `cargo check -p lite-room-adapters`
- `cargo check -p lite-room-drivers`
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`
