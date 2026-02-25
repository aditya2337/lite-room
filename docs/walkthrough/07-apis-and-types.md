# Core APIs and Types

## Application service methods
File:
- [/lite-room/crates/application/src/service.rs](/lite-room/crates/application/src/service.rs)

Key methods:
- `bootstrap_catalog`
- `import_folder`
- `list_images`
- `open_image`
- `show_edit`
- `set_edit`
- `submit_preview`
- `poll_preview`
- `preview_metrics`

## Application port traits
File:
- [/lite-room/crates/application/src/ports.rs](/lite-room/crates/application/src/ports.rs)

Primary traits:
- `CatalogRepository`
- `FileScanner`
- `ThumbnailGenerator`
- `ImageDecoder`
- `Clock`
- `PreviewPipeline`

## Domain DTOs
Files:
- [/lite-room/crates/domain/src/image.rs](/lite-room/crates/domain/src/image.rs)
- [/lite-room/crates/domain/src/edit.rs](/lite-room/crates/domain/src/edit.rs)
- [/lite-room/crates/domain/src/preview.rs](/lite-room/crates/domain/src/preview.rs)

Primary DTOs:
- `ImageId`, `ImageRecord`, `ImportReport`, `DecodedImage`
- `EditParams`
- `PreviewRequest`, `PreviewFrame`, `PreviewMetrics`
