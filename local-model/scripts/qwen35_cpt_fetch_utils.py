from __future__ import annotations

import hashlib
import html
import json
import re
from pathlib import Path
from typing import Any
from urllib.parse import urljoin, urlparse, urlsplit, urlunsplit
from urllib.request import Request, urlopen


USER_AGENT = "qwen35-cpt-fetcher/1.0"
TAG_RE = re.compile(r"<[^>]+>")
TITLE_RE = re.compile(r"(?is)<title[^>]*>(.*?)</title>")
SCRIPT_STYLE_RE = re.compile(r"(?is)<(script|style)[^>]*>.*?</\\1>")
LINK_RE = re.compile(r"""href=["']([^"'#]+)["']""", re.IGNORECASE)
ENV_ASSIGNMENT_PATTERN = re.compile(r"^\s*(?:export\s+)?([A-Z0-9_]+)\s*=\s*(.+?)\s*$")
JSON_SECRET_PATTERN = re.compile(r'^(?P<prefix>\s*"(?P<key>[^"]+)"\s*:\s*")(?P<value>[^"\n]+)(?P<suffix>".*)$')
SECRET_NAME_SEGMENTS = {"API", "KEY", "TOKEN", "SECRET", "PASSWORD", "ACCESS", "REFRESH", "CLIENT"}
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


def is_binary_blob(raw: bytes) -> bool:
    return b"\x00" in raw


def redact_secrets(text: str) -> tuple[str, int]:
    replacements = 0
    redacted_lines: list[str] = []
    for line in text.splitlines():
        env_match = ENV_ASSIGNMENT_PATTERN.match(line)
        if env_match:
            name, value = env_match.groups()
            if set(name.split("_")) & SECRET_NAME_SEGMENTS and len(value.strip("\"'")) >= 8 and not value.strip("\"'").isdigit():
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


def normalize_url(url: str) -> str:
    parts = urlsplit(url)
    return urlunsplit((parts.scheme, parts.netloc, parts.path, parts.query, ""))


def sanitize_segment(value: str) -> str:
    cleaned = re.sub(r"[^a-zA-Z0-9._-]+", "-", value.strip("/"))
    return cleaned.strip("-") or "root"


def html_to_text(raw_text: str) -> str:
    stripped = SCRIPT_STYLE_RE.sub(" ", raw_text)
    stripped = TAG_RE.sub(" ", stripped)
    return html.unescape(re.sub(r"\s+", " ", stripped)).strip()


def extract_links(raw_text: str, base_url: str) -> list[str]:
    links: list[str] = []
    for href in LINK_RE.findall(raw_text):
        absolute = normalize_url(urljoin(base_url, href))
        parsed = urlparse(absolute)
        if parsed.scheme not in {"http", "https"}:
            continue
        links.append(absolute)
    return links


def fetch_url(url: str, timeout: int = 20) -> dict[str, Any]:
    request = Request(url, headers={"User-Agent": USER_AGENT})
    with urlopen(request, timeout=timeout) as response:
        raw_bytes = response.read()
        raw_text = raw_bytes.decode("utf-8", errors="replace")
        final_url = normalize_url(response.geturl())
        content_type = response.headers.get("Content-Type", "")
        etag = response.headers.get("ETag", "")
        last_modified = response.headers.get("Last-Modified", "")
    is_html = "html" in content_type.lower() or "<html" in raw_text[:500].lower()
    title_match = TITLE_RE.search(raw_text) if is_html else None
    title = html.unescape(title_match.group(1).strip()) if title_match else final_url
    text = html_to_text(raw_text) if is_html else raw_text.strip()
    return {
        "final_url": final_url,
        "content_type": content_type,
        "etag": etag,
        "last_modified": last_modified,
        "title": title,
        "text": text,
        "links": extract_links(raw_text, final_url) if is_html else [],
    }


def is_meaningful_page(kind: str, title: str, text: str) -> bool:
    text_lower = text.lower()
    title_lower = title.lower()
    if kind == "issue-archive":
        return False
    if "redirecting" in title_lower:
        return False
    if "featureflags" in text_lower or ("skip to content" in text_lower and "github" in text_lower):
        return False
    if len(text) < 400:
        return False
    return True


def build_document(
    round_id: str,
    source_id: str,
    domain: str,
    path: str,
    relative_path: str,
    text: str,
    metadata_lines: list[str] | None = None,
) -> dict[str, Any]:
    content = text.strip()
    if metadata_lines:
        content = "\n".join([*metadata_lines, "", content]).strip()
    return {
        "schema": "qwen35-cpt-document-v1",
        "round_id": round_id,
        "source_id": source_id,
        "domain": domain,
        "path": path,
        "relative_path": relative_path,
        "content_sha256": hashlib.sha256(content.encode("utf-8")).hexdigest(),
        "char_count": len(content),
        "line_count": content.count("\n") + 1,
        "content": content,
    }


def external_relative_path(source_id: str, url: str, slot: int) -> str:
    parsed = urlparse(url)
    path = parsed.path or "/"
    parts = [sanitize_segment(parsed.netloc), *[sanitize_segment(part) for part in path.split("/") if part]]
    if not parts[-1].endswith(".txt"):
        parts[-1] = parts[-1] + ".txt"
    return "/".join(["external", source_id, f"{slot:03d}", *parts])
