# Qwen35 Agent-Style Eval

This directory contains the bucket files for `Phase 01`.

The current suite is English-first.

- `coding-fix.jsonl`
- `code-understanding.jsonl`
- `grounded-docs-qa.jsonl`
- `research-synthesis.jsonl`
- `plan-repair.jsonl`

The primary manifest remains:

- `../qwen35-agent-style-eval-cases.json`

Current final Phase 01 snapshot:

- Latest checked report: `../../output/qwen35-agent-style-eval-report.json`
- `generated_at`: `2026-04-15T08:14:08.739454+00:00`
- Overall: `35/48 = 0.7292`
- Buckets: `coding-fix = 8/8`, `code-understanding = 7/8`, `grounded-docs-qa = 8/8`, `research-synthesis = 6/12`, `plan-repair = 6/12`
- `dimension_scores`: `correctness = 0.75`, `groundedness = 1.0`, `concision = 0.875`, `verifier_pass_rate = 0.6667`, `citation_faithfulness = 1.0`

Notes:

- Bucket files must declare full `dimension_checks` coverage for all mapped dimensions in this suite.
- The current suite size is `48` cases after expanding the two weakest buckets from `8` to `12` cases each.
- Read `summary.dimension_scores` first for this suite; the latest final Phase 01 snapshot has full direct scoring coverage and `proxy_cases = 0` for all reported dimensions.
- The report also exports `summary.dimension_proxies`, but that view exists only for backward compatibility with older snapshots.
- `verifier_pass_rate` remains a text-level heuristic dimension, not a true execution-backed verifier metric.
- The latest `48`-case snapshot keeps the same absolute pass count as the earlier `35/40` snapshot, but the pass rate drops because the suite now probes broader reasoning and repair behavior.
- The latest oracle tightening makes `plan-repair` a stricter bucket than older snapshots, especially around concise multi-step remediation plans and verifier-style step semantics.

Run:

```bash
python3 ./scripts/run-qwen35-agent-style-eval.py --validate-only
python3 ./scripts/run-qwen35-agent-style-eval.py --base-url http://127.0.0.1:8004 --model qwen3.5-4b
```
