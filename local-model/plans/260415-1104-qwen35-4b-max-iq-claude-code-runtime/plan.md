---
title: "Qwen3.5-4B extreme-max-IQ plan under Claude Code runtime"
description: "Unified extreme-optimization plan for month-scale dataset curation, continual pretraining, parallel SFT search, verifier tuning, RAG, and reflection loops for Qwen3.5-4B under Claude Code runtime."
status: stopped
priority: P1
effort: multi-stage
branch: n/a
tags: [qwen, qwen3.5-4b, claude-code, rag, verifier, continual-pretraining]
created: 2026-04-15
---

# Plan

## Goal
Make `Qwen3.5-4B` materially smarter inside a system that already has `Claude Code` runtime and tool-use, with no short-term wall-clock constraint, by maximizing `reasoning quality`, `grounding`, `workflow competence`, `verifier-backed reliability`, `RAG quality`, and `test-time compute`.

## Decision

This plan is no longer active.

Reset note:

- reset by operator on `2026-04-19`
- do not continue this plan
- keep this file only as a historical research record
- serve and benchmark against the base `Qwen3.5-4B` model unless a new plan explicitly says otherwise

This plan is now explicitly in `extreme optimization` mode:

- optimize for capability gain, not calendar speed
- assume a `weeks-to-months` horizon is acceptable
- treat `3x RTX 3090` as a research cluster, not just a serving cluster
- prefer `parallel search -> select -> long exploit` over a single early distributed run
- use multi-GPU single-run training only when the stage is large enough to justify the added complexity

Plan [260415-1057-qwen35-4b-opus-distill-rag/plan.md](/home/khoa2807/working-sources/duanai/local-model/plans/260415-1057-qwen35-4b-opus-distill-rag/plan.md) is kept as a reference document for:

- dataset shortlist
- `SFT + RAG` baseline
- round-1 data mixture

## Phases

- `Phase 01` [in_progress, still not strong enough to gate expensive month-scale experiments]: [phase-01-build-agent-style-eval.md](./phase-01-build-agent-style-eval.md)
- `Phase 02` [in_progress, strongest real-IQ lane and likely the first stage that may justify true multi-GPU single-run training]: [phase-02-continual-pretraining-corpus.md](./phase-02-continual-pretraining-corpus.md)
- `Phase 03` [in_progress, `wave-2` failed confirmation, the repaired `wave-3` matched the incumbent on the practical gate, and the longer repaired `wave-4` exploit batch completed without beating `wave1-gpu0`]: [phase-03-workflow-trace-sft.md](./phase-03-workflow-trace-sft.md)
- `Phase 04` [pending, becomes mandatory once a strong SFT checkpoint exists]: [phase-04-preference-and-verifier-rl.md](./phase-04-preference-and-verifier-rl.md)
- `Phase 05` [pending, grounding is a first-class capability lane rather than an optional add-on]: [phase-05-rag-and-grounding-stack.md](./phase-05-rag-and-grounding-stack.md)
- `Phase 06` [pending, latency is explicitly secondary to capability]: [phase-06-serving-critique-revise-loop.md](./phase-06-serving-critique-revise-loop.md)

## Dependencies

- Strategy report: [research-260415-1104-qwen35-4b-max-iq-claude-code-runtime.md](../reports/research-260415-1104-qwen35-4b-max-iq-claude-code-runtime.md)
- Earlier data shortlist: [research-260415-1057-qwen35-4b-opus-distill-rag.md](../reports/research-260415-1057-qwen35-4b-opus-distill-rag.md)
- Superseded practical baseline plan: [260415-1057-qwen35-4b-opus-distill-rag/plan.md](/home/khoa2807/working-sources/duanai/local-model/plans/260415-1057-qwen35-4b-opus-distill-rag/plan.md)

## Unified Strategy

Unified priority order:

1. `Build eval first`
2. `Make eval hard enough to reject fake gains`
3. `Curate high-leverage datasets`
4. `Run continual pretraining where it buys real prior`
5. `Run parallel workflow-trace SFT search`
6. `Exploit the winning recipe with longer runs`
7. `Add preference tuning + verifier RL`
8. `Add RAG + reranker + grounding`
9. `Add serving reflection loop`

