# Qwen3.5-4B Workspace

Reset status:

- the experimental `Phase 02 CPT` adapter lane was reset on `2026-04-19`
- use the base model in `models/qwen3.5-4b` as the active model
- the CPT adapter directories were removed; historical JSON reports are kept only as audit logs

This workspace is dedicated to the `Qwen3.5-4B` workflow on `3x RTX 3090`.

Main components:

- local model: `models/qwen3.5-4b`
- 3 vLLM replicas:
  - `GPU0 -> :8100`
  - `GPU1 -> :8101`
  - `GPU2 -> :8102`
- gateway scheduler: `:8004`
- local chat UI: `:8010`
- CCS profile: `qwen-local`

Model aliases through the gateway:

- `qwen3.5-4b-thinking`
- `qwen3.5-4b-no-thinking`
- `qwen3.5-4b`

Default CCS mapping:

- `Opus -> qwen3.5-4b-thinking`
- `Sonnet -> qwen3.5-4b-thinking`
- `Haiku -> qwen3.5-4b-no-thinking`

Main commands:

```bash
./scripts/start-qwen35-4b-cluster.sh
./scripts/stop-qwen35-4b-cluster.sh
./scripts/reset-qwen35-4b-cluster.sh
./scripts/setup-qwen35-ccs-profile.sh
./scripts/smoke-test-qwen35-4b-vllm.sh
```

Benchmark and eval:

```bash
python3 ./scripts/benchmark-qwen35-4b-cluster.py --base-url http://127.0.0.1:8004
python3 ./scripts/run-qwen35-agent-style-eval.py --validate-only
python3 ./scripts/run-qwen35-agent-style-eval.py --base-url http://127.0.0.1:8004 --model qwen3.5-4b
```

Continual pretraining corpus planning:

