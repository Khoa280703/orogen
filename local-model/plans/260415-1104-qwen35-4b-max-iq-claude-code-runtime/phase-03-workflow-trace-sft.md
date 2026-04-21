# Phase 03: Workflow-trace SFT

## Overview

- Priority: P1
- Status: in_progress
- Approximate phase progress: `~78%`
- Goal: teach the model to operate well in a runtime where tools are already available.
- Current gate: the repaired `wave-4` exploit batch is now complete on healthy `GPU1/GPU2`, and the next decision is whether `Phase 03` branches to a genuinely new recipe/data move because the longer repaired exploit run still did not beat `wave1-gpu0`.

## Key Insights

- It is not necessary to overfit on pure tool-schema samples.
- The model needs training traces that look like observe -> plan -> edit -> verify.
- For a `4B` model, the biggest avoidable mistake is committing early to one recipe and then training it for a long time.
- With `3 GPUs`, the correct default is `parallel search`, not `premature distributed training`.

## Requirements

- Data should include:
  - reasoning distill
  - coding/debugging traces
  - grounded QA
  - concise answer quality
- The phase must support two separate modes:
  - `search mode`: many shorter runs, one per GPU
  - `exploit mode`: one longer winning run after eval selection
- Inherit the shortlist from plan `1057`:
  - `Farseen0/opus-4.6-reasoning-sft-12k`
  - `Roman1111111/claude-opus-4.6-10000x`
  - `nohurry/Opus-4.6-Reasoning-3000x-filtered`
  - `TeichAI/Claude-Opus-4.6-Reasoning-887x`
  - `open-thoughts/OpenThoughts3-1.2M`
  - `mrm8488/FineTome-single-turn`

## Progress Update

1. Added `corpora/qwen35-workflow-trace-sft-round-1-manifest.json`.
2. Added `scripts/qwen35_workflow_trace_sft_manifest_utils.py`.
3. Added `scripts/validate-qwen35-workflow-trace-sft-manifest.py`.
4. Added `scripts/build-qwen35-workflow-trace-sft-source-metadata.py`.
5. Added `scripts/check-qwen35-workflow-trace-sft-runtime.py`.
6. Added `configs/qwen35-workflow-trace-sft-round-1.json`.
7. Built `output/qwen35-workflow-trace-sft-manifest-summary.json`.
8. The original planning manifest defines:
   - `6` candidate datasets
   - `24k` target examples
   - `96M` target-token budget in the original planning manifest
   - `8192` max sequence length
   - response-only training enabled
9. The original round-1 planning mixture is:
   - `45%` distill
   - `30%` open reasoning
   - `25%` chat quality
10. Built `output/qwen35-workflow-trace-sft-source-metadata.json`.
11. The metadata lane now checks Hugging Face dataset-server for:
   - dataset validity
   - split/config availability
   - row counts
   - first-row feature schema
12. The current metadata snapshot confirms:
   - `6` datasets reachable
   - `messages` schema for `Farseen0`, `Roman1111111`, and `TeichAI`
   - `problem-solution-reasoning` schema for `nohurry`
   - `conversations` schema for `OpenThoughts3`
   - `prompt-response` schema for `FineTome-single-turn`
13. Built `output/qwen35-workflow-trace-sft-runtime-check.json`.
14. Added `scripts/setup-qwen35-workflow-trace-sft-runtime.sh`.
15. Added `scripts/qwen35_workflow_trace_sft_shard_utils.py`.
16. Added `scripts/build-qwen35-workflow-trace-sft-shard.py`.
17. A fresh import check still fails under `/usr/bin/python3`, but the saved runtime artifact `output/qwen35-workflow-trace-sft-runtime-check.json` records the passing dedicated runtime snapshot under `./.venv-qwen35-sft/bin/python`.
18. The saved dedicated runtime snapshot is:
   - `torch = 2.10.0+cu128`
   - `datasets = 4.8.4`
   - `transformers = 5.5.4`
   - `peft = 0.19.0`
   - `trl = 1.1.0`
   - `accelerate = 1.13.0`
   - `bitsandbytes = 0.49.2`
19. Built `output/qwen35-workflow-trace-sft-shard.jsonl`.
20. Built `output/qwen35-workflow-trace-sft-shard-summary.json`.
21. The original planning shard materializes:
   - `25,232` deduplicated examples
   - `12,232` distill examples
   - `7,000` open-reasoning examples
   - `6,000` chat-quality examples
   - `19,041` reasoning-bearing examples
   - `106,713,880` estimated tokens by the current heuristic
