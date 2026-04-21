# Phase 05: RAG and grounding stack

## Overview

- Priority: P1
- Status: pending
- Goal: use external memory to compensate for the parameter limits of `4B`.

## Key Insights

- This is the strongest lane for factual/doc QA.
- A reranker is close to mandatory once the corpus gets large.
- For a `4B` model with no latency constraint, strong retrieval plus reranking is not optional; it is part of the intelligence stack.
- This phase should be treated as a core capability multiplier, especially for code/docs/system tasks.

## Requirements

- Qwen-family embedding + reranker.
- Citation-aware prompts.
- Dedicated retrieval metrics.
- Hard negative evaluation and retrieval ablations.
- Baseline models inherited from plan `1057`:
  - `Qwen3-Embedding-4B`
  - `Qwen3-Reranker-4B`

## Implementation Steps

1. Ingest docs/code/specs.
2. Apply semantic chunking.
3. Build the embedding index.
4. Run top-k retrieval.
5. Rerank.
6. Build an evidence-pack prompt.
7. Evaluate citation faithfulness.
8. Compare:
   - `tuned-only`
   - `tuned+retrieval`
   - `tuned+retrieval+reranker`
   - `tuned+retrieval+reranker+reflection`
9. Keep the best grounded stack as the default path for documentation and codebase questions.

## Success Criteria

- Grounded QA improves clearly.
- Factual hallucination drops clearly.
- Retrieval quality improves enough that the model can defer to evidence instead of improvising.

## Next Steps

- Move into the serving reflection loop with retrieval already strong enough that reflection can operate on better evidence.
