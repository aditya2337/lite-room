# lite-room App Mock Design (Agent Placement Guide)

This document is the practical map for where each feature belongs in the Clean Architecture workspace.

## 1. Dependency Direction (Always)

```text
drivers -> adapters -> application -> domain
```

Rules:
- `domain` has no dependency on other project crates.
- `application` depends only on `domain`.
- `adapters` depends on `application` + `domain`.
- `drivers` is composition/wiring only.

## 2. What Goes Where

### Domain (`/lite-room/crates/domain`)

Put here:
- Business entities/value objects (`ImageId`, `ImageRecord`, `EditParams`).
- Domain invariants and validation.
- Domain-level errors.
- Pure transformations that do not require IO/framework/runtime.

Do not put here:
- SQLite, filesystem, preview worker queues, CLI/UI concerns.

### Application (`/lite-room/crates/application`)

Put here:
- Use-case commands/queries (`ImportFolderCommand`, `SetEditCommand`, etc.).
- Port traits (`CatalogRepository`, `FileScanner`, `PreviewPipeline`).
- Orchestration and transaction flow (`ApplicationService`).
- Application-level errors.

Do not put here:
- Concrete DB/FS/image libraries.
- CLI argument parsing or UI widget logic.

### Adapters (`/lite-room/crates/adapters`)

Put here:
- Port implementations (sqlite, fs scanner, thumbnail generator, preview background pipeline).
- SQL queries/migrations.
- Presenters/formatters used by drivers.

Do not put here:
- Business rules that decide product behavior.
- CLI command routing.

### Drivers (`/lite-room/crates/drivers`)

Put here:
- Entrypoints (`main.rs`), command parsing, UI startup/runtime config.
- Dependency wiring (`build_application_service`).
- Flow from user input to application use-cases.

Do not put here:
- Raw SQL, file scanning internals, edit computation rules.

## 3. Feature-to-Placement Matrix (v1)

### Feature: Import Folder
- Domain:
  - Supported image kinds / file semantics.
- Application:
  - `ImportFolderCommand`
  - import orchestration in `ApplicationService`.
  - scanner/catalog/thumb ports.
- Adapters:
  - walkdir scanner implementation.
  - sqlite upsert + migration changes.
  - thumbnail generation/cache write.
- Drivers:
  - `import <folder>` CLI command and UI import trigger.

### Feature: Grid/List Catalog
- Domain:
  - `ImageRecord` shape and invariants.
- Application:
  - `ListImagesCommand` use-case.
- Adapters:
  - sqlite list query + row mapping.
  - row presenter formatting.
- Drivers:
  - `list` command output and UI grid binding.

### Feature: Open Image + Edit Params
- Domain:
  - `EditParams` model + defaults + validation ranges.
- Application:
  - `OpenImageCommand`, `ShowEditCommand`, `SetEditCommand`.
  - edit persistence orchestration.
- Adapters:
  - image decode implementation.
  - edit row read/write in sqlite.
- Drivers:
  - `open`, `show-edit`, `set-edit` command interfaces.
  - slider events in UI.

### Feature: Preview Pipeline (async)
- Domain:
  - `PreviewRequest`, `PreviewFrame`, `PreviewMetrics`.
- Application:
  - submit/poll/metrics use-cases.
  - preview pipeline port contract.
- Adapters:
  - background preview worker/channel implementation.
- Drivers:
  - UI frame polling schedule and display updates.

### Feature: Export JPEG (planned)
- Domain:
  - export parameter value objects and constraints.
- Application:
  - `ExportImageCommand` + export port trait.
- Adapters:
  - concrete encoder implementation and output write.
- Drivers:
  - `export` CLI command and UI export dialog flow.

## 4. Canonical File Placement Pattern

When adding a new feature, use this order:

1. `domain/src/<feature>.rs`
- Add entity/value object/invariant/error additions.

2. `application/src/use_cases.rs`
- Add command/query types.

3. `application/src/ports.rs`
- Add/extend trait contract.

4. `application/src/service.rs`
- Implement/adjust use-case orchestration.

5. `adapters/src/<adapter area>/...`
- Implement the new/updated port.
- Add migration/query files if persistent model changes.

6. `drivers/src/main.rs` and/or `drivers/src/ui.rs`
- Wire user entrypoint and call the application service.

## 5. Agent Rules for New Work

Before coding:
- Identify if requirement is business rule, orchestration, infrastructure, or UI entrypoint.
- Place code in the innermost valid layer.

During coding:
- If a layer needs data from outer layer, define an inward-facing port instead of importing outward.
- Keep DTO mapping in adapters or drivers, not domain.

After coding:
- Run architecture checks:
  - `cargo check -p lite-room-domain`
  - `cargo check -p lite-room-application`
  - `cargo check -p lite-room-adapters`
  - `cargo check -p lite-room-drivers`
  - `cargo test --workspace`
  - `cargo clippy --all-targets --all-features -- -D warnings`

## 6. Quick Anti-Pattern Detector

Stop and relocate code if you see:
- SQL or filesystem code in `application` or `domain`.
- CLI parsing logic in `adapters` or `application`.
- Domain entities importing adapter types.
- Business rules implemented directly in `drivers`.

## 7. Decision Shortcut

Ask: "If we replaced SQLite+CLI with something else, should this code survive unchanged?"
- Yes -> likely `domain` or `application`.
- No -> likely `adapters` or `drivers`.