22. Two distill sources saturated below their requested caps after deduplication:
   - `nohurry/Opus-4.6-Reasoning-3000x-filtered`: `783 / 2,000`
   - `TeichAI/Claude-Opus-4.6-Reasoning-887x`: `449 / 800`
23. Added `scripts/qwen35_workflow_trace_sft_train_utils.py`.
24. Added `scripts/run-qwen35-workflow-trace-sft.py`.
25. The first Phase 03 QLoRA dry-run now passes with:
   - `32` sampled rows
   - `1` optimizer step
   - `2048` max sequence length
   - `train_loss = 1.0050`
   - `train_runtime = 8.00s`
26. The dry-run writes a real adapter slice to `output/qwen35-workflow-trace-sft-round-1-lora-dry-run/`.
27. `scripts/run-qwen35-workflow-trace-sft.py` now uses round-config defaults unless `--dry-run` is explicitly set, so it can act as both a validation runner and a real train entrypoint.
28. The prompt-completion builder now skips rows where the final assistant turn is not terminal, instead of trying to train on trailing multi-assistant traces.
29. The prompt-completion builder now also filters rows whose tokenized `prompt` is not a stable prefix of tokenized `prompt + completion`.
30. For sampled dry-runs, the builder now uses stable sampling across the full shard instead of stopping early on the first clean rows.
31. The latest sampled dry-run report now records:
   - `33` rows seen
   - `32` rows kept
   - `1` skipped prefix mismatch
   - `scan_completed = 0`
32. A follow-up sampled prefix probe reached `64` clean rows after scanning `66` rows, with `2` prefix mismatches skipped in that first-source slice.
33. The latest sampled dry-run no longer emitted the earlier prompt-prefix mismatch warning in the trainer log, so the next real concern moved from sampled-boundary correctness to full-shard gating and effective-mixture quality.
34. The runtime setup now pins the current known-good trainer stack versions by default.
35. `scripts/run-qwen35-4b-vllm.sh` now defaults `VLLM_USE_FLASHINFER_SAMPLER=0` because stale flashinfer cached-op paths were breaking cluster restarts after workspace drift.
36. Added `scripts/audit-qwen35-workflow-trace-sft-prefix.py` to run a reproducible full-shard prefix audit and write a separate JSON gate report.
37. Added `configs/qwen35-workflow-trace-sft-search-wave-1.json`.
38. Added `scripts/run-qwen35-workflow-trace-sft-search-wave.py` to launch `GPU0/GPU1/GPU2` search lanes as separate pinned processes with per-lane logs and reports.
39. `scripts/run-qwen35-workflow-trace-sft.py` now accepts config overrides so search lanes can vary a small set of high-impact settings without cloning near-identical base configs.
40. Historical pre-repair note: before the later `chat_quality` repair overwrote the same mutable output paths, the first full-shard prefix audit reported:
   - `25,232` rows seen
   - `14,476` clean rows kept
   - `10,756` prefix mismatches filtered out
41. That historical pre-repair audit also showed a major effective-mixture drift:
   - `chat-quality-guardrail`: `6,000 / 6,000` rows filtered
   - `open-thoughts-breadth`: `4,565 / 7,000` rows filtered
42. The search-wave launcher now validates that the base config, shard, and runner exist before reporting `ok = true`.
43. Each search lane now pins a physical GPU through `CUDA_VISIBLE_DEVICES`, while the masked process itself trains with `runner_gpu_id = 0` to avoid invalid-device failures on `GPU1` and `GPU2`.
44. Added `scripts/probe-qwen35-workflow-trace-sft-source-recovery.py` to measure whether the two drift-heavy sources are structurally recoverable before changing the recipe.
45. The first historical recovery probe initially reported:
   - `chat-quality-guardrail`: `6,000 / 6,000` rows are plain-answer rows and empty-think wrapping recovers `0`
   - `open-thoughts-breadth`: `2,435` rows are already balanced `<think>...</think>`, while appending `</think>` recovers the other `4,565`
46. The shared train conversion path now auto-closes assistant responses that start with `<think>` but omit `</think>`, so OpenThoughts repair is applied consistently in audit, shard selection, and training.
47. Added `scripts/rebalance-qwen35-workflow-trace-sft-manifest.py` and the first pre-repair search-ready manifest:
   - `corpora/qwen35-workflow-trace-sft-round-1-search-ready-manifest.json`
   - `output/qwen35-workflow-trace-sft-search-ready-manifest-summary.json`
