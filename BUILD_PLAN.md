# lite-room Build Plan (v1)

## 1. Plan Summary

This plan delivers a usable v1 RAW editor in progressive slices, prioritizing:
- fast visual feedback,
- non-destructive correctness,
- stable local catalog behavior.

## 2. Delivery Principles

- Vertical slices over isolated subsystems.
- Performance checks in every milestone.
- No milestone is complete without basic acceptance validation.

## 3. Phased Milestones

## Phase 0: Foundation and Spikes (1 week)

### Goals
- Initialize Rust workspace and module layout.
- Add CI checks (build, fmt, clippy, unit tests).
- Run UI framework spike (Slint vs egui) and lock choice.
- Validate libraw + wgpu integration feasibility.

### Deliverables
- Bootstrapped app shell with placeholder views.
- Decision record for UI framework.
- Minimal render loop showing test image.

### Exit Criteria
- App launches on macOS.
- Selected UI framework documented.
- Basic texture shown via wgpu.

## Phase 1: Catalog + JPEG Baseline (1-2 weeks)

### Goals
- Implement SQLite catalog and import flow for JPEG.
- Build grid view with incremental loading.
- Generate and cache thumbnails.

### Deliverables
- Folder import pipeline.
- `images`, `edits`, `thumbnails` schema + migrations.
- Grid view with sort-by-date and open-on-click.

### Exit Criteria
- Import 100 JPEG images and show in grid within 2 seconds target.
- Reopen app and load catalog state from SQLite.
- Thumbnail cache hit behavior confirmed.

## Phase 2: RAW Decode and Single Image View (1-2 weeks)

### Goals
- Add RAW file support with libraw.
- Display demosaiced image in edit view.
- Add metadata extraction for RAW.

### Deliverables
- RAW decoder abstraction.
- Edit view with full preview container + histogram placeholder.
- Supported format filter: CR2/NEF/ARW/DNG/JPEG.

### Exit Criteria
- Open and preview at least one sample from each supported RAW format set (where available).
- Graceful handling of unsupported/corrupt files.

## Phase 3: Non-Destructive Edit Model (1 week)

### Goals
- Implement `EditParams` model and persistence.
- Wire sliders: exposure, contrast, temperature, tint, highlights, shadows.
- Autosave edits with debounce.

### Deliverables
- Edit param serialization/deserialization JSON.
- DB read/write path for edits table.
- Slider UI + app state synchronization.

### Exit Criteria
- Slider changes persist across app restart.
- Original source files remain unchanged after edits.

## Phase 4: GPU Preview Pipeline (2 weeks)

### Goals
- Move edit transforms to wgpu compute/render stages.
- Achieve responsive real-time preview under target conditions.
- Implement stale render cancellation and latest-state prioritization.

### Deliverables
- GPU shader stages for v1 pipeline.
- Preview job scheduling and cancellation logic.
- Render timing instrumentation.

### Exit Criteria
- Preview update p95 under 50 ms for 24 MP scaled preview on target hardware.
- No major UI hitching during rapid slider movement.

## Phase 5: Export Pipeline (1 week)

### Goals
- Full resolution processing and JPEG export.
- Export controls for quality and resolution.

### Deliverables
- Export worker pipeline.
- JPEG encoder integration.
- Export status/progress and error messaging.

### Exit Criteria
- Exported JPEG reflects active edits.
- Export does not mutate DB edit history unexpectedly.

## Phase 6: Hardening and Optimization (1-2 weeks)

### Goals
- Improve catalog scale behavior to 5000+ assets.
- Tighten memory usage and cache policy.
- Add crash-safety and recovery validation.

### Deliverables
- Performance tuning pass.
- WAL/transaction reliability checks.
- Startup and import profiling notes.

### Exit Criteria
- Catalog with 5000 images remains usable with no severe lag.
- Recover cleanly from forced app restart with edits intact.

## 4. Workstreams in Parallel

- Core engine/rendering workstream.
- UI interaction and state management workstream.
- Catalog/storage and import/export workstream.

Weekly integration branch merges should be mandatory to avoid late subsystem conflicts.

## 5. Testing Strategy

## 5.1 Unit Tests
- Edit param serialization.
- DB CRUD operations and migration validity.
- Import format filtering and metadata parsing fallbacks.

## 5.2 Integration Tests
- Import folder -> grid population -> open edit view -> persist edit -> restart -> restore.
- Export path end-to-end from RAW + edits.

## 5.3 Performance Tests
- Import throughput for 100 and 1000 images.
- Preview latency benchmark under rapid slider changes.
- Memory profile with large catalogs and repeated image switching.

## 5.4 Manual QA Checklist
- Verify source RAW immutability.
- Verify error messaging for corrupt files.
- Verify catalog consistency after app crash simulation.

## 6. Risk Register and Mitigation

## Risk: RAW decoding edge cases
- Mitigation: Build a sample corpus early and keep format-specific regression tests.

## Risk: GPU shader debugging complexity
- Mitigation: Start with CPU-reference pipeline and compare outputs frame-by-frame.

## Risk: Color mismatch
- Mitigation: Establish baseline rendering references and controlled test images.

## Risk: Memory pressure with large RAW files
- Mitigation: Explicit cache eviction policy and bounded in-memory buffers.

## 7. Team Cadence (if solo, keep same rhythm)

- Daily: progress + blockers + perf notes.
- Weekly: milestone demo and metric snapshot.
- End of each phase: stabilization day before moving on.

## 8. Definition of Done (v1)

v1 is complete when all are true:
- User can import, browse, edit, and export RAW images.
- Edits are fully non-destructive and persisted in SQLite.
- Preview feels responsive and meets target latency envelope.
- Catalog scale target (5000 images) is practically usable.
