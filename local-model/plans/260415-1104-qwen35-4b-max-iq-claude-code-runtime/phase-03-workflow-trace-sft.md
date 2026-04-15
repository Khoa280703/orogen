# Phase 03: Workflow-trace SFT

## Overview

- Priority: P1
- Status: pending
- Goal: teach the model to operate well in a runtime where tools are already available.

## Key Insights

- It is not necessary to overfit on pure tool-schema samples.
- The model needs training traces that look like observe -> plan -> edit -> verify.

## Requirements

- Data should include:
  - reasoning distill
  - coding/debugging traces
  - grounded QA
  - concise answer quality
- Inherit the shortlist from plan `1057`:
  - `Farseen0/opus-4.6-reasoning-sft-12k`
  - `Roman1111111/claude-opus-4.6-10000x`
  - `nohurry/Opus-4.6-Reasoning-3000x-filtered`
  - `TeichAI/Claude-Opus-4.6-Reasoning-887x`
  - `open-thoughts/OpenThoughts3-1.2M`
  - `mrm8488/FineTome-single-turn`

## Implementation Steps

1. Build a unified schema for workflow traces.
2. Build the round-1 mixture:
   - `45%` Opus/Sonnet distill
   - `30%` open reasoning/code/math
   - `25%` non-reasoning/chat quality
3. Mix reasoning distill with runtime-style traces.
4. Train response-only `LoRA/QLoRA`.
5. Evaluate on agent-style tasks.

## Success Criteria

- The model uses evidence better and is better at course-correcting when context changes.

## Next Steps

- Move into preference + verifier RL.