48. That first pre-repair search-ready manifest targeted `17,500` examples with:
   - `103,310,643` target-token budget
   - `10,500` distill
   - `7,000` open reasoning
   - `0` chat quality in wave-1 because the current source is still structurally incompatible
49. Added `scripts/build-qwen35-workflow-trace-sft-search-ready-shard.py` and materialized the first pre-repair post-probe shard:
   - `output/qwen35-workflow-trace-sft-search-ready-shard.jsonl`
   - `output/qwen35-workflow-trace-sft-search-ready-shard-summary.json`
50. That first pre-repair search-ready shard materialized `17,500` rows and wrote the repaired OpenThoughts rows back into the final JSONL instead of relying on train-time repair only.
51. The first pre-repair search-ready shard summary reported:
   - `rows_seen = 25,232`
   - `rows_written = 17,500`
   - `skipped_inactive_source = 6,000`
   - `skipped_over_cap = 1,564`
   - `skipped_prefix_mismatch = 168`
   - `estimated_tokens = 102,320,294`
   - `opus46-reasoning-core = 5,169`
   - `opus46-volume-topup = 4,135`
   - `opus46-filtered-topup = 783`
   - `small-tool-trace-topup = 413`
   - `open-thoughts-breadth = 7,000`
52. The first pre-repair search-ready prefix audit also passed cleanly on that smaller shard:
   - `17,500 / 17,500` rows kept
   - `0` prefix mismatches
53. `scripts/run-qwen35-workflow-trace-sft.py` and `scripts/audit-qwen35-workflow-trace-sft-prefix.py` now resolve `base_model_path`, `tokenizer_path`, and `chat_template_path` relative to `local-model/`, so real runs no longer depend on the caller `cwd`.
54. The first real `wave-1` search run is now complete on the search-ready shard:
   - `output/qwen35-workflow-trace-sft-search-wave-1-gpu0-report.json`
   - `output/qwen35-workflow-trace-sft-search-wave-1-gpu1-report.json`
   - `output/qwen35-workflow-trace-sft-search-wave-1-gpu2-report.json`
55. The train-loss snapshot for that real wave is:
   - `gpu0-baseline-2048`: `train_loss = 0.9334`
   - `gpu2-low-lr-2048`: `train_loss = 0.9359`
   - `gpu1-longer-context-3072`: `train_loss = 1.0026`
56. The official winner is selected by the eval harness, not by train loss:
   - baseline gateway model before wave-1: `35/48 = 0.7292`
   - `gpu0-baseline-2048`: `36/48 = 0.7500`
   - `gpu1-longer-context-3072`: `34/48 = 0.7083`
   - `gpu2-low-lr-2048`: `34/48 = 0.7083`
57. The winning wave-1 recipe is therefore `gpu0-baseline-2048`:
   - it is the only lane that beats baseline
   - it preserves the already-strong buckets
   - it improves `plan-repair` from `6/12` to `7/12`
58. `scripts/run-qwen35-workflow-trace-sft-search-wave.py` is now hardened after the first real run:
   - `ignored_process_names = ["sunshine"]` can ignore local desktop capture noise during exclusivity checks
   - ignored processes still count toward real VRAM pressure, so false-green launch is less likely
   - the launcher is now fail-closed for the whole wave instead of partial-launching only some lanes
   - lane summaries now record `physical_gpu_id` plus masked runner semantics explicitly
   - `validate-only` now reports validation state separately from execution state
   - saved pre-wave-4 validate snapshot from the post-wave-2 launcher hardening pass: `output/qwen35-workflow-trace-sft-search-wave-1-validate-20260418.json`
   - corrupt lane reports no longer break summary generation
59. Historical note after the wave-launcher hardening: before the repaired exploit batch took over `GPU1/GPU2`, the launcher correctly reported expected preflight failure under `--validate-only` when eval servers were still attached to `GPU0/GPU1/GPU2`; this became an accurate runtime signal rather than a misleading green status.
60. The next `wave-2` batch is now launched as an exploit-biased confirmation batch rather than a brand-new broad search:
   - config: `configs/qwen35-workflow-trace-sft-search-wave-2.json`
   - `gpu0-promote-winner-2048-epoch15`: promote the wave-1 winner into a longer full-shard run
   - `gpu1-replica-winner-2048-epoch15`: exact-replica confirmation lane with only the seed changed
   - `gpu2-neighbor-mid-lr-2048-epoch15`: near-neighbor confirmation lane with only a slightly calmer learning rate
