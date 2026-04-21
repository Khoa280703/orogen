# Phase 01: Build agent-style eval

## Overview

- Priority: P1
- Status: in_progress
- Goal: measure the capabilities that actually matter for the model running under Claude Code.
- Current snapshot: the vertical slice is done, the expanded suite now has manifest + bucket files + `48` English-first cases + `case-level dimension_checks`, and the report includes both `dimension_scores` and `dimension_proxies`; after the weakest-bucket expansion the full phase is still not complete.

## Key Insights

- A plain chat benchmark does not reflect real runtime competence.
- The suite must cover planning, evidence reading, repair, and grounded synthesis.
- Under an extreme month-scale optimization plan, a weak eval is worse than no eval because it can bless expensive but fake gains.
- The first vertical slice proved the runner works against the real gateway and writes the expected report.
- The expanded suite now proves manifest-based workflow, bucket-level organization, `case-level dimension_checks`, manifest/scorecard validation, and reporting with both `dimension_scores` and `dimension_proxies`.
- The old proxy-only limitation is mostly addressed: `dimension_scores` is now the primary scoring path when a case is annotated, and `dimension_proxies` is kept only for backward comparison with older snapshots.
- The validator now enforces full mapped-dimension coverage per case for this suite, so current `dimension_scores` are backed by explicit case annotations rather than accidental proxy fallback.
- Earlier false-red cleanup was kept in part, especially around loosened lexical coverage and concision budgets for clearly valid short answers.
- The latest final `plan-repair-001` verifier-phrasing refinement kept that cleanup selectively, but pushed verifier-step semantics harder with `unordered_line_regexes` and related step constraints so `plan-repair` is less vulnerable to shallow remediation stubs.
- That final refinement intentionally preferred stricter verifier semantics over an inflated pass rate or cleaner-looking heuristic verifier score, even though it kept the same `35/40` overall result before the next bucket expansion.
- The later weakest-bucket expansion kept the absolute pass count at `35` while lowering the rate to `35/48`, which is a useful sign that the suite is exposing broader reasoning and repair weaknesses rather than simply preserving the easier earlier slice.
- The suite is stronger after that tradeoff, but the oracle is still not strong enough to be treated as the final quality gate because `research-synthesis` still has lexical coverage gaps, some `plan-repair` cases remain heuristic, and `verifier_pass_rate` remains text-level rather than execution-backed.

## Requirements

- Have an internal benchmark based on real tasks.
- Compare `base`, `tuned`, `tuned+RAG`, and `tuned+RAG+reflection`.
- Cover the original targets from plan `1057`: `reasoning`, `coding`, `grounded QA`, and `instruction following`.

## Progress Update

- Done in the vertical slice:
  1. Created harness `scripts/run-qwen35-agent-style-eval.py`
  2. Created seed cases `evals/qwen35-agent-style-eval-cases.json`
  3. Updated `README.md` with eval usage
  4. Ran `--validate-only`: pass
  5. Ran baseline live eval through `http://127.0.0.1:8004`: `8/8` pass
- Done additionally in the expanded suite:
  1. Converted the suite to manifest mode in `evals/qwen35-agent-style-eval-cases.json`
  2. Split the suite into `5` bucket files under `evals/agent-style-eval/`
  3. Expanded the suite to `40` cases
  4. Added `scorecard-schema.json`, validated it, and reported both `summary.dimension_scores` and `summary.dimension_proxies`
  5. Added `case-level dimension_checks` for true case-level dimension scoring
  6. Added validation that blocks mismatches between `manifest` and `scorecard.bucket_dimensions`
  7. Added `generated_at` to the report
  8. Added validation that enforces full mapped-dimension coverage per case
  9. Kept the expanded suite running end-to-end through the real gateway
  10. Tightened the oracle with `max_words`, stronger `unordered_line_regexes`, and stricter verifier-step semantics in the latest review rounds
  11. Applied a final `plan-repair-001` verifier-phrasing refinement that kept the same suite score while improving the meaning of the verifier step
  12. Expanded `research-synthesis` from `8` to `12` cases
  13. Expanded `plan-repair` from `8` to `12` cases