Meaning:

- plan `1057` covers the practical near-term pieces: dataset, SFT, RAG
- plan `1104` covers the higher ceiling: correct eval, continual pretraining, verifier, reflection
- in this updated version, `speed` is subordinate to `capability gain per month of compute`

## Compute Policy

Use the `3x RTX 3090` cluster in two modes only:

1. `Search mode`
   - preferred default for `QLoRA/SFT` and recipe selection
   - `GPU0`, `GPU1`, `GPU2` run separate experiments in parallel
   - purpose: compare mixtures, hyperparameters, and curricula quickly enough to avoid wasting long runs on weak recipes
2. `Exploit mode`
   - used only after eval clearly identifies a winning recipe
   - for `LoRA/QLoRA`, usually keep `1 GPU = 1 long run` unless a larger distributed setup proves necessary
   - for `continual pretraining` or other genuinely large runs, using multiple GPUs in one job becomes justified

Rule:

- do not spend all `3 GPUs` on one small `4B QLoRA` run just because the hardware exists
- do spend all `3 GPUs` on one run once the stage is bottlenecked by sequence length, token budget, or full-parameter-style compute

## Immediate Next Milestones

1. Finish hardening `Phase 01` until it can reject weak but plausible-looking tuning gains.
2. Decide whether `Phase 02` needs issue-lane promotion, curriculum repair, or another proxy benchmark after the first CPT adapter regressed on the current agent-style suite.
3. Keep `wave1-gpu0` as the incumbent because `wave-2` failed confirmation against both the pre-wave baseline and the incumbent itself.
4. Close out the repaired exploit branch after confirming that the longer `wave-4` run still only matches the incumbent on the practical gate.
5. Decide whether the next `Phase 02` move is issue-lane promotion, a longer CPT curriculum, or a benchmark-only gate before touching more expensive lanes.
6. Prepare `Phase 04` so verifier-backed tuning can start immediately after the first strong SFT checkpoint exists.
7. Treat `Phase 05` and `Phase 06` as mandatory capability multipliers, not optional polish.

## Current Progress

- Approximate overall plan progress is now around `43%`:
  - `Phase 01` is usable but not yet a final hard gate
  - `Phase 02` now has real corpus assets, a completed first CPT pilot, and a completed first downstream benchmark, but the current adapter regressed on the present suite and is not promoted
  - `Phase 03` has now completed `wave-1`, `wave-2`, the repaired `wave-3`, and the longer repaired `wave-4`, but still has no checkpoint that formally beats `wave1-gpu0`
  - `Phase 04` to `Phase 06` are still effectively pending

- `Phase 01` is now beyond the vertical slice and has an expanded suite that runs end-to-end:
  - harness: `scripts/run-qwen35-agent-style-eval.py`
  - manifest: `evals/qwen35-agent-style-eval-cases.json`
  - bucket files: `evals/agent-style-eval/{coding-fix,code-understanding,grounded-docs-qa,research-synthesis,plan-repair}.jsonl`
  - scorecard schema: `evals/agent-style-eval/scorecard-schema.json`
  - usage docs: `README.md`
- The expanded suite currently has:
  - `5` buckets
  - `48` cases
  - English-first prompts and checks
  - `case-level dimension_checks`
  - validated `scorecard`
  - both `summary.dimension_scores` and `summary.dimension_proxies` in the report
  - `generated_at` in the report
  - validation that blocks mismatches between `manifest` and `scorecard.bucket_dimensions`
  - validation that now enforces full mapped-dimension coverage per case