61. `scripts/run-qwen35-workflow-trace-sft-search-wave.py` now accepts an optional per-lane `seed`, so confirmation lanes can test recipe stability without forking the runner or cloning another launcher.
62. The operational goal of `wave-2` is no longer “find any winner”; it is “verify that the wave-1 winner is reproducible enough to justify the next very long exploit run”.
63. Historical live-ops note for `wave-2`:
   - while the batch was still running, the stable source of truth was the launcher process plus the runtime directory under `output/qwen35-workflow-trace-sft-search-wave-2-runtime/`
   - `summary_output` only became the finished batch snapshot after launcher exit
64. During live monitoring, `wave-2` stopped being a clean `3/3` confirmation batch very early:
   - `gpu1-replica-winner-2048-epoch15` continued on `GPU1`
   - `gpu2-neighbor-mid-lr-2048-epoch15` continued on `GPU2`
   - `gpu0-promote-winner-2048-epoch15` crashed before its first checkpoint/report JSON
65. The original `gpu0` lane failed at the first backward step with:
   - `torch.AcceleratorError: CUDA error: an illegal memory access was encountered`
   - lane log: `output/qwen35-workflow-trace-sft-search-wave-2-gpu0-report.log`
66. The strongest available infrastructure signal confirms that this is likely a `GPU0`-local incident rather than a recipe-wide regression:
   - saved kernel-log excerpt: `output/qwen35-workflow-trace-sft-search-wave-2-gpu0-kernel-xid31.log`
   - kernel log excerpt shows `NVRM: Xid 31 ... pid=304049 ... MMU Fault` at the timestamp of the failed `gpu0` lane
   - `gpu1` was an exact-recipe confirmation lane with only the seed changed and still finished cleanly
   - `gpu2` also finished cleanly on the same code path and same search-ready shard
67. A manual `GPU0` retry and a short diagnostic rerun were both de-prioritized operationally:
   - the detached retry did not produce useful forward progress
   - a foreground diagnostic run showed that the empty log was not a launch bug by itself; it was still in CPU-side dataset/tokenization work when interrupted for traceback
   - the practical conclusion is still to quarantine `GPU0` for the remainder of the current batch instead of burning time on blind repeats
68. The active post-wave-2 Phase 03 operating policy is now:
   - treat `GPU0` as an infrastructure incident pending reset/reboot/display-detach decision
   - keep `wave1-gpu0` as the incumbent recipe/checkpoint
   - only rerun a third confirmation lane after `GPU0` has been explicitly reset, and only if extra confirmation is still worth the cost
69. `wave-2` is now fully complete:
   - batch summary: `output/qwen35-workflow-trace-sft-search-wave-2-summary.json`
   - eval reports:
     - `output/qwen35-agent-style-eval-wave2-gpu1-report.json`
     - `output/qwen35-agent-style-eval-wave2-gpu2-report.json`
   - comparison target:
     - `output/qwen35-agent-style-eval-wave1-gpu0-report.json`
70. The final post-wave-2 eval outcome is:
   - `wave2-gpu1`: `33/48 = 0.6875`
   - `wave2-gpu2`: `34/48 = 0.7083`
   - baseline pre-wave-1: `35/48 = 0.7292`
   - `wave1-gpu0` incumbent: `36/48 = 0.7500`
71. `wave2-gpu2` is the internal winner of `wave-2`, but the batch fails its real objective:
   - it beats `wave2-gpu1`
   - it does not beat baseline
   - it does not match or beat the `wave-1` incumbent
72. The bucket picture confirms that `wave-2` did not buy a meaningful upgrade:
   - both wave-2 lanes preserve the already-strong buckets
   - `wave2-gpu2` keeps `research-synthesis` flat at `6/12`
   - both wave-2 lanes regress `plan-repair` to `5/12`
   - the earlier `wave-1` gain in `plan-repair` was therefore not confirmed by this exploit-biased batch
   - the eval-dimension picture is even clearer:
     - `wave1-gpu0` keeps `verifier_pass_rate = 0.75`
     - both `wave2-gpu1` and `wave2-gpu2` regress `verifier_pass_rate` to `0.5833`
     - this is a stronger reason not to promote `wave-2` than train loss by itself
