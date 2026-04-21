#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from collections import Counter
from pathlib import Path
from typing import Any


ENV_ASSIGNMENT_PATTERN = re.compile(r"^\s*(?:export\s+)?([A-Z0-9_]+)\s*=\s*(.+?)\s*$")
JSON_SECRET_PATTERN = re.compile(r'^(?P<prefix>\s*"(?P<key>[^"]+)"\s*:\s*")(?P<value>[^"\n]+)(?P<suffix>".*)$')
SECRET_NAME_SEGMENTS = {
    "API", "KEY", "TOKEN", "SECRET", "PASSWORD", "ACCESS", "REFRESH", "CLIENT",
}
EXACT_SECRET_KEYS = {
    "api_key", "token", "access_token", "refresh_token", "secret",
    "secret_key", "client_secret", "password",
}


def ensure(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text())
    except FileNotFoundError as exc:
        raise SystemExit(f"Manifest not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Manifest contains invalid JSON: {exc}") from exc
    ensure(isinstance(payload, dict), "Manifest must be a JSON object.")
    return payload


def resolve_patterns(base_dir: Path, patterns: list[str]) -> set[Path]:
    matched: set[Path] = set()
    for pattern in patterns:
        matched.update(path.resolve() for path in base_dir.glob(pattern))
    return matched


def is_binary_blob(raw: bytes) -> bool:
    return b"\x00" in raw


def redact_secrets(text: str) -> tuple[str, int]:
    replacements = 0
    redacted_lines: list[str] = []
    for line in text.splitlines():
        env_match = ENV_ASSIGNMENT_PATTERN.match(line)
        if env_match:
            name, value = env_match.groups()
            segments = set(name.split("_"))
            if segments & SECRET_NAME_SEGMENTS and len(value.strip("\"'")) >= 8 and not value.strip("\"'").isdigit():
                redacted_lines.append(line.replace(value, "[REDACTED]", 1))
                replacements += 1
                continue
        json_match = JSON_SECRET_PATTERN.match(line)
        if json_match and json_match.group("key").lower() in EXACT_SECRET_KEYS:
            value = json_match.group("value")
            if len(value) >= 8 and not value.isdigit():
                redacted_lines.append(f"{json_match.group('prefix')}[REDACTED]{json_match.group('suffix')}")
                replacements += 1
                continue
        redacted_lines.append(line)
    return "\n".join(redacted_lines) + ("\n" if text.endswith("\n") else ""), replacements


def collapse_repeated_lines(text: str, keep: int = 3) -> tuple[str, int]:
    collapsed: list[str] = []
    removed = 0
    previous = None
    streak = 0
    for line in text.splitlines():
        if line == previous:
            streak += 1
        else:
            previous = line
            streak = 1
        if streak <= keep:
            collapsed.append(line)
        else:
            removed += 1
    return "\n".join(collapsed) + ("\n" if text.endswith("\n") and collapsed else ""), removed


def normalize_text(path: Path, raw: bytes, source: dict[str, Any], root: Path) -> tuple[dict[str, Any] | None, dict[str, int]]:
    stats = {"skipped_binary": 0, "skipped_empty": 0, "redactions": 0, "collapsed_log_lines": 0}
    if is_binary_blob(raw):
        stats["skipped_binary"] = 1
        return None, stats
    text = raw.decode("utf-8", errors="replace").replace("\r\n", "\n").replace("\r", "\n")
    if source["domain"] == "troubleshooting":
        text, removed = collapse_repeated_lines(text)
        stats["collapsed_log_lines"] = removed
    text, redactions = redact_secrets(text)
    stats["redactions"] = redactions
    if not text.strip():
        stats["skipped_empty"] = 1
        return None, stats
    digest = hashlib.sha256(text.encode("utf-8")).hexdigest()
    document = {
        "schema": "qwen35-cpt-document-v1",
        "round_id": source["round_id"],
        "source_id": source["id"],
        "domain": source["domain"],
        "path": str(path),
        "relative_path": str(path.relative_to(root)),
        "content_sha256": digest,
        "char_count": len(text),
        "line_count": text.count("\n") + (0 if not text else 1),
        "content": text,
    }
    return document, stats


