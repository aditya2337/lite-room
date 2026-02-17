# lite-room Phase Checklist (v1)

Status legend:
- `Done`: Completed and validated.
- `In Progress`: Partially complete or missing validation.
- `Not Started`: No implementation yet.

## Phase 0: Foundation and Spikes

Overall status: `In Progress`

Goals:
- [x] Initialize Rust workspace and module layout.
- [x] Add CI checks (build, fmt, clippy, unit tests).
- [ ] Run UI framework spike (Slint vs egui) and lock choice.
- [ ] Validate libraw + wgpu integration feasibility.

Deliverables:
- [x] Bootstrapped app shell with placeholder views.
- [ ] Decision record for UI framework.
- [ ] Minimal render loop showing test image.

Exit criteria:
- [x] App launches and initializes from CLI on macOS.
- [ ] Selected UI framework documented.
- [ ] Basic texture shown via wgpu.

## Phase 1: Catalog + JPEG Baseline

Overall status: `In Progress`

Goals:
- [x] Implement SQLite catalog and import flow for JPEG.
- [ ] Build grid view with incremental loading.
- [x] Generate and cache thumbnails.

Deliverables:
- [x] Folder import pipeline.
- [x] `images`, `edits`, `thumbnails` schema + migrations.
- [ ] Grid view with sort-by-date and open-on-click.

Exit criteria:
- [ ] Import 100 JPEG images and show in grid within 2 seconds.
- [x] Reopen app and load catalog state from SQLite.
- [x] Thumbnail cache hit behavior confirmed via existing-thumbnail reuse path.

## Phase 2: RAW Decode and Single Image View

Overall status: `Not Started`

Goals:
- [ ] Add RAW file support with libraw.
- [ ] Display demosaiced image in edit view.
- [ ] Add metadata extraction for RAW.

Deliverables:
- [ ] RAW decoder abstraction.
- [ ] Edit view with full preview container + histogram placeholder.
- [ ] Supported format filter: CR2/NEF/ARW/DNG/JPEG.

Exit criteria:
- [ ] Open and preview sample images from supported RAW formats.
- [ ] Graceful handling of unsupported/corrupt files.

## Phase 3: Non-Destructive Edit Model

Overall status: `Not Started`

Goals:
- [ ] Implement `EditParams` model and persistence.
- [ ] Wire sliders: exposure, contrast, temperature, tint, highlights, shadows.
- [ ] Autosave edits with debounce.

Deliverables:
- [ ] Edit param serialization/deserialization JSON.
- [ ] DB read/write path for edits table.
- [ ] Slider UI + app state synchronization.

Exit criteria:
- [ ] Slider changes persist across app restart.
- [ ] Original source files remain unchanged after edits.

## Phase 4: GPU Preview Pipeline

Overall status: `Not Started`

Goals:
- [ ] Move edit transforms to wgpu compute/render stages.
- [ ] Achieve responsive real-time preview under target conditions.
- [ ] Implement stale render cancellation and latest-state prioritization.

Deliverables:
- [ ] GPU shader stages for v1 pipeline.
- [ ] Preview job scheduling and cancellation logic.
- [ ] Render timing instrumentation.

Exit criteria:
- [ ] Preview update p95 under 50 ms for 24 MP scaled preview on target hardware.
- [ ] No major UI hitching during rapid slider movement.

## Phase 5: Export Pipeline

Overall status: `Not Started`

Goals:
- [ ] Full resolution processing and JPEG export.
- [ ] Export controls for quality and resolution.

Deliverables:
- [ ] Export worker pipeline.
- [ ] JPEG encoder integration.
- [ ] Export status/progress and error messaging.

Exit criteria:
- [ ] Exported JPEG reflects active edits.
- [ ] Export does not mutate DB edit history unexpectedly.

## Phase 6: Hardening and Optimization

Overall status: `Not Started`

Goals:
- [ ] Improve catalog scale behavior to 5000+ assets.
- [ ] Tighten memory usage and cache policy.
- [ ] Add crash-safety and recovery validation.

Deliverables:
- [ ] Performance tuning pass.
- [ ] WAL/transaction reliability checks.
- [ ] Startup and import profiling notes.

Exit criteria:
- [ ] Catalog with 5000 images remains usable with no severe lag.
- [ ] Recover cleanly from forced app restart with edits intact.

## Completed So Far (from current codebase)

- [x] Rust project scaffold with architecture-aligned module structure.
- [x] CI workflow for `fmt`, `clippy`, and `test`.
- [x] SQLite catalog schema and migration wiring.
- [x] JPEG folder import pipeline with supported-format filtering.
- [x] Default non-destructive edit row creation per imported image.
- [x] Real JPEG thumbnail generation + cache pathing and DB thumbnail records.
- [x] CLI commands for `import <folder>` and `list`.
- [x] Unit tests for schema initialization and JPEG import baseline.