73. The practical decision after `wave-2` is now:
   - keep `wave1-gpu0` as the current incumbent recipe/checkpoint
   - mark `wave-2` as a failed confirmation batch, not a promotion event
   - do not open the long exploit run from a `wave-2` lane
   - decide next between:
     - rerunning confirmation from the `wave-1` incumbent after a real `GPU0` reset
     - or promoting the `wave-1` incumbent directly into the next exploit stage
74. The first post-`wave-2` corrective action now targets the missing `chat_quality` lane instead of another blind exploit run:
   - `scripts/qwen35_workflow_trace_sft_train_utils.py` now renders non-reasoning rows with `enable_thinking = false`
   - this preserves the Qwen chat template for reasoning rows while making plain-answer rows prefix-safe again
75. A direct current probe now confirms that `chat-quality-guardrail` is no longer dead under the prefix gate:
   - focused probe: `output/qwen35-workflow-trace-sft-source-recovery-probe-chat-quality.json`
   - current result: `6,000 / 6,000` rows prefix-safe
   - the earlier `0 / 6,000` failure mode was therefore a rendering-path problem, not a data-supply problem
76. The search-ready manifest has now been rebuilt with `chat_quality` restored:
   - summary: `output/qwen35-workflow-trace-sft-search-ready-manifest-summary.json`
   - total target examples: `23,333`
   - group split:
     - `distill = 10,500`
     - `open_reasoning = 7,000`
     - `chat_quality = 5,833`
77. The search-ready shard has now been rebuilt from that repaired manifest:
   - summary: `output/qwen35-workflow-trace-sft-search-ready-shard-summary.json`
   - rows written: `23,333`
   - `skipped_prefix_mismatch = 0`
   - source split now includes `chat-quality-guardrail = 5,833`
   - follow-up audit: `output/qwen35-workflow-trace-sft-search-ready-prefix-audit.json`
   - audit result: `23,333 / 23,333` rows kept with `0` prefix mismatches
78. The healthy-GPU follow-up search is now complete on the repaired shard while `GPU0` stays quarantined:
   - config: `configs/qwen35-workflow-trace-sft-search-wave-3-chat-quality-repair.json`
   - lanes:
     - `gpu1-chat-repair-baseline-2048`
     - `gpu2-chat-repair-mid-lr-2048`
   - train reports:
     - `output/qwen35-workflow-trace-sft-search-wave-3-gpu1-report.json`
     - `output/qwen35-workflow-trace-sft-search-wave-3-gpu2-report.json`
   - summary:
     - `output/qwen35-workflow-trace-sft-search-wave-3-chat-quality-repair-summary.json`
   - train-loss snapshot:
     - `gpu1`: `0.8936`
     - `gpu2`: `0.8911`
   - serving note:
     - the repaired adapters are `LoRA rank 64`, so vLLM serving must include `--max-lora-rank 64`
   - eval reports:
     - `output/qwen35-agent-style-eval-wave3-gpu1-report.json`
     - `output/qwen35-agent-style-eval-wave3-gpu2-report.json`
   - eval outcome:
     - baseline pre-wave-1: `35/48 = 0.7292`
     - incumbent `wave1-gpu0`: `36/48 = 0.7500`
     - `wave3-gpu1`: `36/48 = 0.7500`
     - `wave3-gpu2`: `36/48 = 0.7500`
   - the two repaired lanes match the incumbent on the main practical gate:
     - same overall score: `36/48 = 0.7500`
     - same case-level fail set
     - same bucket picture: `coding-fix = 8/8`, `code-understanding = 7/8`, `grounded-docs-qa = 8/8`, `research-synthesis = 6/12`, `plan-repair = 7/12`
     - same `verifier_pass_rate = 0.75`
     - but not full dimension-score parity: `wave1-gpu0` keeps slightly higher `correctness`
   - practical decision after `wave-3`:
     - keep `wave1-gpu0` as the formal incumbent because `wave-3` does not beat it
     - treat the repaired recipe as a healthy reproduced tie on `GPU1/GPU2`
     - open the next longer exploit batch from the repaired shard on healthy GPUs instead of waiting for another short-run tiebreaker
