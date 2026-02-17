---
name: benchmarking
description: Use when measuring and improving import throughput, preview latency, memory usage, and export performance against project SLOs.
---

# Benchmarking

## Use This Skill When
- Validating performance targets.
- Comparing implementations or regressions.
- Producing repeatable performance reports for milestones.

## Required Inputs
- Scenario definition (import, edit preview, export).
- Hardware profile and dataset size.
- Success thresholds (for example preview p95 < 50 ms).

## Workflow
1. Define benchmark harness and fixed dataset.
2. Warm up and run multiple iterations.
3. Record p50/p95, max, and memory footprint.
4. Compare against baseline commit.
5. Recommend changes ranked by expected impact.

## Guardrails
- Keep runs reproducible (same resolution, same params, same hardware).
- Separate correctness tests from perf tests.
- Do not declare success from a single run.

## Output Contract
- Benchmark report with metrics table.
- Regression/improvement summary.
- Follow-up actions if thresholds are missed.
