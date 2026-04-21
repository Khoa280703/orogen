# Phase 03 wave-2 gpu0 failure

Date: 2026-04-17 09:51 ICT
Work context: `/home/khoa2807/working-sources/duanai/local-model`

## Executive summary

GPU0 lane failed at the first backward pass with `torch.AcceleratorError: CUDA error: an illegal memory access was encountered`. This happened after model load and full dataset tokenization completed, and before step `1/1641` finished.

The same wave recipe is currently progressing on GPU1 and GPU2, so this is not strong evidence of a dataset-wide or config-wide deterministic failure.

## Evidence

- Failure point in `output/qwen35-workflow-trace-sft-search-wave-2-gpu0-report.log`
  - training starts at `0/1641`
  - traceback ends in `trainer.train()` -> `accelerator.backward(loss)` -> `loss.backward()`
  - final error: `cudaErrorIllegalAddress`
- Empty adapter output:
  - `output/qwen35-workflow-trace-sft-search-wave-2-gpu0-adapter/` exists but contains no checkpoint files
- Runner path:
  - model loaded with 4-bit quantization and `device_map={"": args.gpu_id}` in `scripts/run-qwen35-workflow-trace-sft.py`
  - crash occurs inside `train_result = trainer.train()` at [run-qwen35-workflow-trace-sft.py](/home/khoa2807/working-sources/duanai/local-model/scripts/run-qwen35-workflow-trace-sft.py#L175)
- Wave launcher behavior:
  - each lane masks the physical GPU with `CUDA_VISIBLE_DEVICES=<lane gpu>` and then passes `--gpu-id 0`, so gpu1/gpu2 showing `--gpu-id 0` is expected masked indexing, not a routing bug
  - see [run-qwen35-workflow-trace-sft-search-wave.py](/home/khoa2807/working-sources/duanai/local-model/scripts/run-qwen35-workflow-trace-sft-search-wave.py#L50)
- Live state at inspection:
  - GPU0: display-attached, `Disp.A On`
  - GPU0 has graphics stack on it: `Xorg`, `gnome-shell`, `nautilus`, portals, plus `sunshine`
  - `sunshine` uses ~264 MiB and is explicitly ignored by wave preflight
  - manual gpu0 retry PID `400551` is running now; it is CPU-heavy and has only ~256 MiB on GPU0 at this instant, consistent with tokenization / pre-step prep
  - GPU1 and GPU2 training processes are still progressing past step 1

## Likely cause categories

1. GPU0-local CUDA/runtime instability during backward
   - strongest category
   - evidence: illegal memory access appears only on GPU0, at first backward, while sibling lanes with same stack continue

2. Interaction with display-attached GPU0
   - plausible contributing factor, not proven root cause
   - evidence: GPU0 is the only display GPU and has persistent graphics/streaming processes absent on GPU1/GPU2

3. General code/config/dataset bug
   - lower confidence
   - evidence against: same training script, same shard, same hyperparameter family active on GPU1/GPU2 without immediate crash

4. Sunshine specifically
   - weak as primary cause
   - evidence: sunshine is present on GPU0 and ignored by scheduler, but the broader issue is more likely "GPU0 is the desktop/display GPU with active graphics stack" than sunshine alone

## Retry guidance

Safest low-disruption retry while GPU1/GPU2 continue:

- Keep GPU1/GPU2 untouched
- Use the free GPU0 slot only after the prior failed GPU0 process is gone
- Launch GPU0 alone, not via the wave scheduler
- Prefer a clean detached run with its own log file
- If GPU0 fails again, rerun once with `CUDA_LAUNCH_BLOCKING=1` for a better failure site
- If repeated illegal-address failures remain GPU0-only, treat GPU0 as unsuitable for this training recipe while it is the active display GPU

Unresolved questions:

- No kernel `Xid` evidence collected; `dmesg` access was not permitted in this session
- Manual retry log was not redirected to a file, so only live process state was available
