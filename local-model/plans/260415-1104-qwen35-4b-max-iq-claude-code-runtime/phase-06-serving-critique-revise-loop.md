# Phase 06: Serving critique-revise loop

## Overview

- Priority: P1
- Status: pending
- Goal: squeeze more IQ out of a small model through test-time compute.

## Key Insights

- If latency is not a concern, critique/revise is close to mandatory.
- The gain is often better than simply increasing CoT length.

## Requirements

- Support loops such as:
  - draft -> critique -> revise
  - retrieve -> answer -> verify citation -> rewrite
  - plan -> execute -> verify -> patch

## Implementation Steps

1. Design the prompt loops.
2. Choose stop conditions.
3. Add verifier hooks.
4. Benchmark `single-pass vs reflection`.

## Success Criteria

- Accuracy improves.
- Groundedness improves.
- Obvious mistakes decrease.

## Next Steps

- Optimize against eval failure cases.
