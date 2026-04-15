# Phase 02: Continual pretraining corpus

## Overview

- Priority: P1
- Status: pending
- Goal: strengthen the technical prior of `Qwen3.5-4B`.

## Key Insights

- This is the strongest "real IQ" lever, but also the slowest one.
- Data quality matters more than sheer volume.

## Requirements

- Clean corpus, deduplicated, with provenance.
- Prioritize code, docs, logs, and troubleshooting material.

## Implementation Steps

1. Collect corpora:
   - high-quality code
   - API docs
   - design docs
   - issue-to-fix narratives
   - troubleshooting references
2. Deduplicate and filter noise.
3. Train a small continual pretraining round.
4. Re-evaluate before moving into major SFT.

## Success Criteria

- Clear improvement in code/doc understanding without requiring overly long reasoning traces.

## Next Steps

- Move into workflow-trace SFT.
