# Phase 02: Continual pretraining corpus

## Overview

- Priority: P1
- Status: stopped
- Goal: strengthen the technical prior of `Qwen3.5-4B`.
- Current state: this phase has been stopped and reset by operator on `2026-04-19`; keep it only as a historical record of the attempted CPT lane.

## Key Insights

- This is the strongest "real IQ" lever, but also the slowest one.
- Data quality matters more than sheer volume.
- A manifest with provenance, token budgets, and hygiene rules is the minimum viable artifact before fetching or training anything.
- Public sources still need legal and licensing review before commercial use.
- Under an extreme-optimization goal, this phase is no longer optional scaffolding; it is the most plausible way to buy lasting technical prior beyond pure style imitation.
- This is also the first phase where a real `multi-GPU single-run` setup may be worth the complexity, but only after corpus quality and eval gates are strong enough.

## Requirements

- Clean corpus, deduplicated, with provenance.
- Prioritize code, docs, logs, and troubleshooting material.
- Separate `high-trust core data` from `high-variance candidate data`.
- Support multi-round promotion:
  - round-1 trusted core
  - round-2 promoted troubleshooting/issues
  - round-3 larger high-value external docs/code if licensing and pinning are clean

## Progress Update

1. Added `corpora/qwen35-continual-pretraining-round-1-corpus-manifest.json`.
2. Added `scripts/validate-qwen35-continual-pretraining-corpus.py`.
3. Validated the manifest and wrote `output/qwen35-continual-pretraining-corpus-summary.json`.
4. The current round-1 manifest covers `8` sources and `120M` estimated tokens.
5. The current split is:
   - `4` internal local sources with recorded workspace references
   - `4` public sources marked `review-required` and `pending-pin-before-fetch`
6. Local source validation found:
   - `185` matched product-code files
   - `24` matched local-model runtime/serving files
   - `60` matched docs/plan files
   - `20` matched logs/benchmark files
7. The validator now fails if a local source drops below its required match threshold and reports pending provenance explicitly.
8. Added `scripts/build-qwen35-local-cpt-shard.py`.
9. Built `output/qwen35-local-cpt-shard.jsonl` and `output/qwen35-local-cpt-shard-summary.json`.
10. The current local shard contains:
   - `269` normalized documents
   - `209` code documents
   - `60` docs documents
11. Current shard hygiene stats:
   - `4` secret redactions
   - `1` skipped mutable source
   - `0` collapsed repeated log lines in the default seed build
   - `0` skipped empty files
12. The shard summary now records the live git snapshot used during materialization, and the current run is still `workspace_dirty = true`.
13. Mutable runtime-output sources remain in the full manifest, but the default local seed shard excludes them to reduce drift.
14. Added `scripts/build-qwen35-external-cpt-metadata.py`.
15. Built the external review artifacts:
   - `output/qwen35-external-cpt-pinned-manifest-candidate.json`
   - `output/qwen35-external-cpt-metadata-shard.jsonl`
   - `output/qwen35-external-cpt-metadata-summary.json`
16. The current external metadata lane covers:
   - `4` public sources
   - `14` resolved public references
   - `13` strong references
   - `1` weak reference
   - `2` metadata documents that passed the current quality gate
17. The external metadata lane keeps public sources at `ready = false`; it is meant for review/pinning, not bulk ingestion approval.
18. HTML shell and redirect pages are now skipped instead of being promoted into the metadata shard.
19. Added `scripts/approve-qwen35-external-cpt-sources.py`.
20. Added `scripts/build-qwen35-external-cpt-shard.py`.
21. Added `scripts/build-qwen35-mixed-cpt-shard.py`.
22. Built the approval artifacts:
   - `output/qwen35-external-cpt-approved-manifest.json`
   - `output/qwen35-external-cpt-approval-summary.json`
23. The current approval snapshot moves `2` public sources to `approved-for-fetch` and keeps `2` public sources pending review.
24. Built the fetched external shard:
   - `output/qwen35-external-cpt-shard.jsonl`
   - `output/qwen35-external-cpt-shard-summary.json`
25. The current external shard contains `321` normalized documents:
   - `155` from `qwen-official-repositories-and-docs`
   - `166` from `vllm-and-transformers-runtime-stack`
26. Built the first mixed shard:
   - `output/qwen35-mixed-cpt-shard.jsonl`
   - `output/qwen35-mixed-cpt-shard-summary.json`
27. The current mixed shard contains `590` deduplicated documents merged from `269` local documents and `321` approved external documents.
28. Added `scripts/build-qwen35-issue-cpt-candidate-shard.py`.
29. Added `scripts/qwen35_github_issue_fetcher.py`.
30. Built the issue candidate artifacts:
   - `output/qwen35-issue-cpt-candidate-shard.jsonl`
   - `output/qwen35-issue-cpt-candidate-summary.json`
