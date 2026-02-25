# Other Commands

After understanding `import`, map the same flow to the rest.

## `list`
1. Driver calls `run_command(Command::List)`.
2. Application calls `list_images(ListImagesCommand)`.
3. Catalog adapter returns rows from SQLite.
4. Presenter formats rows.

Files:
- [/lite-room/crates/drivers/src/main.rs](../../crates/drivers/src/main.rs)
- [/lite-room/crates/application/src/service.rs](../../crates/application/src/service.rs)
- [/lite-room/crates/adapters/src/sqlite/mod.rs](../../crates/adapters/src/sqlite/mod.rs)
- [/lite-room/crates/adapters/src/presenters/mod.rs](../../crates/adapters/src/presenters/mod.rs)

## `open <image_id>`
1. Driver parses and validates `ImageId`.
2. Application fetches image record from catalog.
3. Decoder adapter returns preview decode metadata.
4. Presenter prints dimensions and kind.

Files:
- [/lite-room/crates/adapters/src/lib.rs](../../crates/adapters/src/lib.rs)
- [/lite-room/crates/application/src/service.rs](../../crates/application/src/service.rs)

## `show-edit <image_id>`
1. Application loads edit JSON from catalog.
2. JSON deserializes into `EditParams`.
3. Driver prints formatted edit params.

## `set-edit <image_id> ...`
1. Driver parses float args into `EditParams`.
2. Application validates `EditParams`.
3. Application upserts edit JSON in catalog.

## `ui`
Files:
- [/lite-room/crates/drivers/src/ui.rs](../../crates/drivers/src/ui.rs)
- [/lite-room/crates/adapters/src/preview/mod.rs](../../crates/adapters/src/preview/mod.rs)

High-level loop:
1. Load initial image state and params.
2. Submit preview jobs.
3. Poll preview frames.
4. Handle slider/image navigation events.
5. Debounce autosave (`set_edit`).