```bash
python3 ./scripts/validate-qwen35-continual-pretraining-corpus.py \
  --manifest ./corpora/qwen35-continual-pretraining-round-1-corpus-manifest.json \
  --output ./output/qwen35-continual-pretraining-corpus-summary.json

python3 ./scripts/build-qwen35-local-cpt-shard.py \
  --manifest ./corpora/qwen35-continual-pretraining-round-1-corpus-manifest.json \
  --output-jsonl ./output/qwen35-local-cpt-shard.jsonl \
  --output-summary ./output/qwen35-local-cpt-shard-summary.json

python3 ./scripts/build-qwen35-external-cpt-metadata.py \
  --manifest ./corpora/qwen35-continual-pretraining-round-1-corpus-manifest.json \
  --output-manifest ./output/qwen35-external-cpt-pinned-manifest-candidate.json \
  --output-jsonl ./output/qwen35-external-cpt-metadata-shard.jsonl \
  --output-summary ./output/qwen35-external-cpt-metadata-summary.json

python3 ./scripts/approve-qwen35-external-cpt-sources.py \
  --manifest ./output/qwen35-external-cpt-pinned-manifest-candidate.json \
  --output-manifest ./output/qwen35-external-cpt-approved-manifest.json \
  --output-summary ./output/qwen35-external-cpt-approval-summary.json \
  --approve-source qwen-official-repositories-and-docs \
  --approve-source vllm-and-transformers-runtime-stack

python3 ./scripts/build-qwen35-external-cpt-shard.py \
  --manifest ./output/qwen35-external-cpt-approved-manifest.json \
  --output-jsonl ./output/qwen35-external-cpt-shard.jsonl \
  --output-summary ./output/qwen35-external-cpt-shard-summary.json

python3 ./scripts/build-qwen35-mixed-cpt-shard.py \
  --local-jsonl ./output/qwen35-local-cpt-shard.jsonl \
  --external-jsonl ./output/qwen35-external-cpt-shard.jsonl \
  --output-jsonl ./output/qwen35-mixed-cpt-shard.jsonl \
  --output-summary ./output/qwen35-mixed-cpt-shard-summary.json

python3 ./scripts/build-qwen35-issue-cpt-candidate-shard.py \
  --manifest ./output/qwen35-external-cpt-pinned-manifest-candidate.json \
  --output-jsonl ./output/qwen35-issue-cpt-candidate-shard.jsonl \
  --output-summary ./output/qwen35-issue-cpt-candidate-summary.json

# Historical Phase 02 commands only. The adapter directories from this lane
# were removed during the reset on 2026-04-19.
CUDA_VISIBLE_DEVICES=1 ./.venv-qwen35-sft/bin/python ./scripts/run-qwen35-continual-pretraining.py \
  --dry-run \
  --config ./configs/qwen35-continual-pretraining-pilot-round-1.json \
  --output-dir ./output/qwen35-continual-pretraining-pilot-round-1-dry-run \
  --report-output ./output/qwen35-continual-pretraining-pilot-round-1-dry-run-report.json \
  --sample-limit 32 \
  --max-steps 1 \
  --max-sequence-length 512 \
  --gpu-id 0

CUDA_VISIBLE_DEVICES=1 ./.venv-qwen35-sft/bin/python ./scripts/run-qwen35-continual-pretraining.py \
  --config ./configs/qwen35-continual-pretraining-pilot-round-1.json \
  --output-dir ./output/qwen35-continual-pretraining-pilot-round-1-lora \
  --report-output ./output/qwen35-continual-pretraining-pilot-round-1-report.json \
  --gpu-id 0 \
  --seed 424242

python3 ./scripts/validate-qwen35-workflow-trace-sft-manifest.py \
  --manifest ./corpora/qwen35-workflow-trace-sft-round-1-manifest.json \
  --output ./output/qwen35-workflow-trace-sft-manifest-summary.json

python3 ./scripts/build-qwen35-workflow-trace-sft-source-metadata.py \
  --manifest ./corpora/qwen35-workflow-trace-sft-round-1-manifest.json \
  --output ./output/qwen35-workflow-trace-sft-source-metadata.json

python3 ./scripts/check-qwen35-workflow-trace-sft-runtime.py \
  --output ./output/qwen35-workflow-trace-sft-runtime-check.json

./scripts/setup-qwen35-workflow-trace-sft-runtime.sh

./.venv-qwen35-sft/bin/python ./scripts/build-qwen35-workflow-trace-sft-shard.py \
  --manifest ./corpora/qwen35-workflow-trace-sft-round-1-manifest.json \
  --metadata ./output/qwen35-workflow-trace-sft-source-metadata.json \
  --output-jsonl ./output/qwen35-workflow-trace-sft-shard.jsonl \
  --output-summary ./output/qwen35-workflow-trace-sft-shard-summary.json

./.venv-qwen35-sft/bin/python ./scripts/audit-qwen35-workflow-trace-sft-prefix.py \
  --config ./configs/qwen35-workflow-trace-sft-round-1.json \
  --train-jsonl ./output/qwen35-workflow-trace-sft-shard.jsonl \
  --output ./output/qwen35-workflow-trace-sft-prefix-audit.json

./.venv-qwen35-sft/bin/python ./scripts/probe-qwen35-workflow-trace-sft-source-recovery.py \
  --config ./configs/qwen35-workflow-trace-sft-round-1.json \
  --manifest ./corpora/qwen35-workflow-trace-sft-round-1-manifest.json \
  --input-jsonl ./output/qwen35-workflow-trace-sft-shard.jsonl \
  --output ./output/qwen35-workflow-trace-sft-source-recovery-probe.json

./.venv-qwen35-sft/bin/python ./scripts/rebalance-qwen35-workflow-trace-sft-manifest.py \
  --base-manifest ./corpora/qwen35-workflow-trace-sft-round-1-manifest.json \
  --audit ./output/qwen35-workflow-trace-sft-prefix-audit.json \
  --recovery-probe ./output/qwen35-workflow-trace-sft-source-recovery-probe.json \
  --output-manifest ./corpora/qwen35-workflow-trace-sft-round-1-search-ready-manifest.json \
  --output-summary ./output/qwen35-workflow-trace-sft-search-ready-manifest-summary.json

./.venv-qwen35-sft/bin/python ./scripts/build-qwen35-workflow-trace-sft-search-ready-shard.py \
  --config ./configs/qwen35-workflow-trace-sft-round-1.json \
  --manifest ./corpora/qwen35-workflow-trace-sft-round-1-search-ready-manifest.json \
  --input-jsonl ./output/qwen35-workflow-trace-sft-shard.jsonl \
  --output-jsonl ./output/qwen35-workflow-trace-sft-search-ready-shard.jsonl \
  --output-summary ./output/qwen35-workflow-trace-sft-search-ready-shard-summary.json

./.venv-qwen35-sft/bin/python ./scripts/run-qwen35-workflow-trace-sft-search-wave.py \
  --config ./configs/qwen35-workflow-trace-sft-search-wave-1.json \
  --validate-only

CUDA_VISIBLE_DEVICES=0 ./.venv-qwen35-sft/bin/python ./scripts/run-qwen35-workflow-trace-sft.py \
  --dry-run \
  --config ./configs/qwen35-workflow-trace-sft-round-1.json \
  --train-jsonl ./output/qwen35-workflow-trace-sft-search-ready-shard.jsonl \
  --output-dir ./output/qwen35-workflow-trace-sft-search-ready-dry-run \
  --report-output ./output/qwen35-workflow-trace-sft-search-ready-dry-run-report.json \
  --sample-limit 32 \
  --max-steps 1 \
  --max-sequence-length 2048 \
  --gpu-id 0
```

