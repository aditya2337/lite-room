---
name: release-engineering
description: Use when preparing builds, versioning, packaging, and release validation for macOS-first desktop delivery, then extending to cross-platform targets.
---

# Release Engineering

## Use This Skill When
- Creating release branches, version bumps, and changelogs.
- Packaging desktop binaries.
- Running release gates before publish.

## Required Inputs
- Target version and release scope.
- Target platforms.
- Required release artifacts.

## Workflow
1. Freeze scope and verify all milestone gates are green.
2. Run clean build and full test suite.
3. Produce signed package/binary artifacts.
4. Run smoke tests on clean environment.
5. Publish release notes with known limitations.

## Guardrails
- No release without reproducible artifact checksums.
- Keep debug symbols and crash diagnostics available.
- Tag versions only after artifact validation.

## Output Contract
- Release artifacts.
- Version tag and release notes.
- Post-release validation checklist status.
