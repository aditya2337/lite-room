# AGENTS.md

## Architecture Memory (Clean Architecture)

This repository follows a strict Clean Architecture workspace layout:

- `/lite-room/crates/domain`
  - Pure business/domain rules only.
  - No IO/framework/database/UI dependencies.
- `/lite-room/crates/application`
  - Use-cases and ports (traits/interfaces).
  - Depends only on `domain`.
- `/lite-room/crates/adapters`
  - Implementations of application ports (sqlite/fs/image/presenters).
  - Depends on `application` and `domain`.
- `/lite-room/crates/drivers`
  - CLI/UI/runtime wiring and entrypoints.
  - Depends on `adapters`, `application`, and `domain` for DTO/read models.

## Dependency Rule (must never be violated)

Dependencies must point inward only:

`drivers -> adapters -> application -> domain`

Forbidden examples:

- `domain` importing anything from `application`, `adapters`, or `drivers`
- `application` importing from `adapters` or `drivers`
- `adapters` importing from `drivers`

## Change Policy

When implementing new behavior:

1. Add/adjust entities/value objects/errors in `domain`.
2. Add/adjust use-cases and port traits in `application`.
3. Implement ports in `adapters`.
4. Wire commands/UI/runtime flow in `drivers`.

Avoid placing business logic directly in `adapters` or `drivers`.

## Validation Commands

Run these after architectural changes:

- `cargo check -p lite-room-domain`
- `cargo check -p lite-room-application`
- `cargo check -p lite-room-adapters`
- `cargo check -p lite-room-drivers`
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`

## Documentation Path Rule

When writing paths in repository Markdown docs, do not use machine-specific absolute prefixes like `/Users/<name>/...`.
Use repository-rooted paths that start with `/lite-room/...` instead.

## Documentation Maintenance Rule

When implementation behavior changes, agents must update user-facing documentation in the same change.

- Keep `/lite-room/docs/walkthrough/README.md` and files under `/lite-room/docs/walkthrough/` in sync with current command/runtime flow (`import`, `list`, `open`, `show-edit`, `set-edit`, `ui`).
- Keep `/lite-room/docs/CODEBASE_WALKTHROUGH.md` as a short landing page that points to the split walkthrough docs.
- Keep `/lite-room/docs/README.md` updated so new docs are easy to discover.
- Prefer Markdown hyperlinks to repository-rooted paths in docs for better readability/navigation.
- If behavior changed but docs were not updated, treat the task as incomplete.

### Required Flow-Change Checklist (must be followed)

When adding, changing, or touching any user-facing flow (CLI command, UI flow, import/edit/preview pipeline, persistence flow, or adapter wiring), agents must do all of the following in the same change:

1. Update the relevant file(s) under `/lite-room/docs/walkthrough/`.
2. If a new flow is introduced, add a dedicated walkthrough file and link it from `/lite-room/docs/walkthrough/README.md`.
3. If navigation changes, update `/lite-room/docs/README.md`.
4. Keep `/lite-room/docs/CODEBASE_WALKTHROUGH.md` pointing to the split walkthrough index.
5. In the final response, explicitly list which docs were updated.

If no documentation file needed changes, agents must explicitly state why in the final response.