Current final Phase 01 snapshot:

- Latest checked report: `output/qwen35-agent-style-eval-report.json`
- `generated_at`: `2026-04-15T08:14:08.739454+00:00`
- Overall: `35/48 = 0.7292`
- Strong buckets: `coding-fix = 8/8`, `grounded-docs-qa = 8/8`, `code-understanding = 7/8`
- Weak buckets: `research-synthesis = 6/12`, `plan-repair = 6/12`

Notes:

- Run from the `local-model/` directory if you want to use default paths.
- The default report path is `output/qwen35-agent-style-eval-report.json`.
- The report always includes `generated_at`; if you want a separate snapshot per run, pass `--output`.
- The runner is resilient per case: if one request fails, the batch still continues and the report is still written.
- `evals/qwen35-agent-style-eval-cases.json` is now the manifest; bucket files live in `evals/agent-style-eval/`.
- The Phase 01 eval suite is now English-first.
- The Phase 01 eval suite currently has `48` cases; the weakest buckets were expanded from `8` to `12` cases each.
- `scorecard-schema.json` is loaded, validated, and included in the report.
- `summary.dimension_scores` is driven by case-level `dimension_checks`; the validator now requires every mapped dimension to be annotated for every case in this suite.
- In the latest final Phase 01 snapshot, `summary.dimension_scores` has full coverage (`coverage = 1.0`) with `proxy_cases = 0` for every reported dimension in this suite.
- `summary.dimension_proxies` is kept only for backward comparison with older snapshots using bucket/case pass-fail; do not use it as the primary read for this suite.
- `summary.overall.pass_rate` is the case-level pass rate; it is not the same thing as a specific dimension such as `correctness` or `concision`.
- `dimension_scores` in the latest final snapshot are: `correctness = 0.75`, `groundedness = 1.0`, `concision = 0.875`, `verifier_pass_rate = 0.6667`, `citation_faithfulness = 1.0`.
- `verifier_pass_rate` is still a text-level heuristic in this suite; it should not be read as a true verifier-backed execution metric.
- The latest `48`-case snapshot keeps the same absolute pass count as the earlier `35/40` snapshot, but the pass rate drops because the suite now probes harder `research-synthesis` and `plan-repair` coverage.
- `plan-repair` is now stricter on verifier-style remediation semantics; read it as a tighter plan-quality screen, not as direct evidence of model regression by itself.
- `Phase 02` now has a validated continual pretraining corpus manifest at `corpora/qwen35-continual-pretraining-round-1-corpus-manifest.json`.
- The current `Phase 02` summary output is `output/qwen35-continual-pretraining-corpus-summary.json`: `8` sources, `120M` estimated tokens, `4` internal sources with recorded workspace references, `4` public sources still marked `review-required` and `pending-pin-before-fetch`.
- `Phase 02` now also has a materialized local seed shard at `output/qwen35-local-cpt-shard.jsonl` with `269` normalized documents and summary stats in `output/qwen35-local-cpt-shard-summary.json`.
- The current local shard summary records the live git snapshot used for materialization; in the current run `workspace_dirty = true`, so this shard should be read as a working-tree seed shard, not a fully pinned reproducible export yet.
- Mutable runtime-output sources remain in the full corpus manifest for later curation, but they are excluded from the default local seed shard build.
- `Phase 02` now also has an external metadata review lane:
  - pinned-manifest candidate: `output/qwen35-external-cpt-pinned-manifest-candidate.json`
  - metadata shard: `output/qwen35-external-cpt-metadata-shard.jsonl`
  - summary: `output/qwen35-external-cpt-metadata-summary.json`
