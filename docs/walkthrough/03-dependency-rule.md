# Dependency Rule in Code

The workspace enforces inward-only dependencies:

`drivers -> adapters -> application -> domain`

## `domain`
Path:
- [/lite-room/crates/domain/src](../../crates/domain/src)

Contains only core business types and invariants.
No DB, filesystem, UI, or runtime framework logic.

## `application`
Paths:
- [/lite-room/crates/application/src/ports.rs](../../crates/application/src/ports.rs)
- [/lite-room/crates/application/src/use_cases.rs](../../crates/application/src/use_cases.rs)
- [/lite-room/crates/application/src/service.rs](../../crates/application/src/service.rs)

Contains use-case orchestration and port traits (interfaces).
No concrete adapter implementations.

## `adapters`
Path:
- [/lite-room/crates/adapters/src](../../crates/adapters/src)

Contains concrete implementations of application ports:
- sqlite repository
- filesystem scanning and thumbnails
- image decoding
- preview pipeline

## `drivers`
Path:
- [/lite-room/crates/drivers/src](../../crates/drivers/src)

Contains entrypoints and runtime wiring (CLI/UI).
