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
- Phase checklist: `/Users/aditya/workspace/lite-room/PHASE_CHECKLIST.md`
- MCP setup checklist: `/Users/aditya/workspace/lite-room/MCP_SETUP_CHECKLIST.md`

## Codex Skills (Repo-local Drafts)

- Rust development: `/Users/aditya/workspace/lite-room/skills/rust-dev/SKILL.md`
- WGPU shader pipeline: `/Users/aditya/workspace/lite-room/skills/wgpu-shader/SKILL.md`
- SQLite migrations: `/Users/aditya/workspace/lite-room/skills/sqlite-migrations/SKILL.md`
- Image pipeline validation: `/Users/aditya/workspace/lite-room/skills/image-pipeline/SKILL.md`
- Benchmarking/perf: `/Users/aditya/workspace/lite-room/skills/benchmarking/SKILL.md`
- Release engineering: `/Users/aditya/workspace/lite-room/skills/release-engineering/SKILL.md`

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