- The current external metadata lane resolved `4` public sources into `14` candidate references (`13` strong, `1` weak) and kept only `2` metadata documents that passed the current quality gate, while still keeping `ready = false` for legal/review gates.
- `Phase 02` now also has an approval lane and first fetched external shard:
  - approved manifest: `output/qwen35-external-cpt-approved-manifest.json`
  - approval summary: `output/qwen35-external-cpt-approval-summary.json`
  - external shard: `output/qwen35-external-cpt-shard.jsonl`
  - external shard summary: `output/qwen35-external-cpt-shard-summary.json`
- The current approval snapshot moves `2` public sources to `approved-for-fetch` and keeps `2` sources pending review:
  - approved: `qwen-official-repositories-and-docs`, `vllm-and-transformers-runtime-stack`
  - pending: `issue-fix-and-troubleshooting-narratives`, `api-design-and-systems-reference-docs`
- The current fetched external shard contains `321` normalized documents from the approved public sources:
  - `qwen-official-repositories-and-docs`: `155` documents
  - `vllm-and-transformers-runtime-stack`: `166` documents
- `Phase 02` now also has a first mixed shard at `output/qwen35-mixed-cpt-shard.jsonl` with `590` deduplicated documents built from `269` local documents and `321` approved external documents.
- `Phase 02` now also has a dedicated issue candidate lane:
  - issue candidate shard: `output/qwen35-issue-cpt-candidate-shard.jsonl`
  - issue candidate summary: `output/qwen35-issue-cpt-candidate-summary.json`
- The current issue candidate snapshot materializes `18` issue documents from closed GitHub issue searches:
  - `6` from `vllm-project/vllm`
  - `6` from `huggingface/transformers`
  - `6` from `QwenLM/Qwen3`
- The issue lane is still a candidate lane only; it is not merged into the main approved external shard yet.
- `Phase 02` now also has a real CPT pilot runner and first completed pilot:
  - config: `configs/qwen35-continual-pretraining-pilot-round-1.json`
  - train utils: `scripts/qwen35_continual_pretraining_train_utils.py`
  - runner: `scripts/run-qwen35-continual-pretraining.py`
  - dry-run report: `output/qwen35-continual-pretraining-pilot-round-1-dry-run-report.json`
  - pilot report: `output/qwen35-continual-pretraining-pilot-round-1-report.json`
  - adapter output: removed during reset on `2026-04-19`
- The completed CPT pilot used the full mixed shard on `1x RTX 3090` with:
  - `588` kept documents
  - `429` token blocks at `2048` max sequence length
  - `386` train blocks
  - `43` holdout blocks
  - `3.0` epochs
  - `147` optimizer steps
  - `train_loss = 0.6869`
  - final `eval_loss = 0.7410`
  - best checkpoint: `checkpoint-147`
- The full pilot holdout loss decreased monotonically across saved eval steps:
  - `step 25`: `0.8197`
  - `step 50`: `0.7768`
  - `step 75`: `0.7557`
  - `step 100`: `0.7421`
  - `step 125`: `0.7415`
  - `step 147`: `0.7410`
