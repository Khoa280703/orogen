# Phase 06: Serving critique-revise loop

## Overview

- Priority: P1
- Status: pending
- Goal: squeeze more IQ out of a small model through test-time compute.

## Key Insights

- If latency is not a concern, critique/revise is close to mandatory.
- The gain is often better than simply increasing CoT length.
- Under this plan, latency is explicitly a secondary concern, so test-time compute should be used aggressively where it buys accuracy.
- Reflection without verifiers or evidence can become self-delusion; this phase depends on earlier verifier and RAG work.

## Requirements

- Support loops such as:
  - draft -> critique -> revise
  - retrieve -> answer -> verify citation -> rewrite
  - plan -> execute -> verify -> patch
- Support adaptive depth so the system can spend more turns on hard tasks and fewer on easy tasks.
- Preserve a single-pass fallback for cheap requests, but optimize the default path for hard tasks.

## Implementation Steps

1. Design the prompt loops.
2. Choose stop conditions.
3. Add verifier hooks.
4. Add retrieval-aware critique and rewrite paths.
5. Add task difficulty routing or confidence gating for when to invoke deeper loops.
6. Benchmark:
   - `single-pass`
   - `reflection only`
   - `RAG + reflection`
   - `RAG + verifier + reflection`

## Success Criteria

- Accuracy improves.
- Groundedness improves.
- Obvious mistakes decrease.
- The system spends extra test-time compute where it matters and wins enough quality to justify the added latency.

## Next Steps

- Optimize against eval failure cases.
