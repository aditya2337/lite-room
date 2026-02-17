---
name: sqlite-migrations
description: Use when creating or updating SQLite schema, queries, and migrations for catalog, edits, and thumbnail data with backward-safe changes and performance checks.
---

# SQLite Migrations

## Use This Skill When
- Adding or changing tables/columns/indexes.
- Writing catalog queries for import, grid, edit persistence, or cache lookup.
- Improving query performance for large catalogs.

## Required Inputs
- Current schema and desired schema/query change.
- Data compatibility requirement (fresh DB only or existing user DBs).

## Workflow
1. Propose migration with rollback strategy.
2. Add idempotent migration scripts in order.
3. Add/adjust query-layer tests.
4. Run migration on empty and populated sample DB.
5. Review query plan for hot paths.

## Guardrails
- Never break existing catalog data without explicit migration handling.
- Add indexes for frequent filters/sorts (`capture_date`, import recency).
- Use transactions for batch writes.

## Output Contract
- Migration file(s) and query updates.
- Compatibility notes and test results.
