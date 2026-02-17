# Lumiere Architecture Design (v1)

## 1. Objectives

Lumiere v1 is a desktop-first, non-destructive RAW editor optimized for fast preview feedback and local catalog scale.

Key targets:
- Preview latency under 50 ms after slider changes (scaled preview).
- Catalog scale of 5000+ images.
- No writes to original RAW files.

## 2. High-Level Architecture

```text
+---------------------+      +----------------------+      +----------------------+
|       UI Layer      | ---> | Application Layer    | ---> | Image Engine         |
| (Slint or egui)     | <--- | (state + use-cases)  | <--- | (decode + process)   |
+---------------------+      +----------------------+      +----------------------+
           |                            |                               |
           v                            v                               v
   +----------------+           +----------------+              +--------------------+
   | Input/Event    |           | Catalog Service|              | GPU Renderer (wgpu)|
   +----------------+           +----------------+              +--------------------+
                                         |
                                         v
                                 +------------------+
                                 | SQLite + cache   |
                                 | (thumbs/previews)|
                                 +------------------+
```

## 3. Layer Responsibilities

### 3.1 UI Layer
- Grid view, edit view, histogram, sliders, import/export dialogs.
- Emits user intents only (e.g., `ImportFolder`, `SetExposure`).
- Renders state snapshots provided by Application Layer.

### 3.2 Application Layer
- Owns app state machine and orchestration.
- Manages async tasks: import scanning, thumbnail generation, preview updates, export.
- Coordinates services:
  - Catalog Service (SQLite CRUD)
  - Engine Service (decode/process/render)
  - Cache Service (thumbnail/preview disk cache)

### 3.3 Image Engine Layer
- RAW decode via libraw bindings.
- CPU preprocessing + GPU pipeline execution via wgpu.
- Produces:
  - Thumbnails
  - Edit previews
  - Full-resolution export buffer

## 4. Runtime Subsystems

## 4.1 Catalog Service
- Database: SQLite (`rusqlite`).
- Stores image metadata, edit params, ratings/flags, thumbnail pointers.
- Uses transactions for import batches and edit autosave.

## 4.2 Cache Service
- On-disk cache folders:
  - `cache/thumbs/` for grid thumbnails.
  - `cache/previews/` for scaled edit previews.
- Cache key includes:
  - image id
  - source file modified timestamp
  - edit params hash
  - requested resolution

## 4.3 Import Pipeline
1. Enumerate files in selected directory.
2. Filter supported extensions.
3. Extract metadata.
4. Insert/update catalog rows.
5. Generate thumbnail job queue.
6. UI incrementally updates as records arrive.

## 4.4 Edit/Preview Pipeline
1. Load RAW once per active image session.
2. Keep decoded linear working buffer in memory.
3. Apply edit params to processing graph.
4. Run GPU shader stages.
5. Present scaled preview to UI.
6. Persist edit params JSON async (debounced autosave).

## 4.5 Export Pipeline
1. Resolve final edit params from DB.
2. Decode source RAW at full resolution.
3. Run full pipeline.
4. Encode JPEG with selected quality/resolution.
5. Write export output.

## 5. Data Model

## 5.1 Proposed Schema (v1)

```sql
CREATE TABLE images (
  id INTEGER PRIMARY KEY,
  file_path TEXT NOT NULL UNIQUE,
  import_date TEXT NOT NULL,
  capture_date TEXT,
  camera_model TEXT,
  iso INTEGER,
  rating INTEGER NOT NULL DEFAULT 0,
  flag INTEGER NOT NULL DEFAULT 0,
  metadata_json TEXT NOT NULL
);

CREATE TABLE edits (
  image_id INTEGER PRIMARY KEY,
  edit_params_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(image_id) REFERENCES images(id) ON DELETE CASCADE
);

CREATE TABLE thumbnails (
  image_id INTEGER PRIMARY KEY,
  file_path TEXT NOT NULL,
  width INTEGER NOT NULL,
  height INTEGER NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(image_id) REFERENCES images(id) ON DELETE CASCADE
);

CREATE INDEX idx_images_import_date ON images(import_date);
CREATE INDEX idx_images_capture_date ON images(capture_date);
```

## 5.2 Edit Parameters Contract

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
struct EditParams {
    exposure: f32,
    contrast: f32,
    temperature: f32,
    tint: f32,
    highlights: f32,
    shadows: f32,
}
```

Default edit params are stored on first import (or first open) to guarantee deterministic rendering.

## 6. Rendering Design

## 6.1 Processing Graph (v1)
1. RAW demosaic.
2. White balance + temperature/tint.
3. Exposure.
4. Highlights/shadows recovery.
5. Contrast/tone mapping.
6. Gamma and output conversion.
7. Framebuffer presentation.

## 6.2 GPU Strategy
- Use wgpu compute shaders for pixel transforms and staging textures.
- Keep processing in GPU memory once upload is complete.
- Minimize CPU-GPU roundtrips.

## 6.3 Latency Controls
- Debounce slider events (short interval) while preserving fluid motion.
- Prioritize latest slider state; cancel stale render jobs.
- Use scaled preview resolution in edit mode.

## 7. Concurrency Model

- Main/UI thread: event loop + render submission.
- Worker pool:
  - Import scanning worker.
  - Thumbnail generation workers.
  - Metadata extraction worker.
  - Export worker(s).
- Message-passing channels between UI/App/Engine to avoid shared mutable state contention.

## 8. Error Handling and Recovery

- Unsupported/corrupt files:
  - mark record with error state in metadata.
  - surface non-blocking warning in UI.
- DB writes:
  - transactional commit + rollback on failure.
  - WAL mode for safer concurrent reads/writes.
- Autosave:
  - debounced and retryable.
  - restore last persisted edits on app restart.

## 9. Observability

- Structured logs with subsystem tags: `import`, `db`, `engine`, `render`, `export`.
- Basic metrics counters:
  - import throughput
  - thumbnail generation time
  - preview render time p50/p95
  - export duration

## 10. Security and Data Safety

- Never write to source RAW path.
- Write exports to explicit output folder only.
- Validate paths and extension support before processing.
- Keep catalog and cache in app-owned local directory.

## 11. Folder/Layout Proposal

```text
src/
  app/
    controller.rs
    state.rs
    events.rs
  ui/
    grid_view.rs
    edit_view.rs
    histogram.rs
  engine/
    raw_decode.rs
    pipeline.rs
    renderer_wgpu.rs
    export.rs
  catalog/
    db.rs
    models.rs
    queries.rs
    migrations/
  cache/
    thumbs.rs
    previews.rs
  infra/
    logging.rs
    config.rs
```

## 12. Decision Points

- UI framework spike: Slint vs egui (startup time, rendering integration complexity, widget velocity).
- Histogram implementation detail (GPU-derived vs CPU).
- Color management scope in v1 (basic sRGB output vs broader ICC support).
