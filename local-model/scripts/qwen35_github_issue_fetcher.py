from __future__ import annotations

import html
import json
import re
from pathlib import Path
from typing import Any
from urllib.parse import parse_qsl, urlencode, urljoin, urlparse, urlsplit, urlunsplit
from urllib.request import Request, urlopen

from qwen35_cpt_fetch_utils import build_document, normalize_url, redact_secrets


ISSUE_LINK_RE = re.compile(r'^/([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+)/issues/(\d+)$')
JSON_LD_RE = re.compile(r'(?is)<script[^>]+type=["\']application/ld\+json["\'][^>]*>(.*?)</script>')


def fetch_raw_page(url: str, timeout: int = 20) -> dict[str, str]:
    request = Request(url, headers={"User-Agent": "qwen35-issue-fetcher/1.0"})
    with urlopen(request, timeout=timeout) as response:
        raw = response.read().decode("utf-8", errors="replace")
        return {
            "final_url": normalize_url(response.geturl()),
            "etag": response.headers.get("ETag", ""),
            "last_modified": response.headers.get("Last-Modified", ""),
            "raw_html": raw,
        }


def add_closed_filter(url: str) -> tuple[str, bool]:
    parts = urlsplit(url)
    query_pairs = parse_qsl(parts.query, keep_blank_values=True)
    updated = []
    changed = False
    for key, value in query_pairs:
        if key == "q" and "is:closed" not in value:
            value = f"{value} is:closed".strip()
            changed = True
        updated.append((key, value))
    if not any(key == "q" for key, _ in updated):
        updated.append(("q", "is:issue is:closed"))
        changed = True
    return urlunsplit((parts.scheme, parts.netloc, parts.path, urlencode(updated), "")), changed


def extract_issue_urls(search_html: str, owner: str, repo: str, max_issues: int) -> list[str]:
    results: list[str] = []
    seen: set[str] = set()
    for href in re.findall(r"""href=["']([^"'#]+)["']""", search_html, re.IGNORECASE):
        match = ISSUE_LINK_RE.match(href)
        if not match:
            continue
        if match.group(1).lower() != owner.lower() or match.group(2).lower() != repo.lower():
            continue
        absolute = normalize_url(urljoin("https://github.com", href))
        if absolute in seen:
            continue
        seen.add(absolute)
        results.append(absolute)
        if len(results) >= max_issues:
            break
    return results


def iter_json_ld_objects(raw_html: str) -> list[dict[str, Any]]:
    objects: list[dict[str, Any]] = []
    for payload in JSON_LD_RE.findall(raw_html):
        try:
            parsed = json.loads(html.unescape(payload.strip()))
        except json.JSONDecodeError:
            continue
        if isinstance(parsed, dict):
            objects.append(parsed)
        elif isinstance(parsed, list):
            objects.extend(item for item in parsed if isinstance(item, dict))
    return objects


def extract_discussion_payload(raw_html: str) -> dict[str, Any] | None:
    for candidate in iter_json_ld_objects(raw_html):
        if candidate.get("@type") == "DiscussionForumPosting":
            return candidate
    return None


def normalize_issue_markdown(markdown: str) -> str:
    text = html.unescape(markdown).replace("\r\n", "\n").replace("\r", "\n").strip()
    return re.sub(r"\n{3,}", "\n\n", text)


def build_issue_document(
    round_id: str,
    source_id: str,
    domain: str,
    seed_url: str,
    query_url: str,
    issue_url: str,
    repo_slug: str,
    issue_number: str,
    title: str,
    body: str,
    etag: str,
    last_modified: str,
) -> dict[str, Any]:
    metadata = [
        f"Seed URL: {seed_url}",
        f"Query URL: {query_url}",
        f"Issue URL: {issue_url}",
        f"Issue Number: {issue_number}",
        f"Title: {title}",
    ]
    if etag:
        metadata.append(f"ETag: {etag}")
    if last_modified:
        metadata.append(f"Last-Modified: {last_modified}")
    return build_document(
        round_id=round_id,
        source_id=source_id,
        domain=domain,
        path=issue_url,
        relative_path=f"external/{source_id}/issues/{repo_slug}/{issue_number}.md",
        text=body,
        metadata_lines=metadata,
    )


def fetch_issue_documents(source: dict[str, Any], seed_url: str, max_issues: int) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    parsed = urlparse(seed_url)
    parts = [part for part in parsed.path.split("/") if part]
    if len(parts) < 3 or parts[2] != "issues":
        raise SystemExit(f"Unsupported GitHub issue seed URL: {seed_url}")
    owner, repo = parts[0], parts[1]
    query_url, closed_filter_added = add_closed_filter(seed_url)
    search_page = fetch_raw_page(query_url)
    issue_urls = extract_issue_urls(search_page["raw_html"], owner, repo, max_issues)
    documents: list[dict[str, Any]] = []
    skipped_short = 0
    for issue_url in issue_urls:
        issue_page = fetch_raw_page(issue_url)
        payload = extract_discussion_payload(issue_page["raw_html"])
        if not payload:
            continue
        headline = str(payload.get("headline", "")).strip()
        article_body = normalize_issue_markdown(str(payload.get("articleBody", "")).strip())
        article_body, _ = redact_secrets(article_body)
        if len(article_body) < 300:
            skipped_short += 1
            continue
        issue_number = issue_url.rstrip("/").split("/")[-1]
        documents.append(build_issue_document(
            round_id=source["round_id"],
            source_id=source["id"],
            domain=source["domain"],
            seed_url=seed_url,
            query_url=search_page["final_url"],
            issue_url=issue_page["final_url"],
            repo_slug=f"{owner}--{repo}",
            issue_number=issue_number,
            title=headline,
            body=article_body,
            etag=issue_page["etag"],
            last_modified=issue_page["last_modified"],
        ))
    stats = {
        "query_url": search_page["final_url"],
        "closed_filter_added": closed_filter_added,
        "issue_candidates": len(issue_urls),
        "documents": len(documents),
        "skipped_short_issues": skipped_short,
    }
    return documents, stats