31. The current issue candidate shard contains `18` normalized documents from closed issue searches:
   - `6` from `vllm-project/vllm`
   - `6` from `huggingface/transformers`
   - `6` from `QwenLM/Qwen3`
32. The issue lane is still a candidate lane; it is intentionally kept separate from the approved external shard until quality review decides whether to promote it.
33. Added `configs/qwen35-continual-pretraining-pilot-round-1.json`.
34. Added `scripts/qwen35_continual_pretraining_train_utils.py`.
35. Added `scripts/run-qwen35-continual-pretraining.py`.
36. The CPT runner now:
   - reads `input_jsonl` from config by default
   - builds packed LM blocks from raw `content`
   - creates a holdout split for in-run eval
   - emits structured failure reports
   - enforces `single visible GPU` launch semantics for this pilot lane
37. Built the pilot dry-run artifacts:
   - `output/qwen35-continual-pretraining-pilot-round-1-dry-run/` (removed during reset on `2026-04-19`)
   - `output/qwen35-continual-pretraining-pilot-round-1-dry-run-report.json`
38. The current dry-run proves the CPT lane can run end-to-end with:
   - `32` sampled documents
   - `57` token blocks at `512` max sequence length
   - `49` train blocks and `8` holdout blocks
   - `train_loss = 0.9873`
   - `eval_loss = 1.0128`
39. Built the first full CPT pilot artifacts:
   - `output/qwen35-continual-pretraining-pilot-round-1-lora/` (removed during reset on `2026-04-19`)
   - `output/qwen35-continual-pretraining-pilot-round-1-report.json`
40. The completed full CPT pilot used the whole mixed shard on `1x RTX 3090` with:
   - `590` documents seen
   - `588` documents kept
   - `429` token blocks at `2048` max sequence length
   - `386` train blocks
   - `43` holdout blocks
   - `3.0` epochs
   - `147` optimizer steps
   - `train_runtime = 5060s`
   - `train_loss = 0.6869`
   - final `eval_loss = 0.7410`
41. The pilot holdout loss improved monotonically across all saved eval checkpoints:
   - `step 25`: `0.8197`
   - `step 50`: `0.7768`
   - `step 75`: `0.7557`
   - `step 100`: `0.7421`
   - `step 125`: `0.7415`
   - `step 147`: `0.7410`
42. The best checkpoint is the final adapter at `checkpoint-147`, so this pilot passed the current `stable end-to-end / non-divergent / usable artifact` gate.
43. Built the first downstream benchmark artifacts for the current CPT pilot adapter:
   - `output/qwen35-agent-style-eval-cpt-pilot-round-1-report.json`
   - `output/qwen35-agent-style-eval-cpt-pilot-round-1-summary.json`
44. The first downstream benchmark is currently a regression against both the pre-CPT baseline and the formal incumbent:
   - pre-CPT baseline: `35/48 = 0.7292`
   - formal incumbent `wave1-gpu0`: `36/48 = 0.7500`
   - CPT pilot adapter: `33/48 = 0.6875`
45. The regression is concentrated in `research-synthesis`, while `plan-repair` stays level with the pre-CPT baseline and still below the formal incumbent:
   - unchanged: `coding-fix = 8/8`, `code-understanding = 7/8`, `grounded-docs-qa = 8/8`
   - reasoning buckets after CPT: `research-synthesis = 4/12`, `plan-repair = 6/12`
46. Case-level delta versus the pre-CPT baseline is currently:
   - improved: `plan-repair-001`
   - regressed: `plan-repair-003`, `research-synthesis-001`, `research-synthesis-007`
47. Current interpretation:
   - the CPT pilot is a `training/runtime` success
   - the current adapter is **not** a `promotion` success on the present downstream eval suite

## Related Code Files

- Modified:
  - `README.md`
  - `plans/260415-1104-qwen35-4b-max-iq-claude-code-runtime/plan.md`
  - `plans/260415-1104-qwen35-4b-max-iq-claude-code-runtime/phase-02-continual-pretraining-corpus.md`
