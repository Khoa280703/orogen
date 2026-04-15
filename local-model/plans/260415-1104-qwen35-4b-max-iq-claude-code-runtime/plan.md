---
title: "Qwen3.5-4B master plan under Claude Code runtime"
description: "Unified master plan for dataset curation, SFT, continual pretraining, verifier RL, RAG, and reflection loops for Qwen3.5-4B under Claude Code runtime."
status: in_progress
priority: P1
effort: multi-stage
branch: n/a
tags: [qwen, qwen3.5-4b, claude-code, rag, verifier, continual-pretraining]
created: 2026-04-15
---

# Plan

## Goal
Make `Qwen3.5-4B` as strong as possible inside a system that already has `Claude Code` runtime and tool-use, by optimizing `reasoning quality`, `grounding`, `workflow competence`, `RAG`, and `verifier feedback`.

## Decision

This is the **primary plan going forward**.

Plan [260415-1057-qwen35-4b-opus-distill-rag/plan.md](/home/khoa2807/working-sources/duanai/local-model/plans/260415-1057-qwen35-4b-opus-distill-rag/plan.md) is kept as a reference document for:

- dataset shortlist
- `SFT + RAG` baseline
- round-1 data mixture

## Phases

- `Phase 01` [in_progress, English-first 40-case snapshot after final plan-repair-001 verifier-phrasing refinement]: [phase-01-build-agent-style-eval.md](./phase-01-build-agent-style-eval.md)
- `Phase 02` [pending]: [phase-02-continual-pretraining-corpus.md](./phase-02-continual-pretraining-corpus.md)
- `Phase 03` [pending]: [phase-03-workflow-trace-sft.md](./phase-03-workflow-trace-sft.md)
- `Phase 04` [pending]: [phase-04-preference-and-verifier-rl.md](./phase-04-preference-and-verifier-rl.md)
- `Phase 05` [pending]: [phase-05-rag-and-grounding-stack.md](./phase-05-rag-and-grounding-stack.md)
- `Phase 06` [pending]: [phase-06-serving-critique-revise-loop.md](./phase-06-serving-critique-revise-loop.md)

## Dependencies

- Strategy report: [research-260415-1104-qwen35-4b-max-iq-claude-code-runtime.md](../../reports/research-260415-1104-qwen35-4b-max-iq-claude-code-runtime.md)
- Earlier data shortlist: [research-260415-1057-qwen35-4b-opus-distill-rag.md](../../reports/research-260415-1057-qwen35-4b-opus-distill-rag.md)
- Superseded practical baseline plan: [260415-1057-qwen35-4b-opus-distill-rag/plan.md](/home/khoa2807/working-sources/duanai/local-model/plans/260415-1057-qwen35-4b-opus-distill-rag/plan.md)

## Unified Strategy

Unified priority order:

1. `Build eval first`
2. `Curate dataset mixture`
3. `Continual pretraining`
4. `Workflow-trace SFT`
5. `Preference tuning + verifier RL`
6. `RAG + reranker + grounding`
7. `Serving reflection loop`

Meaning:

- plan `1057` covers the practical near-term pieces: dataset, SFT, RAG
- plan `1104` covers the higher ceiling: correct eval, continual pretraining, verifier, reflection

## Immediate Next Milestones

1. Expand `Phase 01` from the current `40` cases toward a full suite of `40-100` cases per bucket.
2. Tighten the remaining heuristic oracle logic, especially lexical coverage in `research-synthesis` and the remaining verifier phrasing gaps in `plan-repair`.
3. Finalize the round-1 dataset mixture from the shortlist.
4. Select the first continual pretraining corpus.
5. Write the `QLoRA/LoRA` config for the SFT baseline.
6. Design the baseline `RAG + reranker` stack.

## Current Progress

- `Phase 01` is now beyond the vertical slice and has an expanded suite that runs end-to-end:
  - harness: `scripts/run-qwen35-agent-style-eval.py`
  - manifest: `evals/qwen35-agent-style-eval-cases.json`
  - bucket files: `evals/agent-style-eval/{coding-fix,code-understanding,grounded-docs-qa,research-synthesis,plan-repair}.jsonl`
  - scorecard schema: `evals/agent-style-eval/scorecard-schema.json`
  - usage docs: `README.md`
- The expanded suite currently has:
  - `5` buckets
  - `40` cases
  - English-first prompts and checks
  - `case-level dimension_checks`
  - validated `scorecard`
  - both `summary.dimension_scores` and `summary.dimension_proxies` in the report
  - `generated_at` in the report
  - validation that blocks mismatches between `manifest` and `scorecard.bucket_dimensions`
  - validation that now enforces full mapped-dimension coverage per case
- Current expanded-suite baseline after the latest final `plan-repair-001` verifier-phrasing refinement:
  - `--validate-only`: pass
  - live eval through gateway `:8004`: `35/40 = 0.875`
  - latest report timestamp: `2026-04-15T07:33:06.280203+00:00`
  - bucket summary:
    - `coding-fix`: `8/8`
    - `code-understanding`: `7/8`
    - `grounded-docs-qa`: `8/8`
    - `research-synthesis`: `6/8`
    - `plan-repair`: `6/8`
  - `dimension_scores`:
    - `correctness`: `0.9`
    - `groundedness`: `1.0`
    - `concision`: `0.9583`
    - `verifier_pass_rate`: `0.875`
    - `citation_faithfulness`: `1.0`
  - earlier false-red cleanup was kept in part, especially around concision budgets and less brittle lexical matching, but the final refinement for `plan-repair-001` preferred stricter verifier-step semantics over preserving a higher-looking pass rate
  - that last refinement kept the same `35/40` overall score while improving the semantics of the verifier step for `plan-repair-001`, so pre-remediation verification phrasing is less likely to count as a true post-remediation verifier step
  - this number should be read as the current baseline for the expanded English-first suite after the latest oracle pass; it is not directly comparable to the older Vietnamese-oriented snapshot and does not mean the model weights regressed or improved on their own
- The current state confirms the eval harness + manifest + bucket files are more usable as an internal baseline:
  - the earlier `proxy-only` limitation is largely addressed because `dimension_scores` now prioritizes per-case `dimension_checks`, while `dimension_proxies` is retained only for backward comparison
  - major false-green and false-red issues called out in earlier review rounds were reduced materially, although not eliminated; `plan-repair` is now less vulnerable to weak step-list false-green because verifier-oriented step structure stayed strict after the final `plan-repair-001` phrasing refinement
  - the validator now enforces full mapped-dimension coverage per case for this suite, which makes silent proxy fallback much less likely in the current manifest
  - residual heuristics still remain, especially lexical coverage gaps in `research-synthesis`, remaining verifier phrasing gaps in `plan-repair`, and the text-level heuristic nature of `verifier_pass_rate`, so `Phase 01` is not yet strict enough to act as the final quality gate for every large training round

## Success Criteria

- The model improves clearly on agent-style eval.
- Grounded docs QA and coding tasks improve clearly.
- Hallucination drops when evidence is available.
- The `tuned + RAG + verifier + reflection` system is clearly better than `tuned-only`.