- `Phase 02` now also has the first downstream benchmark read on the current CPT pilot adapter:
  - adapter eval report: `output/qwen35-agent-style-eval-cpt-pilot-round-1-report.json`
  - comparison summary: `output/qwen35-agent-style-eval-cpt-pilot-round-1-summary.json`
- The current CPT adapter does **not** beat the pre-CPT baseline on the current `48`-case suite:
  - pre-CPT baseline: `35/48 = 0.7292`
  - formal incumbent `wave1-gpu0`: `36/48 = 0.7500`
  - CPT pilot adapter: `33/48 = 0.6875`
- The regression is concentrated in `research-synthesis`, while `plan-repair` stays level with the pre-CPT baseline and still below the formal incumbent:
  - unchanged strong buckets: `coding-fix = 8/8`, `code-understanding = 7/8`, `grounded-docs-qa = 8/8`
  - reasoning buckets after CPT: `research-synthesis = 4/12`, `plan-repair = 6/12`
  - improved vs baseline: `plan-repair-001`
  - regressed vs baseline: `plan-repair-003`, `research-synthesis-001`, `research-synthesis-007`
- Read this CPT pilot as a confirmed `runtime + train-loss-behavior` success but a current `downstream eval` miss; the present adapter is not yet promote-worthy on the suite.
- `Phase 03` now has a concrete workflow-trace SFT baseline:
  - SFT manifest: `corpora/qwen35-workflow-trace-sft-round-1-manifest.json`
  - train config: `configs/qwen35-workflow-trace-sft-round-1.json`
  - manifest summary: `output/qwen35-workflow-trace-sft-manifest-summary.json`
  - source metadata summary: `output/qwen35-workflow-trace-sft-source-metadata.json`
  - runtime check: `output/qwen35-workflow-trace-sft-runtime-check.json`
  - runtime setup: `scripts/setup-qwen35-workflow-trace-sft-runtime.sh`
  - normalized shard builder: `scripts/build-qwen35-workflow-trace-sft-shard.py`
  - dry-run trainer: `scripts/run-qwen35-workflow-trace-sft.py`
  - normalized shard: `output/qwen35-workflow-trace-sft-shard.jsonl`
  - shard summary: `output/qwen35-workflow-trace-sft-shard-summary.json`
  - dry-run report: `output/qwen35-workflow-trace-sft-dry-run-report.json`
- The original Phase 03 planning manifest defines a `24k`-example round-1 target with:
  - `45%` distill
  - `30%` open reasoning
  - `25%` chat quality
- The current metadata lane resolves `6` candidate SFT datasets and confirms their first-row schema against the normalization plan.
- A fresh import check still fails under `/usr/bin/python3`, but the saved runtime artifact `output/qwen35-workflow-trace-sft-runtime-check.json` records the passing dedicated SFT runtime snapshot under `./.venv-qwen35-sft/bin/python`.
- The saved dedicated SFT runtime snapshot is:
  - `torch = 2.10.0+cu128`
  - `datasets = 4.8.4`
  - `transformers = 5.5.4`
  - `peft = 0.19.0`
  - `trl = 1.1.0`
  - `accelerate = 1.13.0`
  - `bitsandbytes = 0.49.2`
- The original planning-shard materialization for Phase 03 produced `25,232` deduplicated SFT examples with:
  - `12,232` distill examples
  - `7,000` open-reasoning examples
  - `6,000` chat-quality examples
  - `19,041` examples that still contain explicit reasoning traces
  - `106,713,880` estimated tokens by the current char-based heuristic
- That original planning-shard build surfaced two supply limits after deduplication:
  - `nohurry/Opus-4.6-Reasoning-3000x-filtered`: `783` usable examples vs cap `2,000`
  - `TeichAI/Claude-Opus-4.6-Reasoning-887x`: `449` usable examples vs cap `800`
- The first Phase 03 dry-run now passes on `1x RTX 3090` with:
  - `32` sampled rows
  - `1` optimizer step
  - `2048` max sequence length
  - `train_loss = 1.0050`
  - `train_runtime = 8.00s`
- `scripts/run-qwen35-workflow-trace-sft.py` is no longer implicitly dry-run-only:
  - use `--dry-run` for the `32 rows / 1 step / 2048 tokens` validation slice
  - omit `--dry-run` if you actually want it to honor the round config defaults