- Created:
  - `corpora/qwen35-continual-pretraining-round-1-corpus-manifest.json`
  - `scripts/approve-qwen35-external-cpt-sources.py`
  - `scripts/build-qwen35-external-cpt-metadata.py`
  - `scripts/build-qwen35-external-cpt-shard.py`
  - `scripts/build-qwen35-issue-cpt-candidate-shard.py`
  - `scripts/build-qwen35-local-cpt-shard.py`
  - `scripts/build-qwen35-mixed-cpt-shard.py`
  - `scripts/qwen35_cpt_fetch_utils.py`
  - `scripts/qwen35_github_issue_fetcher.py`
  - `scripts/validate-qwen35-continual-pretraining-corpus.py`
  - `output/qwen35-continual-pretraining-corpus-summary.json`
  - `output/qwen35-external-cpt-approved-manifest.json`
  - `output/qwen35-external-cpt-approval-summary.json`
  - `output/qwen35-external-cpt-pinned-manifest-candidate.json`
  - `output/qwen35-external-cpt-metadata-shard.jsonl`
  - `output/qwen35-external-cpt-metadata-summary.json`
  - `output/qwen35-external-cpt-shard.jsonl`
  - `output/qwen35-external-cpt-shard-summary.json`
  - `output/qwen35-issue-cpt-candidate-shard.jsonl`
  - `output/qwen35-issue-cpt-candidate-summary.json`
  - `output/qwen35-local-cpt-shard.jsonl`
  - `output/qwen35-local-cpt-shard-summary.json`
  - `output/qwen35-mixed-cpt-shard.jsonl`
  - `output/qwen35-mixed-cpt-shard-summary.json`
  - `configs/qwen35-continual-pretraining-pilot-round-1.json`
  - `scripts/qwen35_continual_pretraining_train_utils.py`
  - `scripts/run-qwen35-continual-pretraining.py`
  - `output/qwen35-continual-pretraining-pilot-round-1-dry-run-report.json`
  - `output/qwen35-continual-pretraining-pilot-round-1-report.json`
  - `output/qwen35-continual-pretraining-pilot-round-1-lora/` (removed during reset on `2026-04-19`)
  - `output/qwen35-agent-style-eval-cpt-pilot-round-1-report.json`
  - `output/qwen35-agent-style-eval-cpt-pilot-round-1-summary.json`

## Implementation Steps

1. Freeze the round-1 manifest and review public-source licensing.
2. Keep the local shard as the trusted seed corpus.
3. Use the external metadata lane to pin and review public-source references.
4. Fetch and normalize the approved public sources into the same document schema.
5. Deduplicate and filter the merged corpus according to the manifest policy.
6. Build the first actual mixed training shard from the validated sources.
7. Split the corpus into:
   - trusted core
   - candidate troubleshooting/issues
   - deferred/weak-reference sources
8. Run at least one small continual pretraining pilot to verify loss behavior and eval direction.
9. Compare the promoted CPT adapter against the current agent-style eval or another strong proxy benchmark before claiming capability gain.
10. If the CPT adapter regresses, treat that as a curriculum/data warning rather than blindly promoting the longer run.
11. Only if the CPT adapter beats the pre-CPT baseline on meaningful probes should this phase move into a longer multi-checkpoint continual pretraining curriculum.
12. Re-evaluate before and after each major checkpoint before moving into large SFT exploit runs.

## Success Criteria

- Clear improvement in code/doc understanding without requiring overly long reasoning traces.
- A provenance-tracked corpus manifest exists with enforced structural checks and explicit pending pins for public sources.
- The token plan stays consistent with the intended round budget.
- Public-source reproducibility gaps are explicit in the summary rather than silently implied.
- A local normalized seed shard can be rebuilt from the current workspace snapshot with the git state recorded explicitly.
- Public sources can be reviewed through a pinned-manifest candidate and a quality-gated metadata shard before bulk fetch.
- Approved public sources can now be fetched and normalized into the same schema as the local shard.
- A first mixed shard can be rebuilt from the current approved external slice and the current local seed shard.
- Issue archives can now be converted into a separate candidate shard using closed GitHub issue searches and extracted issue bodies.
- At least one continual pretraining pilot completes end to end with stable holdout loss and a usable adapter artifact.
- Promotion beyond the first pilot requires downstream gain on the eval suite or on clearly relevant proxy tasks.
- A pilot that regresses on the downstream suite is evidence to hold promotion and branch-change the curriculum.
- Promotion from candidate issue data into the training mix is driven by measured gain, not by intuition.

## Next Steps

- Review the remaining `review-required` public sources before downloading them in bulk.
- Rebuild the local shard from a clean pinned checkout when you want a fully reproducible seed export.
- Decide whether to approve the weak-reference docs source or replace it with a stronger pinned equivalent.
- Review the issue candidate shard and decide whether to promote some or all of it into the training mixture.
- Diagnose why the current CPT adapter regressed on `research-synthesis` and one `plan-repair` case before promoting it.
- Decide whether the next Phase 02 move is:
  - promote some issue-lane data and rerun a shorter CPT compare
  - tighten the trusted-core curriculum or loss recipe
  - benchmark on another proxy suite before discarding the adapter entirely
- Move into workflow-trace SFT only after the corpus and CPT lane are strong enough that the model prior is worth preserving and exploiting.
