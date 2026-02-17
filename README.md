# lite-room

lite-room is a lightweight, non-destructive desktop RAW photo editor focused on high-performance local workflows.

## v1 Focus

- Non-destructive RAW editing
- GPU-accelerated preview rendering
- Local SQLite catalog
- Fast thumbnail and preview pipeline

## Project Docs

- Product spec: `/Users/aditya/workspace/lite-room/SPEC.md`
- Architecture design: `/Users/aditya/workspace/lite-room/ARCHITECTURE.md`
- Build plan: `/Users/aditya/workspace/lite-room/BUILD_PLAN.md`

## Planned Core Features

- Import photos from folders (CR2, NEF, ARW, DNG, JPEG)
- Grid browsing with metadata
- Edit controls:
  - Exposure
  - Contrast
  - Temperature
  - Tint
  - Highlights
  - Shadows
- Non-destructive edit persistence
- JPEG export with quality and resolution options

## Tech Stack (v1)

- Rust
- wgpu
- SQLite (`rusqlite`)
- libraw bindings
- UI: Slint or egui (to be finalized after spike)

## Current Status

Planning complete. Implementation scaffolding and milestone execution are next.