79. The repaired exploit batch is now complete on healthy GPUs only:
   - config: `configs/qwen35-workflow-trace-sft-search-wave-4-repaired-exploit.json`
   - lanes:
     - `gpu1-repaired-baseline-2048-epoch15`
     - `gpu2-repaired-midlr-2048-epoch15`
   - train reports:
     - `output/qwen35-workflow-trace-sft-search-wave-4-gpu1-report.json`
     - `output/qwen35-workflow-trace-sft-search-wave-4-gpu2-report.json`
   - summary:
     - `output/qwen35-workflow-trace-sft-search-wave-4-repaired-exploit-summary.json`
   - long-run train snapshot:
     - `gpu1`: `train_loss = 0.7879`
     - `gpu2`: `train_loss = 0.7921`
   - eval reports:
     - `output/qwen35-agent-style-eval-wave4-gpu1-report.json`
     - `output/qwen35-agent-style-eval-wave4-gpu2-report.json`
   - eval outcome:
     - baseline pre-wave-1: `35/48 = 0.7292`
     - incumbent `wave1-gpu0`: `36/48 = 0.7500`
     - `wave4-gpu1`: `36/48 = 0.7500`
     - `wave4-gpu2`: `36/48 = 0.7500`
   - practical decision after `wave-4`:
     - the longer repaired exploit run confirms the repaired recipe can hold incumbent-level quality over a long run on healthy GPUs
     - the two `wave-4` lanes still match `wave1-gpu0` on the main practical gate rather than beating it
     - `wave1-gpu0` remains the formal incumbent
     - the next `Phase 03` move should be a real branch change instead of another same-family exploit repeat

## Related Code Files

- Modified:
  - `README.md`
  - `plans/260415-1104-qwen35-4b-max-iq-claude-code-runtime/plan.md`
  - `plans/260415-1104-qwen35-4b-max-iq-claude-code-runtime/phase-03-workflow-trace-sft.md`
- Created:
  - `corpora/qwen35-workflow-trace-sft-round-1-manifest.json`
  - `corpora/qwen35-workflow-trace-sft-round-1-search-ready-manifest.json`
  - `configs/qwen35-workflow-trace-sft-round-1.json`
  - `scripts/qwen35_workflow_trace_sft_manifest_utils.py`
  - `scripts/validate-qwen35-workflow-trace-sft-manifest.py`
  - `scripts/build-qwen35-workflow-trace-sft-source-metadata.py`
  - `scripts/check-qwen35-workflow-trace-sft-runtime.py`
  - `scripts/setup-qwen35-workflow-trace-sft-runtime.sh`
  - `scripts/audit-qwen35-workflow-trace-sft-prefix.py`
  - `scripts/probe-qwen35-workflow-trace-sft-source-recovery.py`
  - `scripts/rebalance-qwen35-workflow-trace-sft-manifest.py`
  - `scripts/qwen35_workflow_trace_sft_shard_utils.py`
  - `scripts/build-qwen35-workflow-trace-sft-shard.py`
  - `scripts/build-qwen35-workflow-trace-sft-search-ready-shard.py`
  - `scripts/qwen35_workflow_trace_sft_train_utils.py`
  - `scripts/run-qwen35-workflow-trace-sft.py`
  - `scripts/run-qwen35-workflow-trace-sft-search-wave.py`
  - `scripts/run-qwen35-4b-vllm.sh`
  - `configs/qwen35-workflow-trace-sft-search-wave-1.json`
  - `configs/qwen35-workflow-trace-sft-search-wave-2.json`
  - `output/qwen35-workflow-trace-sft-manifest-summary.json`
  - `output/qwen35-workflow-trace-sft-source-metadata.json`
  - `output/qwen35-workflow-trace-sft-source-recovery-probe.json`
  - `output/qwen35-workflow-trace-sft-search-ready-manifest-summary.json`
  - `output/qwen35-workflow-trace-sft-runtime-check.json`
  - `output/qwen35-workflow-trace-sft-shard.jsonl`
  - `output/qwen35-workflow-trace-sft-shard-summary.json`
  - `output/qwen35-workflow-trace-sft-search-ready-shard.jsonl`
  - `output/qwen35-workflow-trace-sft-search-ready-shard-summary.json`
  - `output/qwen35-workflow-trace-sft-search-ready-prefix-audit.json`
  - `output/qwen35-workflow-trace-sft-dry-run-report.json`
  - `output/qwen35-workflow-trace-sft-round-1-lora-dry-run/`

## Implementation Steps