- The dry-run writes a real LoRA adapter under `output/qwen35-workflow-trace-sft-round-1-lora-dry-run/`.
- The prompt-completion builder now streams the shard, skips non-terminal assistant rows, and filters token-level prompt-prefix mismatches before samples reach `TRL`.
- The latest dry-run report now includes `dataset_build_stats` for the sampled slice:
  - `33` rows seen
  - `32` rows kept
  - `1` skipped prefix mismatch
  - `scan_completed = 0`; the builder still uses stable sampling across the full shard, but the current report does not mark the sampled slice as a completed full scan
- The latest sampled dry-run no longer emitted the earlier prompt-prefix mismatch warning in the trainer log, and the first base full-shard audit now acts only as a pre-recovery diagnostic snapshot.
- `scripts/audit-qwen35-workflow-trace-sft-prefix.py` now exists as a reproducible full-shard prefix gate and writes `output/qwen35-workflow-trace-sft-prefix-audit.json`.
- Historical pre-repair note: before the later `chat_quality` repair overwrote the same mutable output paths, the first full-shard prefix audit reported:
  - `25,232` rows seen
  - `14,476` clean rows kept
  - `10,756` prefix mismatches filtered out
- That historical pre-repair audit showed the original effective-mixture drift:
  - `chat-quality-guardrail`: `6,000 / 6,000` rows filtered
  - `open-thoughts-breadth`: `4,565 / 7,000` rows filtered
- `scripts/probe-qwen35-workflow-trace-sft-source-recovery.py` now writes `output/qwen35-workflow-trace-sft-source-recovery-probe.json` and shows:
  - historical pre-repair snapshot: `chat-quality-guardrail` was `6,000 / 6,000` plain answers and empty-think wrapping recovered `0`
  - `open-thoughts-breadth`: `2,435` rows are already balanced `<think>...</think>`, while appending `</think>` recovers the other `4,565`
- The shared train conversion path now auto-closes assistant outputs that start with `<think>` but omit `</think>`, so the OpenThoughts repair is applied consistently in audit, shard selection, and training.
- `scripts/rebalance-qwen35-workflow-trace-sft-manifest.py` now writes the post-probe search-ready manifest:
  - manifest: `corpora/qwen35-workflow-trace-sft-round-1-search-ready-manifest.json`
  - summary: `output/qwen35-workflow-trace-sft-search-ready-manifest-summary.json`
  - historical pre-repair target: `17,500` examples
  - historical pre-repair target-token budget: `103,310,643`
  - historical pre-repair active split: `10,500` distill, `7,000` open reasoning, `0` chat quality in wave-1
- `scripts/build-qwen35-workflow-trace-sft-search-ready-shard.py` first wrote the pre-repair `search-ready` shard with `17,500` rows:
  - `opus46-reasoning-core = 5,169`
  - `opus46-volume-topup = 4,135`
  - `opus46-filtered-topup = 783`
  - `small-tool-trace-topup = 413`
  - `open-thoughts-breadth = 7,000`
  - selection stats: `skipped_inactive_source = 6,000`, `skipped_over_cap = 1,564`, `skipped_prefix_mismatch = 168`, `estimated_tokens = 102,320,294`
- That first pre-repair search-ready prefix audit reported `17,500 / 17,500` rows kept with `0` prefix mismatches.
- `scripts/run-qwen35-workflow-trace-sft-search-wave.py` plus `configs/qwen35-workflow-trace-sft-search-wave-1.json` now define the first `GPU0/GPU1/GPU2` search-wave scaffold for short parallel SFT experiments.
- The search-wave launcher now validates that the base config, shard, and runner exist before reporting success, and each lane now points at the search-ready shard while still using `CUDA_VISIBLE_DEVICES=<physical-gpu>` plus `runner_gpu_id = 0` inside the masked process.
- The first real `Phase 03` search wave is now complete as of `2026-04-17`:
  - baseline gateway model before wave-1: `35/48 = 0.7292`
  - `gpu0-baseline-2048`: `36/48 = 0.7500`
  - `gpu1-longer-context-3072`: `34/48 = 0.7083`
  - `gpu2-low-lr-2048`: `34/48 = 0.7083`
  - winner: `gpu0-baseline-2048`
  - decisive delta: `plan-repair` improved from `6/12` to `7/12`, while `gpu1` and `gpu2` regressed on that bucket to `5/12`
