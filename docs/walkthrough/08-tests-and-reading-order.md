# Tests and Reading Order

## Tests to read first

### Driver parse tests
- [/lite-room/crates/drivers/src/main.rs](../../crates/drivers/src/main.rs)

### Domain invariant tests
- [/lite-room/crates/domain/src/image.rs](../../crates/domain/src/image.rs)
- [/lite-room/crates/domain/src/edit.rs](../../crates/domain/src/edit.rs)

### Application service tests (with fakes)
- [/lite-room/crates/application/src/service.rs](../../crates/application/src/service.rs)

### SQLite adapter tests
- [/lite-room/crates/adapters/src/sqlite/mod.rs](../../crates/adapters/src/sqlite/mod.rs)

## Reading order
1. [/lite-room/crates/drivers/src/main.rs](../../crates/drivers/src/main.rs)
2. [/lite-room/crates/application/src/use_cases.rs](../../crates/application/src/use_cases.rs)
3. [/lite-room/crates/application/src/ports.rs](../../crates/application/src/ports.rs)
4. [/lite-room/crates/application/src/service.rs](../../crates/application/src/service.rs)
5. [/lite-room/crates/domain/src/image.rs](../../crates/domain/src/image.rs)
6. [/lite-room/crates/domain/src/edit.rs](../../crates/domain/src/edit.rs)
7. [/lite-room/crates/adapters/src/fs/scanner.rs](../../crates/adapters/src/fs/scanner.rs)
8. [/lite-room/crates/adapters/src/sqlite/mod.rs](../../crates/adapters/src/sqlite/mod.rs)
9. [/lite-room/crates/adapters/src/fs/thumbs.rs](../../crates/adapters/src/fs/thumbs.rs)
10. [/lite-room/crates/drivers/src/ui.rs](../../crates/drivers/src/ui.rs)
11. [/lite-room/crates/adapters/src/preview/mod.rs](../../crates/adapters/src/preview/mod.rs)