- Current expanded-suite baseline after the weakest-bucket expansion to `12` `research-synthesis` and `12` `plan-repair` cases:
  - `--validate-only`: pass
  - live eval through gateway `:8004`: `35/48 = 0.7292`
  - latest report timestamp: `2026-04-15T08:14:08.739454+00:00`
  - bucket summary:
    - `coding-fix`: `8/8`
    - `code-understanding`: `7/8`
    - `grounded-docs-qa`: `8/8`
    - `research-synthesis`: `6/12`
    - `plan-repair`: `6/12`
  - `dimension_scores`:
    - `correctness`: `0.75`
    - `groundedness`: `1.0`
    - `concision`: `0.875`
    - `verifier_pass_rate`: `0.6667`
    - `citation_faithfulness`: `1.0`
  - the absolute pass count stayed at `35`, but the pass rate fell because the suite now probes broader trade-off articulation and post-remediation verification behavior in the two weakest buckets
  - the new fail pattern is still mostly meaningful rather than obviously brittle: many failures omit an explicit downside in `research-synthesis` or omit a real verify step in `plan-repair`
  - this number should be read as the current baseline for the expanded English-first suite after the latest oracle pass; it is not directly comparable to the earlier `35/40` snapshot and does not mean the model weights regressed on their own
- The current state confirms the eval harness + manifest + bucket files are more usable as an internal baseline:
  - the earlier `proxy-only` limitation is largely addressed because `dimension_scores` now prioritizes per-case `dimension_checks`, while `dimension_proxies` is retained only for backward comparison
  - major false-green and false-red issues called out in earlier review rounds were reduced materially, although not eliminated; `plan-repair` is now less vulnerable to weak step-list false-green because verifier-oriented step structure stayed strict after the final `plan-repair-001` phrasing refinement and the later weakest-bucket expansion
  - the validator now enforces full mapped-dimension coverage per case for this suite, which makes silent proxy fallback much less likely in the current manifest
  - residual heuristics still remain, especially lexical coverage gaps in `research-synthesis`, remaining verifier phrasing gaps in `plan-repair`, and the text-level heuristic nature of `verifier_pass_rate`, so `Phase 01` is not yet strict enough to act as the final quality gate for every large training round
