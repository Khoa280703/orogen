# Qwen3.5-4B Workspace

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

Current final Phase 01 snapshot:

- Latest checked report: `output/qwen35-agent-style-eval-report.json`
- `generated_at`: `2026-04-15T07:33:06.280203+00:00`
- Overall: `35/40 = 0.875`
- Strong buckets: `coding-fix = 8/8`, `grounded-docs-qa = 8/8`, `code-understanding = 7/8`
- Weak buckets: `research-synthesis = 6/8`, `plan-repair = 6/8`

Notes:

- Run from the `local-model/` directory if you want to use default paths.
- The default report path is `output/qwen35-agent-style-eval-report.json`.
- The report always includes `generated_at`; if you want a separate snapshot per run, pass `--output`.
- The runner is resilient per case: if one request fails, the batch still continues and the report is still written.
- `evals/qwen35-agent-style-eval-cases.json` is now the manifest; bucket files live in `evals/agent-style-eval/`.
- The Phase 01 eval suite is now English-first.
- `scorecard-schema.json` is loaded, validated, and included in the report.
- `summary.dimension_scores` is driven by case-level `dimension_checks`; the validator now requires every mapped dimension to be annotated for every case in this suite.
- In the latest final Phase 01 snapshot, `summary.dimension_scores` has full coverage (`coverage = 1.0`) with `proxy_cases = 0` for every reported dimension in this suite.
- `summary.dimension_proxies` is kept only for backward comparison with older snapshots using bucket/case pass-fail; do not use it as the primary read for this suite.
- `summary.overall.pass_rate` is the case-level pass rate; it is not the same thing as a specific dimension such as `correctness` or `concision`.
- `dimension_scores` in the latest final snapshot are: `correctness = 0.9`, `groundedness = 1.0`, `concision = 0.9583`, `verifier_pass_rate = 0.875`, `citation_faithfulness = 1.0`.
- `verifier_pass_rate` is still a text-level heuristic in this suite; it should not be read as a true verifier-backed execution metric.
- After the last `plan-repair-001` verifier-phrasing refinement, the final Phase 01 snapshot intentionally stays at `35/40` to reduce false-green risk instead of preserving a superficially higher pass rate or a cleaner-looking heuristic verifier score.
- `plan-repair` is now stricter on verifier-style remediation semantics; read it as a tighter plan-quality screen, not as direct evidence of model regression by itself.