def get_git_snapshot(repo_root: Path) -> dict[str, Any]:
    head = subprocess.run(
        ["git", "-C", str(repo_root), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    status = subprocess.run(
        ["git", "-C", str(repo_root), "status", "--porcelain"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.splitlines()
    return {
        "head_commit": head,
        "workspace_dirty": bool(status),
        "dirty_paths": [line[3:] for line in status[:20]],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", default="corpora/qwen35-continual-pretraining-round-1-corpus-manifest.json")
    parser.add_argument("--output-jsonl", default="output/qwen35-local-cpt-shard.jsonl")
    parser.add_argument("--output-summary", default="output/qwen35-local-cpt-shard-summary.json")
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    manifest = load_manifest(manifest_path)
    ensure(manifest.get("format") == "qwen35-continual-pretraining-corpus-v1", "Unsupported manifest format.")
    base_dir = manifest_path.parent
    repo_root = manifest_path.parents[2]
    git_snapshot = get_git_snapshot(repo_root)
    output_jsonl = Path(args.output_jsonl).resolve()
    output_summary = Path(args.output_summary).resolve()
    output_jsonl.parent.mkdir(parents=True, exist_ok=True)
    output_summary.parent.mkdir(parents=True, exist_ok=True)

    documents: list[dict[str, Any]] = []
    seen_hashes: set[str] = set()
    source_counts: Counter[str] = Counter()
    domain_counts: Counter[str] = Counter()
    totals = Counter()

    for source in manifest.get("sources", []):
        if source.get("kind") != "local-glob":
            continue
        if not source.get("provenance", {}).get("ready"):
            continue
        if source.get("include_in_local_seed_shard") is False:
            totals["skipped_mutable_sources"] += 1
            continue
        include_globs = source.get("include_globs", [])
        exclude_globs = source.get("exclude_globs", [])
        domain = source.get("domain")
        default_max_file_bytes = 400000 if domain in {"code", "docs"} else 1000000
        max_file_bytes = source.get("filters", {}).get("max_file_bytes", default_max_file_bytes)
        ensure(isinstance(max_file_bytes, int) and max_file_bytes > 0, f"{source.get('id')}: invalid max_file_bytes")
        included = resolve_patterns(base_dir, include_globs)
        excluded = resolve_patterns(base_dir, exclude_globs)
        for path in sorted(candidate for candidate in included if candidate not in excluded and candidate.is_file()):
            raw = path.read_bytes()
            if len(raw) > max_file_bytes:
                totals["skipped_large_files"] += 1
                continue
            source_with_round = dict(source)
            source_with_round["round_id"] = manifest["round_id"]
            document, stats = normalize_text(path, raw, source_with_round, repo_root)
            totals.update(stats)
            if document is None:
                continue
            if document["content_sha256"] in seen_hashes:
                totals["deduped_documents"] += 1
                continue
            seen_hashes.add(document["content_sha256"])
            documents.append(document)
            source_counts[source["id"]] += 1
            domain_counts[source["domain"]] += 1

    with output_jsonl.open("w", encoding="utf-8") as handle:
        for document in documents:
            handle.write(json.dumps(document, ensure_ascii=False) + "\n")

    summary = {
        "ok": True,
        "manifest": str(manifest_path),
        "output_jsonl": str(output_jsonl),
        "round_id": manifest["round_id"],
        "git_snapshot": git_snapshot,
        "document_count": len(documents),
        "sources_included": dict(sorted(source_counts.items())),
        "domains_included": dict(sorted(domain_counts.items())),
        "stats": {
            "skipped_mutable_sources": totals["skipped_mutable_sources"],
            "deduped_documents": totals["deduped_documents"],
            "skipped_large_files": totals["skipped_large_files"],
            "skipped_binary": totals["skipped_binary"],
            "skipped_empty": totals["skipped_empty"],
            "secret_redactions": totals["redactions"],
            "collapsed_log_lines": totals["collapsed_log_lines"],
        },
    }
    output_summary.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