1. Build a unified schema for workflow traces.
2. Build the round-1 mixture:
   - `45%` Opus/Sonnet distill
   - `30%` open reasoning/code/math
   - `25%` non-reasoning/chat quality
3. Mix reasoning distill with runtime-style traces.
4. Install or isolate the training runtime required by the baseline config.
5. Materialize the first normalized SFT shard from the approved sources.
6. Run a first small response-only `LoRA/QLoRA` dry-run.
7. Tighten prompt/completion boundary handling for the full run.
8. Define the first parallel search matrix across `GPU0/GPU1/GPU2`, varying only a few high-impact dimensions:
   - mixture weights
   - reasoning-vs-grounding balance
   - sequence length
   - optimization settings that materially change learning dynamics
9. Probe any heavy prefix failures before deciding whether a source is dead or just structurally malformed.
10. Build a search-ready manifest from audit plus recovery results.
11. Run multiple shorter SFT experiments in parallel on the search-ready shard and compare them with the same eval harness.
12. Select exactly one winning recipe based on eval deltas, not training loss.
13. Launch a longer exploit run only after the search wave identifies a clear winner.
14. Evaluate the winning adapter on agent-style tasks before moving into preference/verifier tuning.

## Success Criteria

- The model uses evidence better and is better at course-correcting when context changes.
- The round-1 SFT manifest is validated and its mixture weights are internally consistent.
- The dataset metadata lane confirms that every selected source is reachable and mapped to a known normalization strategy.
- A baseline train config exists for response-only `QLoRA`.
- A dedicated runtime exists and passes import checks before the first actual SFT run starts.
- A normalized shard exists for the round-1 mixture with provenance, dedupe stats, and per-source coverage.
- A real 1-step QLoRA dry-run passes on local hardware and produces an adapter artifact.
- At least one search-wave recipe produces a meaningful eval gain over the current baseline.
- Long exploit training is reserved for recipes that already win under the eval harness.

## Next Steps

- Keep the eval harness fixed at the current `48`-case English-first suite so future exploit and confirmation runs are compared against the same gate.
- Record `wave-2` as a failed confirmation batch and keep `wave1-gpu0` as the incumbent until a stronger candidate exists.
- Finish the repaired exploit batch and check whether either longer run beats `wave1-gpu0` instead of merely matching it.
- Move into preference + verifier RL only after the next promoted checkpoint beats the current `36/48` `wave1-gpu0` snapshot by a more convincing margin.
- Operator-ready `wave-3` eval sequence if the repaired adapters need to be re-served later:
  - serve `gpu1` adapter:
    - `QWEN35_GPU_DEVICES=1 QWEN35_PORT=8101 QWEN35_SERVED_MODEL_NAME=qwen35-wave3-gpu1 QWEN35_OUTPUT_DIR=./output/qwen35-wave3-gpu1-vllm ./scripts/start-qwen35-4b-vllm.sh --enable-lora --max-lora-rank 64 --lora-modules qwen35-wave3-gpu1=./output/qwen35-workflow-trace-sft-search-wave-3-gpu1-adapter`
  - serve `gpu2` adapter:
    - `QWEN35_GPU_DEVICES=2 QWEN35_PORT=8102 QWEN35_SERVED_MODEL_NAME=qwen35-wave3-gpu2 QWEN35_OUTPUT_DIR=./output/qwen35-wave3-gpu2-vllm ./scripts/start-qwen35-4b-vllm.sh --enable-lora --max-lora-rank 64 --lora-modules qwen35-wave3-gpu2=./output/qwen35-workflow-trace-sft-search-wave-3-gpu2-adapter`
  - eval `gpu1` lane:
    - `python3 ./scripts/run-qwen35-agent-style-eval.py --base-url http://127.0.0.1:8101 --model qwen35-wave3-gpu1 --output ./output/qwen35-agent-style-eval-wave3-gpu1-report.json`
  - eval `gpu2` lane:
    - `python3 ./scripts/run-qwen35-agent-style-eval.py --base-url http://127.0.0.1:8102 --model qwen35-wave3-gpu2 --output ./output/qwen35-agent-style-eval-wave3-gpu2-report.json`
  - compare against:
    - `output/qwen35-agent-style-eval-report.json`
    - `output/qwen35-agent-style-eval-wave1-gpu0-report.json`
    - `output/qwen35-agent-style-eval-wave2-gpu1-report.json`
    - `output/qwen35-agent-style-eval-wave2-gpu2-report.json`