- `Phase 02` now has its first executable scaffolding:
  - corpus manifest: `corpora/qwen35-continual-pretraining-round-1-corpus-manifest.json`
  - validator/summarizer: `scripts/validate-qwen35-continual-pretraining-corpus.py`
  - current summary output: `output/qwen35-continual-pretraining-corpus-summary.json`
  - external metadata builder: `scripts/build-qwen35-external-cpt-metadata.py`
  - external approval builder: `scripts/approve-qwen35-external-cpt-sources.py`
  - external shard builder: `scripts/build-qwen35-external-cpt-shard.py`
  - local shard builder: `scripts/build-qwen35-local-cpt-shard.py`
  - mixed shard builder: `scripts/build-qwen35-mixed-cpt-shard.py`
  - current external review outputs:
    - `output/qwen35-external-cpt-pinned-manifest-candidate.json`
    - `output/qwen35-external-cpt-metadata-shard.jsonl`
    - `output/qwen35-external-cpt-metadata-summary.json`
  - current external approval/fetch outputs:
    - `output/qwen35-external-cpt-approved-manifest.json`
    - `output/qwen35-external-cpt-approval-summary.json`
    - `output/qwen35-external-cpt-shard.jsonl`
    - `output/qwen35-external-cpt-shard-summary.json`
  - current issue candidate outputs:
    - `output/qwen35-issue-cpt-candidate-shard.jsonl`
    - `output/qwen35-issue-cpt-candidate-summary.json`
  - current local shard outputs:
    - `output/qwen35-local-cpt-shard.jsonl`
    - `output/qwen35-local-cpt-shard-summary.json`
  - current mixed shard outputs:
    - `output/qwen35-mixed-cpt-shard.jsonl`
    - `output/qwen35-mixed-cpt-shard-summary.json`
  - current validated round-1 plan:
    - `8` sources
    - `120M` estimated tokens
    - `4` internal local sources with recorded workspace references
    - `4` public sources marked `review-required` and `pending-pin-before-fetch`
  - current external metadata review lane:
    - `4` public sources
    - `14` resolved public references
    - `13` strong references
    - `1` weak reference
    - `2` metadata documents that passed the current quality gate
    - candidate pins written without flipping `ready = true`
  - current approval/fetch lane:
    - `2` public sources approved for fetch
    - `2` public sources still pending review
    - `321` normalized documents fetched into the first external shard
    - approved source split:
      - `qwen-official-repositories-and-docs`: `155` documents
      - `vllm-and-transformers-runtime-stack`: `166` documents
  - current issue candidate lane:
    - `18` normalized issue documents materialized from closed GitHub issue searches
    - source split:
      - `vllm-project/vllm`: `6` documents
      - `huggingface/transformers`: `6` documents
      - `QwenLM/Qwen3`: `6` documents
    - this lane remains separate from the main approved external shard pending quality review
  - current local scan:
    - `185` matched product-code files
    - `24` matched local-model runtime/serving files
    - `60` matched docs/plan files
    - `20` matched logs/benchmark files
  - the validator now enforces minimum local match counts, surfaces pending provenance explicitly, and reports cross-source local overlap if it appears
  - the local shard builder now materializes `269` normalized internal seed documents before any public-source fetch
  - mutable runtime-output sources remain in the full manifest, but are excluded from the default local seed shard to reduce drift
  - the shard summary records the live git snapshot used during materialization; the current run is still from a dirty workspace, so it should be read as a working-tree seed shard rather than a fully pinned export
  - the current mixed shard now materializes `590` deduplicated documents from the current trusted local seed and the approved external shard
  - `Phase 02` now also has a completed first CPT pilot:
    - config: `configs/qwen35-continual-pretraining-pilot-round-1.json`
    - runner: `scripts/run-qwen35-continual-pretraining.py`
    - dry-run report: `output/qwen35-continual-pretraining-pilot-round-1-dry-run-report.json`
    - full pilot report: `output/qwen35-continual-pretraining-pilot-round-1-report.json`
    - adapter output: removed during reset on `2026-04-19`
  - the completed CPT pilot used the full mixed shard on `1x RTX 3090` and finished `3.0` epochs / `147` steps with:
    - `588` kept documents
    - `429` total token blocks
    - `386` train blocks
    - `43` holdout blocks
    - `train_loss = 0.6869`
    - final `eval_loss = 0.7410`
    - best checkpoint: `checkpoint-147`
  - the holdout loss improved monotonically across the saved eval checkpoints:
    - `0.8197 -> 0.7768 -> 0.7557 -> 0.7421 -> 0.7415 -> 0.7410`
  - `Phase 02` now also has the first downstream CPT benchmark:
    - adapter eval report: `output/qwen35-agent-style-eval-cpt-pilot-round-1-report.json`
    - comparison summary: `output/qwen35-agent-style-eval-cpt-pilot-round-1-summary.json`
  - the current downstream read is a regression on the present suite:
    - pre-CPT baseline: `35/48 = 0.7292`
    - formal incumbent `wave1-gpu0`: `36/48 = 0.7500`
    - CPT pilot adapter: `33/48 = 0.6875`
  - the regression is concentrated in reasoning-heavy buckets:
    - `research-synthesis`: `6/12 -> 4/12`
    - `plan-repair`: `6/12 -> 6/12` versus baseline, still below the incumbent `7/12`
    - case delta versus baseline:
      - improved: `plan-repair-001`
      - regressed: `plan-repair-003`, `research-synthesis-001`, `research-synthesis-007`
  - this means `Phase 02` is now training-ready end to end for the trusted-core curriculum, but the current adapter is not promotion-ready; the next move has to be a curriculum/data/benchmark branch change, not a blind longer exploit of the same CPT recipe
