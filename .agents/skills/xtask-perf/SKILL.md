---
name: xtask-perf
description: >-
  Runs VectorCraft performance budget checks and benchmarks (vectorcraft-cli perf / bench).
  Use after renderer, format, or geometry changes to ensure performance budgets are maintained.
---

# VectorCraft Performance Verification

Runs performance benchmark checks against budgets on a synthetic 50k-path document or times specific files.

## Benchmark Workflow

1. Execute the performance budget check:
   ```bash
   cargo run --release -p vectorcraft-cli -- perf
   ```
2. Benchmark a specific file (if testing targeted optimizations):
   ```bash
   cargo run --release -p vectorcraft-cli -- bench <FILE>
   ```
3. Evaluate metrics:
   - Render / pan / zoom latency
   - Hit test responsiveness
   - Save / open times
   - SVG import/export
   - Pathfinder boolean operations
