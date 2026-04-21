#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.parse import urlparse

from qwen35_cpt_fetch_utils import (
    build_document,
    ensure,
    external_relative_path,
    fetch_url,
    is_binary_blob,
    is_meaningful_page,
    load_manifest,
    normalize_url,
    redact_secrets,
)


TEXT_EXTENSIONS = {".md", ".rst", ".txt", ".py", ".rs", ".toml", ".yaml", ".yml", ".json", ".js", ".ts", ".tsx", ".sh"}
SKIP_DIR_MARKERS = {".git", ".github", "__pycache__", "node_modules", "dist", "build", "target", "vendor", "third_party", "fixtures"}
SKIP_SUFFIXES = {".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".pdf", ".parquet", ".safetensors", ".bin", ".onnx", ".pt"}


def repo_commit_for_uri(source: dict[str, Any], uri: str) -> str | None:
    prefix = f"{uri} -> git:"
    for ref in source.get("provenance", {}).get("resolved_references", []):
        if ref.startswith(prefix):
            return ref.split("git:", 1)[1]
    return None


def repo_path_score(path: Path) -> tuple[int, int, str]:
    rel = str(path).lower()
    score = 0
    for token, weight in {
        "docs/": 6, "doc/": 6, "examples/": 5, "example/": 5, "tutorial": 5,
        "guide": 4, "readme": 4, "server": 4, "serve": 4, "inference": 4,
        "engine": 3, "model": 2, "embedding": 3, "config": 2, "api": 2,
        "test": -4, "benchmark": -3,
    }.items():
        score += weight if token in rel else 0
    return (-score, path.stat().st_size, rel)


def should_skip_repo_path(path: Path) -> bool:
    if any(part in SKIP_DIR_MARKERS for part in path.parts):
        return True
    suffix = path.suffix.lower()
    if suffix in SKIP_SUFFIXES:
        return True
    if suffix and suffix not in TEXT_EXTENSIONS:
        return True
    return path.name.endswith(".min.js")


def materialize_repo_documents(source: dict[str, Any], uri: str, commit: str, temp_root: Path, max_files: int) -> tuple[list[dict[str, Any]], dict[str, int]]:
    repo_name = uri.rstrip("/").split("/")[-1]
    repo_dir = temp_root / repo_name
    subprocess.run(["git", "clone", "--depth", "1", "--filter=blob:none", uri, str(repo_dir)], check=True, capture_output=True, text=True)
    head = subprocess.run(["git", "-C", str(repo_dir), "rev-parse", "HEAD"], check=True, capture_output=True, text=True).stdout.strip()
    if head != commit:
        subprocess.run(["git", "-C", str(repo_dir), "fetch", "--depth", "1", "origin", commit], check=True, capture_output=True, text=True)
        subprocess.run(["git", "-C", str(repo_dir), "checkout", "--detach", "FETCH_HEAD"], check=True, capture_output=True, text=True)
    candidates = [path for path in repo_dir.rglob("*") if path.is_file() and not should_skip_repo_path(path)]
    selected = sorted(candidates, key=repo_path_score)[:max_files]
    documents: list[dict[str, Any]] = []
    stats = {"repo_candidates": len(candidates), "repo_selected": len(selected), "repo_binary": 0, "repo_empty": 0, "repo_redactions": 0}
    for path in selected:
        raw = path.read_bytes()
        if is_binary_blob(raw):
            stats["repo_binary"] += 1
            continue
        text = raw.decode("utf-8", errors="replace").replace("\r\n", "\n").replace("\r", "\n")
        text, redactions = redact_secrets(text)
        stats["repo_redactions"] += redactions
        if not text.strip():
            stats["repo_empty"] += 1
            continue
        repo_rel = path.relative_to(repo_dir)
        documents.append(build_document(
            round_id=source["round_id"],
            source_id=source["id"],
            domain=source["domain"],
            path=f"{uri}/blob/{commit}/{repo_rel.as_posix()}",
            relative_path=f"external/{source['id']}/{repo_name}/{repo_rel.as_posix()}",
            text=text,
            metadata_lines=[f"Repository: {uri}", f"Commit: {commit}", f"Path: {repo_rel.as_posix()}"],
        ))
    return documents, stats


def crawl_doc_pages(source: dict[str, Any], seed_url: str, max_pages: int) -> tuple[list[dict[str, Any]], dict[str, int]]:
    seed_page = fetch_url(seed_url)
    parsed_seed = urlparse(seed_page["final_url"])
    prefix = parsed_seed.path if parsed_seed.path.endswith("/") else parsed_seed.path.rsplit("/", 1)[0] + "/"
    queue = [seed_page["final_url"]]
    visited: set[str] = set()
    documents: list[dict[str, Any]] = []
    stats = {"pages_visited": 0, "pages_kept": 0, "pages_skipped": 0}
    slot = 1
    while queue and len(visited) < max_pages:
        current = queue.pop(0)
        if current in visited:
            continue
        visited.add(current)
        page = fetch_url(current)
        stats["pages_visited"] += 1
        parsed = urlparse(page["final_url"])
        if parsed.netloc != parsed_seed.netloc or not parsed.path.startswith(prefix):
            continue
        if is_meaningful_page(source["kind"], page["title"], page["text"]):
            metadata = [f"Seed URL: {seed_url}", f"Final URL: {page['final_url']}", f"Title: {page['title']}"]
            if page["etag"]:
                metadata.append(f"ETag: {page['etag']}")
            if page["last_modified"]:
                metadata.append(f"Last-Modified: {page['last_modified']}")
            documents.append(build_document(
                round_id=source["round_id"],
                source_id=source["id"],
                domain=source["domain"],
                path=page["final_url"],
                relative_path=external_relative_path(source["id"], page["final_url"], slot),
                text=page["text"],
                metadata_lines=metadata,
            ))
            slot += 1
            stats["pages_kept"] += 1
        else:
            stats["pages_skipped"] += 1
        for link in page["links"]:
            normalized = normalize_url(link)
            link_parts = urlparse(normalized)
            if link_parts.netloc == parsed_seed.netloc and link_parts.path.startswith(prefix) and normalized not in visited:
                queue.append(normalized)
    return documents, stats


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", default="output/qwen35-external-cpt-approved-manifest.json")
    parser.add_argument("--output-jsonl", default="output/qwen35-external-cpt-shard.jsonl")
    parser.add_argument("--output-summary", default="output/qwen35-external-cpt-shard-summary.json")
    parser.add_argument("--max-doc-pages-per-uri", type=int, default=24)
    parser.add_argument("--max-repo-files-per-uri", type=int, default=160)
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    output_jsonl = Path(args.output_jsonl).resolve()
    output_summary = Path(args.output_summary).resolve()
    output_jsonl.parent.mkdir(parents=True, exist_ok=True)
    output_summary.parent.mkdir(parents=True, exist_ok=True)

    manifest = load_manifest(manifest_path)
    ensure(manifest.get("format") == "qwen35-continual-pretraining-corpus-v1", "Unsupported manifest format.")
    sources = []
    for source in manifest.get("sources", []):
        if source.get("provenance", {}).get("type") != "public":
            continue
        if not source.get("provenance", {}).get("ready"):
            continue
        source_with_round = dict(source)
        source_with_round["round_id"] = manifest["round_id"]
        sources.append(source_with_round)
    ensure(sources, "No approved public sources found in manifest.")

    documents: list[dict[str, Any]] = []
    seen_hashes: set[str] = set()
    warnings: list[str] = []
    summary_sources: list[dict[str, Any]] = []
    totals = Counter()

    with tempfile.TemporaryDirectory(prefix="qwen35-external-cpt-") as temp_dir_raw:
        temp_dir = Path(temp_dir_raw)
        for source in sources:
            source_stats = Counter()
            for uri in source.get("uris", []):
                try:
                    commit = repo_commit_for_uri(source, uri)
                    if commit:
                        fetched_docs, stats = materialize_repo_documents(source, uri, commit, temp_dir, args.max_repo_files_per_uri)
                    else:
                        fetched_docs, stats = crawl_doc_pages(source, uri, args.max_doc_pages_per_uri)
                except (subprocess.CalledProcessError, HTTPError, URLError, TimeoutError, OSError) as exc:
                    warnings.append(f"{source['id']}: failed to fetch {uri}: {type(exc).__name__}")
                    source_stats["failed_uris"] += 1
                    continue
                source_stats.update(stats)
                for document in fetched_docs:
                    if document["content_sha256"] in seen_hashes:
                        totals["deduped_documents"] += 1
                        continue
                    seen_hashes.add(document["content_sha256"])
                    documents.append(document)
                    source_stats["documents"] += 1
                    totals["documents"] += 1
            summary_sources.append({"id": source["id"], **dict(sorted(source_stats.items()))})
            shutil.rmtree(temp_dir, ignore_errors=True)
            temp_dir.mkdir(parents=True, exist_ok=True)

    with output_jsonl.open("w", encoding="utf-8") as handle:
        for document in documents:
            handle.write(json.dumps(document, ensure_ascii=False) + "\n")

    summary = {
        "ok": True,
        "manifest": str(manifest_path),
        "output_jsonl": str(output_jsonl),
        "approved_source_count": len(sources),
        "document_count": len(documents),
        "stats": {"deduped_documents": totals["deduped_documents"]},
        "sources": summary_sources,
        "warnings": warnings,
    }
    output_summary.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
