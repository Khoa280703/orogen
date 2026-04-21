---
title: "Qwen3.5-4B Phase 02 CPT pilot slice"
description: "Small bounded plan for the first real continual-pretraining pilot on the existing mixed CPT shard."
status: pending
priority: P1
effort: 3h
branch: main
tags: [qwen, qwen3.5-4b, continual-pretraining, pilot, phase-02]
created: 2026-04-19
---

# Goal
Prove the current approved mixed CPT shard can support a cheap, real `QLoRA causal-LM` pilot before any promoted curriculum change or multi-GPU CPT run.

## Exact files
- Add: `configs/qwen35-continual-pretraining-pilot-round-1.json`
- Add: `scripts/qwen35_cpt_train_utils.py`
- Add: `scripts/run-qwen35-continual-pretraining.py`
- Modify after run: `README.md`
- Modify after run: `plans/260415-1104-qwen35-4b-max-iq-claude-code-runtime/phase-02-continual-pretraining-corpus.md`

## Key design choices
- Reuse `Phase 03` runtime + script shape, not a new framework:
  - same `./.venv-qwen35-sft`
  - same JSON config + optional override pattern
  - same relative-path resolution and JSON report artifact
- Train on existing `output/qwen35-mixed-cpt-shard.jsonl`; do not create another corpus lane first.
- First pilot is `single-GPU adapter CPT`, not distributed/full-parameter CPT.
- Keep data bounded to the approved mixed shard only; do not promote `output/qwen35-issue-cpt-candidate-shard.jsonl` yet.
- Prefer `GPU1` or `GPU2`; avoid making `GPU0` the default pilot target after the Phase 03 infra incident.

## Implementation slice
1. Add a CPT config mirroring `configs/qwen35-workflow-trace-sft-round-1.json`, but for plain-text LM:
   - base model/tokenizer = `./models/qwen3.5-4b`
   - input shard = `./output/qwen35-mixed-cpt-shard.jsonl`
   - output dir = `./output/qwen35-cpt-pilot-round-1-adapter`
   - `max_sequence_length = 2048`
   - `learning_rate = 5e-5`
   - `per_device_train_batch_size = 1`
   - `gradient_accumulation_steps = 16`
   - `save_steps = 32`, `eval_steps = 16`
- 2. Add small shared utils to:
  - load CPT docs from `qwen35-cpt-document-v1`
  - stable-sample documents by seed
  - split small eval holdout by stable hash
  - chunk `content` into token blocks for causal-LM
  - summarize source/domain/token coverage into the final report
3. Add a runner shaped like `scripts/run-qwen35-workflow-trace-sft.py`, but using `transformers.Trainer` + causal-LM collator instead of prompt/completion SFT.
4. Support `--validate-only`, `--dry-run`, `--sample-limit`, `--eval-sample-limit`, `--max-steps`, `--max-sequence-length`, `--gpu-id`, `--seed`.

## Validation
- `output/qwen35-mixed-cpt-shard-summary.json` still shows `590` deduped docs before training.
- `--validate-only` passes config/model/tokenizer/dataset checks and writes a JSON report.
- `--dry-run` completes `1` step on a small sample and writes a real adapter dir.
- Real pilot completes `64` steps on `GPU1` or `GPU2` with no OOM or illegal-memory failure.
- Report shows finite `train_loss` and finite `eval_loss`; no divergence by the last logged step.
- Report includes dataset mix actually seen, so later promotion decisions are grounded in the pilot input, not just the corpus plan.

## Smallest real pilot worth running now
```bash
CUDA_VISIBLE_DEVICES=1 ./.venv-qwen35-sft/bin/python ./scripts/run-qwen35-continual-pretraining.py \
  --config ./configs/qwen35-continual-pretraining-pilot-round-1.json \
  --train-jsonl ./output/qwen35-mixed-cpt-shard.jsonl \
  --output-dir ./output/qwen35-cpt-pilot-round-1-gpu1-adapter \
  --report-output ./output/qwen35-cpt-pilot-round-1-gpu1-report.json \
  --sample-limit 512 \
  --eval-sample-limit 64 \
  --max-steps 64 \
  --max-sequence-length 2048 \
  --gpu-id 0
```

## Done when
- Pilot artifacts exist: adapter dir, JSON report, log.
- Loss curve is sane enough to justify a second slice: either larger token budget on the same approved shard, or a controlled promotion test for the issue lane.

## Unresolved questions
- None for this slice.
