#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import html
import json
import re
import subprocess
from pathlib import Path
from typing import Any
from urllib.error import URLError, HTTPError
from urllib.parse import urlparse
from urllib.request import Request, urlopen


TAG_RE = re.compile(r"<[^>]+>")
TITLE_RE = re.compile(r"(?is)<title[^>]*>(.*?)</title>")


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


def github_repo_ref(url: str) -> str | None:
    parsed = urlparse(url)
    if parsed.netloc != "github.com":
        return None
    parts = [part for part in parsed.path.split("/") if part]
    if len(parts) != 2 or parsed.query:
        return None
    repo_url = f"https://github.com/{parts[0]}/{parts[1]}"
    result = subprocess.run(
        ["git", "ls-remote", repo_url, "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    )
    head = result.stdout.strip().split()
    ensure(head, f"Could not resolve git HEAD for {repo_url}")
    return f"git:{head[0]}"


def fetch_web_page(url: str) -> dict[str, str]:
    request = Request(url, headers={"User-Agent": "qwen35-cpt-metadata-builder/1.0"})
    with urlopen(request, timeout=20) as response:
        raw = response.read().decode("utf-8", errors="replace")
        final_url = response.geturl()
        etag = response.headers.get("ETag", "")
        last_modified = response.headers.get("Last-Modified", "")
    title_match = TITLE_RE.search(raw)
    title = html.unescape(title_match.group(1).strip()) if title_match else final_url
    text = TAG_RE.sub(" ", raw)
    text = html.unescape(re.sub(r"\s+", " ", text)).strip()
    return {
        "final_url": final_url,
        "title": title,
        "etag": etag,
        "last_modified": last_modified,
        "excerpt": text[:12000],
    }


def is_meaningful_page(source: dict[str, Any], page: dict[str, str]) -> bool:
    text = page["excerpt"].lower()
    title = page["title"].lower()
    if source["kind"] == "issue-archive":
        return False
    if "redirecting" in title:
        return False
    if "featureflags" in text or "skip to content" in text and "github" in text:
        return False
    if len(text) < 400:
        return False
    return True


def normalize_doc(round_id: str, source_id: str, domain: str, slot: int, uri: str, page: dict[str, str]) -> dict[str, Any]:
    content = "\n".join(
        line for line in [
            f"Source URL: {uri}",
            f"Final URL: {page['final_url']}",
            f"Title: {page['title']}",
            f"ETag: {page['etag']}" if page["etag"] else "",
            f"Last-Modified: {page['last_modified']}" if page["last_modified"] else "",
            "",
            page["excerpt"],
        ] if line
    )
    return {
        "schema": "qwen35-cpt-document-v1",
        "round_id": round_id,
        "source_id": source_id,
        "domain": domain,
        "path": uri,
        "relative_path": f"external/{source_id}/{slot:02d}.txt",
        "content_sha256": hashlib.sha256(content.encode("utf-8")).hexdigest(),
        "char_count": len(content),
        "line_count": content.count("\n") + 1,
        "content": content,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", default="corpora/qwen35-continual-pretraining-round-1-corpus-manifest.json")
    parser.add_argument("--output-manifest", default="output/qwen35-external-cpt-pinned-manifest-candidate.json")
    parser.add_argument("--output-jsonl", default="output/qwen35-external-cpt-metadata-shard.jsonl")
    parser.add_argument("--output-summary", default="output/qwen35-external-cpt-metadata-summary.json")
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    manifest = load_manifest(manifest_path)
    ensure(manifest.get("format") == "qwen35-continual-pretraining-corpus-v1", "Unsupported manifest format.")

    output_manifest = Path(args.output_manifest).resolve()
    output_jsonl = Path(args.output_jsonl).resolve()
    output_summary = Path(args.output_summary).resolve()
    output_manifest.parent.mkdir(parents=True, exist_ok=True)
    output_jsonl.parent.mkdir(parents=True, exist_ok=True)
    output_summary.parent.mkdir(parents=True, exist_ok=True)

    pinned_manifest = json.loads(json.dumps(manifest))
    documents: list[dict[str, Any]] = []
    summary_sources: list[dict[str, Any]] = []
    warnings: list[str] = []

    for source in pinned_manifest.get("sources", []):
        provenance = source.get("provenance", {})
        if provenance.get("type") != "public":
            continue
        resolved_refs: list[str] = []
        strong_refs = 0
        weak_refs = 0
        fetched = 0
        skipped_documents = 0
        failed_uris = 0
        for index, uri in enumerate(source.get("uris", []), start=1):
            try:
                repo_ref = github_repo_ref(uri)
                if repo_ref is not None:
                    resolved_refs.append(f"{uri} -> {repo_ref}")
                    strong_refs += 1
                    continue
                page = fetch_web_page(uri)
            except (subprocess.CalledProcessError, URLError, HTTPError, TimeoutError, OSError) as exc:
                failed_uris += 1
                warnings.append(f"{source['id']}: failed to resolve {uri}: {type(exc).__name__}")
                continue
            ref_parts = [page["final_url"]]
            if page["etag"]:
                ref_parts.append(f"etag={page['etag']}")
                strong_refs += 1
            elif page["last_modified"]:
                ref_parts.append(f"last-modified={page['last_modified']}")
                strong_refs += 1
            else:
                weak_refs += 1
            resolved_refs.append("url:" + "#".join(ref_parts))
            if is_meaningful_page(source, page):
                documents.append(normalize_doc(manifest["round_id"], source["id"], source["domain"], index, uri, page))
                fetched += 1
            else:
                skipped_documents += 1
        provenance["reference"] = "candidate-pin-generated-from-current-public-metadata-review"
        provenance["resolved_references"] = resolved_refs
        provenance["ready"] = False
        summary_sources.append(
            {
                "id": source["id"],
                "uri_count": len(source.get("uris", [])),
                "resolved_reference_count": len(resolved_refs),
                "strong_reference_count": strong_refs,
                "weak_reference_count": weak_refs,
                "failed_uri_count": failed_uris,
                "metadata_documents": fetched,
                "skipped_documents": skipped_documents,
            }
        )

    with output_jsonl.open("w", encoding="utf-8") as handle:
        for document in documents:
            handle.write(json.dumps(document, ensure_ascii=False) + "\n")
    output_manifest.write_text(json.dumps(pinned_manifest, ensure_ascii=False, indent=2) + "\n")

    summary = {
        "ok": True,
        "manifest": str(manifest_path),
        "output_manifest": str(output_manifest),
        "output_jsonl": str(output_jsonl),
        "public_source_count": len(summary_sources),
        "metadata_document_count": len(documents),
        "sources": summary_sources,
        "warnings": warnings,
    }
    output_summary.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
