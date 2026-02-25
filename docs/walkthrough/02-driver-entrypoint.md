# Driver Entrypoint and Command Dispatch

Primary file:
- [/lite-room/crates/drivers/src/main.rs](../../crates/drivers/src/main.rs)

## `main()` startup flow
1. Initialize logging: `logging::init_logging()`.
2. Read CLI args into `Vec<String>`.
3. Load defaults from [/lite-room/crates/drivers/src/config.rs](../../crates/drivers/src/config.rs).
4. Build `ApplicationService` via dependency injection.
5. Bootstrap catalog schema (`bootstrap_catalog`).
6. Parse CLI command (`parse_command`).
7. Dispatch to application layer (`run_command`).

## Dependency injection in `build_application_service()`
Concrete adapters wired into application service:
- `SqliteCatalogRepository`
- `WalkdirFileScanner`
- `FsThumbnailGenerator`
- `ImageCrateDecoder`
- `SystemClock`
- `BackgroundPreviewPipeline`

This is the concrete wiring point for `drivers -> adapters -> application`.

## Command parsing
`parse_command()` converts CLI strings into:
- `Ui`
- `Import { folder }`
- `List`
- `Open { image_id }`
- `ShowEdit { image_id }`
- `SetEdit { image_id, params }`

## Command execution
`run_command()` calls `ApplicationService` methods and maps errors into:
- usage (`CommandError::Usage`)
- runtime (`CommandError::Runtime`)