- The second real `Phase 03` confirmation wave is now complete as of `2026-04-18`:
  - `gpu0-promote-winner-2048-epoch15` did not produce a valid eval candidate because the lane crashed on `GPU0` with `CUDA illegal memory access`; see `output/qwen35-workflow-trace-sft-search-wave-2-gpu0-report.log` plus the saved kernel excerpt `output/qwen35-workflow-trace-sft-search-wave-2-gpu0-kernel-xid31.log`. This should be treated as a `GPU0` runtime incident, not as recipe evidence
  - comparison reports:
    - baseline: `output/qwen35-agent-style-eval-report.json`
    - incumbent: `output/qwen35-agent-style-eval-wave1-gpu0-report.json`
    - `wave2-gpu1`: `output/qwen35-agent-style-eval-wave2-gpu1-report.json`
    - `wave2-gpu2`: `output/qwen35-agent-style-eval-wave2-gpu2-report.json`
  - `wave2-gpu1` (winner recipe replica, different seed): `33/48 = 0.6875`
  - `wave2-gpu2` (near-neighbor lower-lr recipe): `34/48 = 0.7083`
  - post-wave-2 incumbent remains `wave1-gpu0 = 36/48 = 0.7500`
  - decisive deltas:
    - both valid wave-2 lanes fell back to `plan-repair = 5/12`, so the confirmation batch failed to beat the incumbent on the hardest bucket
    - `verifier_pass_rate` also regressed from `wave1-gpu0 = 0.75` to `wave2-gpu1 = 0.5833` and `wave2-gpu2 = 0.5833`, so `wave-2` should be read as a failed confirmation batch even before considering the crashed `gpu0` lane
- The search-wave launcher semantics are now stricter after the first real run:
  - `ignored_process_names = ["sunshine"]` can ignore desktop-capture noise for exclusivity checks
  - ignored processes still count toward real VRAM pressure
  - `validate-only` now reports validation separately from execution and no longer implies the wave already ran
  - current post-wave-2 validate snapshot: `output/qwen35-workflow-trace-sft-search-wave-1-validate-20260418.json`
  - future waves now fail closed at the whole-wave level instead of partial-launching a subset of lanes
- The current blocker for a clean three-lane rerun is no longer data drift:
  - `GPU0` should be treated as quarantined for training until it is reset or rebooted cleanly after the `Xid 31` incident on the display-attached card
  - if a third confirmation lane is still needed later, rerun it only after `GPU0` has been explicitly recovered
- The first post-wave-2 corrective pass now repairs the dropped `chat_quality` lane instead of pushing another blind exploit:
  - `scripts/qwen35_workflow_trace_sft_train_utils.py` now renders non-reasoning rows with `enable_thinking = false`
  - focused probe: `output/qwen35-workflow-trace-sft-source-recovery-probe-chat-quality.json`
  - repaired `chat-quality-guardrail` result: `6,000 / 6,000` prefix-safe
- The repaired search-ready manifest is now larger and balanced across three groups again:
  - summary: `output/qwen35-workflow-trace-sft-search-ready-manifest-summary.json`
  - total target examples: `23,333`
  - split: `distill = 10,500`, `open_reasoning = 7,000`, `chat_quality = 5,833`
- The repaired search-ready shard has now been rebuilt successfully:
  - summary: `output/qwen35-workflow-trace-sft-search-ready-shard-summary.json`
  - rows written: `23,333`
  - `skipped_prefix_mismatch = 0`
  - `chat-quality-guardrail = 5,833` rows are now present in the final shard
  - follow-up audit: `output/qwen35-workflow-trace-sft-search-ready-prefix-audit.json`
  - audit result: `23,333 / 23,333` rows kept with `0` prefix mismatches
