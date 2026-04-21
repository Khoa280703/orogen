# Phase 04: Preference and verifier RL

## Overview

- Priority: P1
- Status: pending
- Goal: optimize judgment and reliability.

## Key Insights

- For a small model, preference tuning is useful only if pair quality is high.
- RL is worth doing only when a real verifier exists.
- In an extreme-optimization plan, this phase is the main path from "better style" to "better judgment".
- Bad preference data or fake verifiers can easily erase gains from earlier phases.

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
- The phase should start with offline pairwise methods and move to RL only after verifiers are trusted.
- Preference data should be targeted at the actual weakest eval buckets, not generic assistant chat.

## Implementation Steps

1. Collect high-quality good/bad pair data focused on current eval failures.
2. Run at least one offline preference-tuning round such as `DPO/ORPO/KTO`.
3. Build verifiers that are hard to game:
   - real code tests
   - exact-match or executable checks
   - citation or evidence validation
4. Start with a small verifier-backed RL pilot and measure whether it improves or destabilizes the model.
5. Scale verifier-backed RL gradually only if the pilot wins on the eval suite.

## Success Criteria

- Hallucination decreases.
- Verifier pass rate improves.
- Final answers become shorter and more accurate.
- The gains survive re-evaluation on the hardest buckets rather than only on the training preference distribution.

## Next Steps

- Move into the RAG stack with a stronger tuned checkpoint and a clearer understanding of which failures are still parametric vs retrieval-bound.