- `Phase 03` now has a materialized workflow-trace SFT baseline:
  - SFT manifest: `corpora/qwen35-workflow-trace-sft-round-1-manifest.json`
  - manifest validator: `scripts/validate-qwen35-workflow-trace-sft-manifest.py`
  - metadata builder: `scripts/build-qwen35-workflow-trace-sft-source-metadata.py`
  - runtime checker: `scripts/check-qwen35-workflow-trace-sft-runtime.py`
  - runtime setup: `scripts/setup-qwen35-workflow-trace-sft-runtime.sh`
  - shard builder: `scripts/build-qwen35-workflow-trace-sft-shard.py`
  - dry-run trainer: `scripts/run-qwen35-workflow-trace-sft.py`
  - shard utils: `scripts/qwen35_workflow_trace_sft_shard_utils.py`
  - shared manifest utils: `scripts/qwen35_workflow_trace_sft_manifest_utils.py`
  - baseline train config: `configs/qwen35-workflow-trace-sft-round-1.json`
  - current outputs:
    - `output/qwen35-workflow-trace-sft-manifest-summary.json`
    - `output/qwen35-workflow-trace-sft-source-metadata.json`
    - `output/qwen35-workflow-trace-sft-runtime-check.json`
    - `output/qwen35-workflow-trace-sft-shard.jsonl`
    - `output/qwen35-workflow-trace-sft-shard-summary.json`
    - `output/qwen35-workflow-trace-sft-dry-run-report.json`
  - original validated round-1 SFT planning manifest:
    - `6` candidate datasets
    - `24k` target examples
    - `96M` target-token budget in the original planning manifest
    - mixture:
      - `45%` distill
      - `30%` open reasoning
      - `25%` chat quality
  - the metadata lane now verifies first-row schema and dataset size through Hugging Face dataset-server before any full materialization
  - a fresh import check under `/usr/bin/python3` still fails, but the saved runtime artifact `output/qwen35-workflow-trace-sft-runtime-check.json` records the passing dedicated runtime snapshot under `./.venv-qwen35-sft/bin/python`
  - the saved dedicated runtime snapshot is:
    - `torch = 2.10.0+cu128`
    - `datasets = 4.8.4`
    - `transformers = 5.5.4`
    - `peft = 0.19.0`
    - `trl = 1.1.0`
    - `accelerate = 1.13.0`
    - `bitsandbytes = 0.49.2`
  - the first real `wave-1` search run is now complete as of `2026-04-17`:
    - lane reports:
      - `output/qwen35-workflow-trace-sft-search-wave-1-gpu0-report.json`
      - `output/qwen35-workflow-trace-sft-search-wave-1-gpu1-report.json`
      - `output/qwen35-workflow-trace-sft-search-wave-1-gpu2-report.json`
    - train-loss snapshot:
      - `gpu0-baseline-2048`: `0.9334`
      - `gpu2-low-lr-2048`: `0.9359`
      - `gpu1-longer-context-3072`: `1.0026`
  - the winning recipe is now selected by the eval harness rather than by train loss:
    - baseline gateway model before wave-1: `35/48 = 0.7292`
    - `gpu0-baseline-2048`: `36/48 = 0.7500`
    - `gpu1-longer-context-3072`: `34/48 = 0.7083`
    - `gpu2-low-lr-2048`: `34/48 = 0.7083`
    - the only actual gain over baseline comes from `gpu0`, driven by `plan-repair` improving from `6/12` to `7/12`
  - the search-wave launcher is now hardened for future waves:
    - `ignored_process_names = ["sunshine"]` is supported for exclusivity checks without hiding real VRAM pressure
    - `validate-only` no longer reports top-level success as if the wave already executed
    - execution is now fail-closed at the whole-wave level: if one lane fails preflight, no lane starts
    - lane summaries now expose both `physical_gpu_id` and masked runner semantics
    - corrupt per-lane report JSON no longer crashes summary generation
  - `wave-2` is now complete as an exploit-biased confirmation batch:
    - config: `configs/qwen35-workflow-trace-sft-search-wave-2.json`
    - operator runtime dir for detached launcher bookkeeping: `output/qwen35-workflow-trace-sft-search-wave-2-runtime/`
    - lane split:
      - `gpu0-promote-winner-2048-epoch15`
      - `gpu1-replica-winner-2048-epoch15`
    - `gpu2-neighbor-mid-lr-2048-epoch15`
    - purpose: verify that the `wave-1` gain is reproducible enough to justify a single much longer exploit run
    - summary: `output/qwen35-workflow-trace-sft-search-wave-2-summary.json`
  - the final `wave-2` train/eval snapshot is now:
    - comparison reports:
      - baseline: `output/qwen35-agent-style-eval-report.json`
      - incumbent: `output/qwen35-agent-style-eval-wave1-gpu0-report.json`
      - `wave2-gpu1`: `output/qwen35-agent-style-eval-wave2-gpu1-report.json`
      - `wave2-gpu2`: `output/qwen35-agent-style-eval-wave2-gpu2-report.json`
    - `gpu0-promote-winner-2048-epoch15`: failed before writing a lane report JSON
    - `gpu1-replica-winner-2048-epoch15`: train ok, eval `33/48 = 0.6875`
    - `gpu2-neighbor-mid-lr-2048-epoch15`: train ok, eval `34/48 = 0.7083`
    - `wave-2` best lane is therefore `gpu2`, but it still underperforms both baseline `35/48` and the `wave-1` incumbent `36/48`
    - the dimension regression is explicit in the eval evidence:
      - baseline `verifier_pass_rate = 0.6667`
      - `wave1-gpu0` `verifier_pass_rate = 0.75`
      - `wave2-gpu1` `verifier_pass_rate = 0.5833`
      - `wave2-gpu2` `verifier_pass_rate = 0.5833`
    - the promotion decision is therefore based on eval regressions, not on train-loss impressions or the incomplete `gpu0` training lane alone
  - the strongest current evidence points to a `GPU0` infrastructure incident rather than a recipe-wide failure:
    - lane log: `output/qwen35-workflow-trace-sft-search-wave-2-gpu0-report.log`
    - saved kernel-log excerpt: `output/qwen35-workflow-trace-sft-search-wave-2-gpu0-kernel-xid31.log`
    - kernel log excerpt: `NVRM: Xid 31 ... pid=304049 ... MMU Fault`
    - exact-recipe confirmation on `GPU1` finished cleanly, so the `GPU0` crash does not by itself invalidate the `wave-1` winner recipe
  - the current operating rule is therefore:
    - keep `wave1-gpu0` as the current incumbent
    - record `wave-2` as a failed confirmation batch rather than a promotion event
    - quarantine `GPU0` from blind retries until an explicit `GPU0` reset / reboot / display detachment decision exists
    - only re-run the missing confirmation lane if additional confirmation is still worth the cost after deciding the next exploit strategy
  - the original planning shard for Phase 03 materializes `25,232` deduplicated examples:
    - `12,232` distill
    - `7,000` open reasoning
    - `6,000` chat quality
    - `19,041` reasoning-bearing examples
    - `106,713,880` estimated tokens by the current heuristic
  - the first QLoRA dry-run now passes on `1x RTX 3090`:
    - `32` sampled rows
    - `1` optimizer step
    - `2048` max sequence length
    - `train_loss = 1.0050`
    - `train_runtime = 8.00s`
  - `scripts/run-qwen35-workflow-trace-sft.py` now honors round-config defaults unless `--dry-run` is set explicitly
  - the prompt-completion builder now skips rows where the final assistant turn is not terminal
  - the prompt-completion builder now filters token-level prompt-prefix mismatches before rows reach `TRL`
  - the latest sampled dry-run report records:
    - `33` rows seen
    - `32` rows kept
    - `1` skipped prefix mismatch
    - `scan_completed = 0`
  - a dedicated full-shard prefix-audit script now exists so this boundary can be checked outside the train path
  - historical pre-repair note: before the later `chat_quality` repair overwrote the same mutable output paths, the first full-shard prefix audit reported:
    - `25,232` rows seen
    - `14,476` clean rows kept before recovery probing
    - `10,756` prefix mismatches filtered out before recovery probing
  - the first historical recovery probe explained the drift more precisely:
    - `chat-quality-guardrail` initially stayed dead under the then-current thinking template:
      - `6,000 / 6,000` rows are plain-answer rows
      - empty-think wrapping recovers `0`
    - `open-thoughts-breadth` is mostly recoverable:
      - `2,435` rows are already balanced `<think>...</think>`
      - `4,565` rows start with `<think>` and only miss the closing tag
      - appending `</think>` recovers those `4,565` rows under the same prefix gate
  - the training conversion path now repairs assistant replies that start with `<think>` but omit `</think>`, which lifts the effective clean supply without inventing reasoning for chat-only rows
  - the first pre-repair post-probe search-ready manifest now existed with:
    - `corpora/qwen35-workflow-trace-sft-round-1-search-ready-manifest.json`
    - `output/qwen35-workflow-trace-sft-search-ready-manifest-summary.json`
    - `17,500` target examples
    - `103,310,643` target-token budget
    - active group split:
      - `10,500` distill
      - `7,000` open reasoning
      - `0` chat quality in wave-1
  - the first pre-repair post-probe search-ready shard then existed with:
    - `output/qwen35-workflow-trace-sft-search-ready-shard.jsonl`
    - `output/qwen35-workflow-trace-sft-search-ready-shard-summary.json`
    - `17,500` rows written
    - source split:
      - `opus46-reasoning-core = 5,169`
      - `opus46-volume-topup = 4,135`
      - `opus46-filtered-topup = 783`
      - `small-tool-trace-topup = 413`
      - `open-thoughts-breadth = 7,000`
    - `6,000` inactive `chat_quality` rows excluded from the wave
    - `168` prefix mismatches rejected while selecting the final capped shard
    - `102,320,294` estimated tokens in the final shard
  - that first pre-repair search-ready prefix audit also passed cleanly on the smaller shard:
    - `17,500 / 17,500` rows kept
    - `0` prefix mismatches
  - the first `GPU0/GPU1/GPU2` search-wave scaffold now exists as a matrix config plus a pinned multi-process launcher
  - the launcher now validates preconditions before reporting success, uses the search-ready shard by default, and avoids invalid-device errors by pinning physical GPUs outside the process while training on masked `gpu_id = 0` inside each lane
  - historical pre-launch blocker note:
    - before the eval servers were torn down, an earlier `search-wave --validate-only` snapshot returned `gpu_preflight_ok = false` because `VLLM::EngineCore` was still attached to the three training GPUs
  - historical pre-wave-4 snapshot:
    - `output/qwen35-workflow-trace-sft-search-wave-1-validate-20260418.json` records `gpu_preflight_ok = true`
    - that snapshot was the evidence that `GPU1/GPU2` were available again before the repaired exploit batch was launched
    - `GPU0` remains strategically quarantined because of the earlier `Xid 31` incident, not because of generic cluster-wide VRAM scarcity
  - the immediate post-wave-2 fix now targets data quality rather than another blind exploit:
    - `scripts/qwen35_workflow_trace_sft_train_utils.py` now renders non-reasoning rows with `enable_thinking = false`
    - focused probe `output/qwen35-workflow-trace-sft-source-recovery-probe-chat-quality.json` now confirms `chat-quality-guardrail = 6000/6000` prefix-safe
  - the repaired search-ready assets are now materially stronger:
    - manifest summary: `output/qwen35-workflow-trace-sft-search-ready-manifest-summary.json`
    - shard summary: `output/qwen35-workflow-trace-sft-search-ready-shard-summary.json`
    - prefix audit: `output/qwen35-workflow-trace-sft-search-ready-prefix-audit.json`
    - repaired shard size: `23,333` rows with `chat_quality = 5,833` restored and `0` prefix mismatches under both shard build and follow-up audit
  - the healthy-GPU follow-up search is now complete from that repaired shard:
    - config: `configs/qwen35-workflow-trace-sft-search-wave-3-chat-quality-repair.json`
    - lanes: `gpu1-chat-repair-baseline-2048`, `gpu2-chat-repair-mid-lr-2048`
    - `GPU0` remains out of the batch by design
    - train reports:
      - `output/qwen35-workflow-trace-sft-search-wave-3-gpu1-report.json`
      - `output/qwen35-workflow-trace-sft-search-wave-3-gpu2-report.json`
    - summary:
      - `output/qwen35-workflow-trace-sft-search-wave-3-chat-quality-repair-summary.json`
    - train-loss snapshot:
      - `gpu1`: `0.8936`
      - `gpu2`: `0.8911`
    - vLLM serving note:
      - the repaired adapters are `LoRA rank 64`, so serving must include `--max-lora-rank 64`
    - eval reports:
      - `output/qwen35-agent-style-eval-wave3-gpu1-report.json`
      - `output/qwen35-agent-style-eval-wave3-gpu2-report.json`
    - eval outcome:
      - baseline: `35/48 = 0.7292`
      - incumbent `wave1-gpu0`: `36/48 = 0.7500`
      - `wave3-gpu1`: `36/48 = 0.7500`
      - `wave3-gpu2`: `36/48 = 0.7500`
    - the repaired recipe therefore clears the “no regression on healthy GPUs” bar and matches the incumbent on the main practical gate, but it does not produce a strict short-run win over the incumbent and still trails slightly on full `correctness`
  - the next exploit batch is now also complete from that repaired recipe:
    - config: `configs/qwen35-workflow-trace-sft-search-wave-4-repaired-exploit.json`
    - lanes:
      - `gpu1-repaired-baseline-2048-epoch15`
      - `gpu2-repaired-midlr-2048-epoch15`
    - train reports:
      - `output/qwen35-workflow-trace-sft-search-wave-4-gpu1-report.json`
      - `output/qwen35-workflow-trace-sft-search-wave-4-gpu2-report.json`
    - summary:
      - `output/qwen35-workflow-trace-sft-search-wave-4-repaired-exploit-summary.json`
    - train-loss snapshot:
      - `gpu1`: `0.7879`
      - `gpu2`: `0.7921`
    - eval reports:
      - `output/qwen35-agent-style-eval-wave4-gpu1-report.json`
      - `output/qwen35-agent-style-eval-wave4-gpu2-report.json`
    - eval outcome:
      - baseline: `35/48 = 0.7292`
      - incumbent `wave1-gpu0`: `36/48 = 0.7500`
      - `wave4-gpu1`: `36/48 = 0.7500`
      - `wave4-gpu2`: `36/48 = 0.7500`
    - practical read:
      - the longer repaired exploit run confirms the repaired recipe is stable over a long run on healthy `GPU1/GPU2`
      - but it still does not produce a strict win over `wave1-gpu0`
      - `wave1-gpu0` therefore remains the formal incumbent
  - the main train runner plus the audit entrypoint now canonicalize model/tokenizer/template paths relative to `local-model/`, so the actual run path no longer depends on the caller `cwd`
  - the runtime setup now pins the current known-good trainer stack versions by default
  - the shard build surfaced two practical source ceilings after deduplication:
    - `nohurry/Opus-4.6-Reasoning-3000x-filtered`: `783/2000`
    - `TeichAI/Claude-Opus-4.6-Reasoning-887x`: `449/800`
  - the dry-run does write a real LoRA adapter artifact, but this is still only a runtime-validation slice, not the full planned SFT round
  - the latest sampled dry-run no longer emitted the earlier prompt-prefix mismatch warning in the trainer log; that old launch blocker is now historical because multiple real search and exploit waves have already run
  - `scripts/run-qwen35-4b-vllm.sh` now defaults `VLLM_USE_FLASHINFER_SAMPLER=0` to avoid flashinfer cached-op restart failures caused by stale absolute paths in the JIT workspace

## Success Criteria

- The model improves `materially`, not cosmetically, on the agent-style eval.
- The weakest buckets (`research-synthesis`, `plan-repair`) improve by enough margin that the difference is robust to a harder oracle.
- Grounded docs QA and coding tasks improve clearly.
- Hallucination drops clearly when evidence is available.
- The best `tuned + RAG + verifier + reflection` stack is clearly better than `tuned-only`.
- Every major long run is justified by earlier search results rather than intuition alone.
