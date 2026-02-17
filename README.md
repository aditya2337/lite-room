# lite-room

lite-room is a lightweight, non-destructive desktop RAW photo editor focused on high-performance local workflows.

## Workspace Layout

- `/lite-room/crates/domain`: entities, value objects, and domain invariants
- `/lite-room/crates/application`: use-cases and inbound ports
- `/lite-room/crates/adapters`: sqlite/fs/image implementations and presenters
- `/lite-room/crates/drivers`: CLI/UI runtime entrypoints (`lite-room` binary)

Dependency direction is strictly inward:
`drivers -> adapters -> application -> domain`.

## Build

```bash
cargo check --workspace
cargo test --workspace
```

## Commands

```bash
lite-room ui
lite-room import <folder>
lite-room list
lite-room open <image_id>
```
