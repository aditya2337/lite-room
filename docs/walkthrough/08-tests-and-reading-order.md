# Tests and Reading Order

## Tests to read first

### Driver parse tests
- [/lite-room/crates/drivers/src/main.rs](/lite-room/crates/drivers/src/main.rs)

### Domain invariant tests
- [/lite-room/crates/domain/src/image.rs](/lite-room/crates/domain/src/image.rs)
- [/lite-room/crates/domain/src/edit.rs](/lite-room/crates/domain/src/edit.rs)

### Application service tests (with fakes)
- [/lite-room/crates/application/src/service.rs](/lite-room/crates/application/src/service.rs)

### SQLite adapter tests
- [/lite-room/crates/adapters/src/sqlite/mod.rs](/lite-room/crates/adapters/src/sqlite/mod.rs)

## Reading order
1. [/lite-room/crates/drivers/src/main.rs](/lite-room/crates/drivers/src/main.rs)
2. [/lite-room/crates/application/src/use_cases.rs](/lite-room/crates/application/src/use_cases.rs)
3. [/lite-room/crates/application/src/ports.rs](/lite-room/crates/application/src/ports.rs)
4. [/lite-room/crates/application/src/service.rs](/lite-room/crates/application/src/service.rs)
5. [/lite-room/crates/domain/src/image.rs](/lite-room/crates/domain/src/image.rs)
6. [/lite-room/crates/domain/src/edit.rs](/lite-room/crates/domain/src/edit.rs)
7. [/lite-room/crates/adapters/src/fs/scanner.rs](/lite-room/crates/adapters/src/fs/scanner.rs)
8. [/lite-room/crates/adapters/src/sqlite/mod.rs](/lite-room/crates/adapters/src/sqlite/mod.rs)
9. [/lite-room/crates/adapters/src/fs/thumbs.rs](/lite-room/crates/adapters/src/fs/thumbs.rs)
10. [/lite-room/crates/drivers/src/ui.rs](/lite-room/crates/drivers/src/ui.rs)
11. [/lite-room/crates/adapters/src/preview/mod.rs](/lite-room/crates/adapters/src/preview/mod.rs)
