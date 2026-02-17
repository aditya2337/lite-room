# lite-room MCP Setup Checklist

## 1. GitHub MCP

Purpose:
- Manage issues, PRs, milestones, and project tracking from the agent.

Checklist:
- [ ] Install/configure GitHub MCP server in Codex.
- [ ] Authenticate with a token that has repo and project permissions.
- [ ] Connect repository: `aditya2337/lite-room`.
- [ ] Verify read operations: list issues and PRs.
- [ ] Verify write operations: create a test issue and close it.

Done criteria:
- Agent can create/update issues and comment on PRs for this repo.

## 2. SQLite MCP

Purpose:
- Inspect and validate catalog schema, migrations, and runtime query behavior.

Checklist:
- [ ] Install/configure SQLite MCP server.
- [ ] Point server to local catalog DB path used by app.
- [ ] Verify schema read (tables, indexes, foreign keys).
- [ ] Verify controlled query execution (read-only first).
- [ ] Add safe profile for migration validation runs.

Done criteria:
- Agent can inspect schema and run diagnostic queries for performance and correctness.

## 3. Filesystem MCP

Purpose:
- Structured file operations and artifact management in workspace.

Checklist:
- [ ] Configure filesystem MCP with workspace root `/Users/gvadityanaidu/personal/lite-room`.
- [ ] Confirm read/write permissions in project directories.
- [ ] Confirm access to cache/test-assets directories when added.

Done criteria:
- Agent can reliably read, edit, and create project files without path issues.

## 4. Rust Docs MCP (or docs lookup MCP)

Purpose:
- Fast primary-doc lookup for `wgpu`, `rusqlite`, and related crates.

Checklist:
- [ ] Add docs MCP server or equivalent docs index.
- [ ] Validate source preference is official crate docs and primary references.
- [ ] Test lookups for `wgpu` pipeline setup and `rusqlite` transaction APIs.

Done criteria:
- Agent retrieves accurate API references during implementation tasks.

## 5. Observability MCP

Purpose:
- Inspect logs/metrics for preview latency, import throughput, and export duration.

Checklist:
- [ ] Configure log/metrics backend used by local app runs.
- [ ] Define core metrics: preview p50/p95, import rate, cache hit rate, export duration.
- [ ] Validate agent can query recent runs and filter by subsystem tags.

Done criteria:
- Agent can diagnose performance regressions with real run metrics.

## 6. Image Test Assets MCP

Purpose:
- Manage canonical RAW/JPEG corpus for decode and visual regression tests.

Checklist:
- [ ] Set up dataset source and MCP access.
- [ ] Organize corpus by camera/model/format.
- [ ] Mark baseline expected outputs for regression tests.
- [ ] Validate checksums for dataset consistency.

Done criteria:
- Agent can run repeatable visual and performance checks against stable assets.

## 7. Rollout Order

Recommended order:
1. Filesystem MCP
2. GitHub MCP
3. SQLite MCP
4. Rust Docs MCP
5. Image Test Assets MCP
6. Observability MCP

Rationale:
- Enables core coding flow first, then correctness tooling, then performance tooling.

## 8. Repo Integration Tasks

- [ ] Add a section in `/Users/gvadityanaidu/personal/lite-room/README.md` linking this checklist.
- [ ] Create GitHub labels: `perf`, `rendering`, `catalog`, `export`, `bug`, `milestone`.
- [ ] Create milestone issues aligned to `/Users/gvadityanaidu/personal/lite-room/BUILD_PLAN.md`.
- [ ] Add a performance issue template that records hardware and dataset details.
