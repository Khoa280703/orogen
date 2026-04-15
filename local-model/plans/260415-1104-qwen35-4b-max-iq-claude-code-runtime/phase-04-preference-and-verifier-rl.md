# Phase 04: Preference and verifier RL

## Overview

- Priority: P1
- Status: pending
- Goal: optimize judgment and reliability.

## Key Insights

- For a small model, preference tuning is useful only if pair quality is high.
- RL is worth doing only when a real verifier exists.

## Requirements

- Preference pairs for:
  - groundedness
  - lower hallucination
  - concise correctness
  - better repair decisions
- Verifiers for:
  - code tests
  - exact answers
  - citation faithfulness

## Implementation Steps

1. Collect good/bad pair data.
2. Run DPO/ORPO/KTO.
3. Build verifier-backed RL tasks.
4. Start with a small verifier-based RL round and scale up gradually.

## Success Criteria

- Hallucination decreases.
- Verifier pass rate improves.
- Final answers become shorter and more accurate.

## Next Steps

- Move into the RAG stack.