- Current baseline snapshot:
  1. `--validate-only`: pass
  2. live eval: `35/48 = 0.7292`
  3. report timestamp: `2026-04-15T08:14:08.739454+00:00`
  4. buckets:
     - `coding-fix`: `8/8`
     - `code-understanding`: `7/8`
     - `grounded-docs-qa`: `8/8`
     - `research-synthesis`: `6/12`
     - `plan-repair`: `6/12`
  5. `dimension_scores`:
     - `correctness`: `0.75`
     - `groundedness`: `1.0`
     - `concision`: `0.875`
     - `verifier_pass_rate`: `0.6667`
     - `citation_faithfulness`: `1.0`
  6. this baseline reflects the expanded English-first suite after the newest oracle pass and should not be compared directly against the older Vietnamese-oriented snapshot
  7. the absolute pass count stayed at `35`, but the new bucket expansion exposed more explicit misses around trade-off articulation and post-remediation verification
  8. earlier false-red cleanup was only partially retained; the final `plan-repair-001` refinement kept stricter verifier semantics and explicitly preferred less false-green over preserving a prettier pass rate
  9. the newest snapshot is stricter about counting verifier-style post-remediation steps, so shallow redeploy-only or pre-remediation verification phrasing is less likely to over-score
  10. the suite is materially stronger as a screening baseline now, but some residual false-green and text-only heuristic scoring still remain
- Not finished for the full phase:
  1. Expand each bucket further to `40-100` cases
  2. Tighten the remaining heuristic oracle logic, especially lexical coverage in `research-synthesis` and remaining verifier phrasing gaps in `plan-repair`
  3. Keep validator and schema guardrails strong enough to prevent future proxy-fallback regressions
  4. Harden malformed-input and validator/reporter regression coverage
  5. Lock a stable baseline that can serve as a quality gate for later training and tuning rounds

## Implementation Steps

1. Create 5 buckets:
   - coding fix
   - code understanding
   - grounded docs QA
   - research synthesis
   - multi-step plan/repair
2. Complete the vertical-slice seed set across the 5 buckets.
3. Move the suite to manifest + bucket files.
4. Expand each bucket to at least `40-100` examples.
5. Build the scorecard:
   - correctness
   - groundedness
   - concision
   - verifier pass rate
   - citation faithfulness
6. Annotate `dimension_checks` at case level and keep proxy summaries in parallel for back-comparison.
7. Map back to the older baseline:
   - `reasoning` -> `research synthesis` + `plan/repair`
   - `coding` -> `coding fix` + `code understanding`
   - `grounded QA` -> `grounded docs QA`
   - `instruction following` -> measured through `correctness + concision`

## Success Criteria

- Vertical slice complete:
  - harness runs end-to-end
  - seed cases run against the real gateway
  - baseline report is generated reliably
- Expanded slice complete:
  - suite uses manifest + bucket files
  - `48` cases run end-to-end
  - scorecard schema is loaded and validated
  - report includes `dimension_scores` and `dimension_proxies`
  - validation blocks manifest/scorecard mismatches
  - report includes `generated_at`
- Full phase complete:
  - each bucket has `40-100` cases
  - validator/oracle is tight enough to reduce false green and false red in the quality-sensitive buckets
  - mapped dimensions remain fully annotated per case so that `dimension_scores` does not silently fall back to proxy scoring
  - the baseline is stable enough to compare repeated train/tune rounds
  - the eval suite is strong enough to drive go/no-go decisions for future training work

## Next Steps

- Expand the case set first in the weakest buckets, while tightening the remaining heuristic oracle logic.
- Keep `dimension_proxies` for back-comparison, but use `dimension_scores` as the primary reading path for new snapshots.
- Prioritize broader semantic oracle cleanup in `research-synthesis` and cleaner verifier phrasing coverage in `plan-repair` before treating this as a hard gate.
- Do not bless any month-scale exploit run until the eval suite is strong enough to measure real differences.