- The healthy-GPU follow-up search is now complete on the repaired shard:
  - config: `configs/qwen35-workflow-trace-sft-search-wave-3-chat-quality-repair.json`
  - lanes: `gpu1-chat-repair-baseline-2048`, `gpu2-chat-repair-mid-lr-2048`
  - train reports:
    - `output/qwen35-workflow-trace-sft-search-wave-3-gpu1-report.json`
    - `output/qwen35-workflow-trace-sft-search-wave-3-gpu2-report.json`
  - wave summary:
    - `output/qwen35-workflow-trace-sft-search-wave-3-chat-quality-repair-summary.json`
  - short-run train snapshot:
    - `gpu1`: `train_loss = 0.8936`
    - `gpu2`: `train_loss = 0.8911`
  - serving note:
    - the adapters are `LoRA rank 64`, so vLLM serving must include `--max-lora-rank 64`
  - eval reports:
    - `output/qwen35-agent-style-eval-wave3-gpu1-report.json`
    - `output/qwen35-agent-style-eval-wave3-gpu2-report.json`
  - eval outcome:
    - baseline pre-wave-1: `35/48 = 0.7292`
    - incumbent `wave1-gpu0`: `36/48 = 0.7500`
    - `wave3-gpu1`: `36/48 = 0.7500`
    - `wave3-gpu2`: `36/48 = 0.7500`
  - the two repaired `wave-3` lanes match the incumbent on the main practical gate:
    - same overall score as `wave1-gpu0`: `36/48 = 0.7500`
    - same failed-case set on the current suite
    - same bucket picture: `coding-fix = 8/8`, `code-understanding = 7/8`, `grounded-docs-qa = 8/8`, `research-synthesis = 6/12`, `plan-repair = 7/12`
    - same `verifier_pass_rate = 0.75`
    - but not full dimension-score parity: `wave1-gpu0` still has slightly higher `correctness`
  - practical decision:
    - keep `wave1-gpu0` as the formal incumbent because `wave-3` did not beat it
    - treat the repaired recipe as a healthy reproduced tie on `GPU1/GPU2`, which makes it a credible base for the next longer exploit run on healthy cards
- The repaired long-run exploit batch is now complete on healthy GPUs:
  - config: `configs/qwen35-workflow-trace-sft-search-wave-4-repaired-exploit.json`
  - lanes:
    - `gpu1-repaired-baseline-2048-epoch15`
    - `gpu2-repaired-midlr-2048-epoch15`
  - train reports:
    - `output/qwen35-workflow-trace-sft-search-wave-4-gpu1-report.json`
    - `output/qwen35-workflow-trace-sft-search-wave-4-gpu2-report.json`
  - wave summary:
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
  - the two longer repaired `wave-4` lanes still match the incumbent on the main practical gate:
    - same overall score as `wave1-gpu0`: `36/48 = 0.7500`
    - same bucket picture: `coding-fix = 8/8`, `code-understanding = 7/8`, `grounded-docs-qa = 8/8`, `research-synthesis = 6/12`, `plan-repair = 7/12`
    - same `verifier_pass_rate = 0.75`
    - still not full dimension-score parity: `wave1-gpu0` keeps slightly higher `correctness`
  - practical decision after `wave-4`:
    - keep `wave1-gpu0` as the formal incumbent because the longer repaired exploit run still does not beat it
    - treat `wave-4` as a stability confirmation that the repaired recipe can hold incumbent-level quality over a long run on healthy GPUs
    - the next Phase 03 decision now has to be a real branch change, not another same-family exploit repeat
- `scripts/run-qwen35-workflow-trace-sft.py` and `scripts/audit-qwen35-workflow-trace-sft-prefix.py` now resolve model/tokenizer/template paths relative to `local-model/`, so they no longer depend on the caller `cwd`.
- The SFT runtime setup now pins the current known-good trainer stack by default instead of floating to latest package releases.
- `scripts/run-qwen35-4b-vllm.sh` now defaults `VLLM_USE_FLASHINFER_SAMPLER=0` for safer cluster restarts after workspace/cache drift.
