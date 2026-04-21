from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from qwen35_workflow_trace_sft_manifest_utils import ensure


ROLE_ALIASES = {
    "assistant": "assistant",
    "gpt": "assistant",
    "model": "assistant",
    "bot": "assistant",
    "human": "user",
    "user": "user",
    "customer": "user",
    "system": "system",
}


def clean_text(value: Any) -> str:
    if value is None:
        return ""
    text = str(value).replace("\r\n", "\n").replace("\r", "\n").replace("\x00", "")
    return text.strip()


def normalize_role(value: Any) -> str | None:
    if value is None:
        return None
    return ROLE_ALIASES.get(str(value).strip().lower())


def merge_reasoning(reasoning: Any, content: Any, drop_tags: bool) -> str:
    reasoning_text = clean_text(reasoning)
    content_text = clean_text(content)
    if not reasoning_text:
        return content_text
    if drop_tags:
        return "\n\n".join(part for part in [reasoning_text, content_text] if part)
    if "<think>" in content_text.lower():
        return content_text
    if content_text:
        return f"<think>\n{reasoning_text}\n</think>\n\n{content_text}"
    return f"<think>\n{reasoning_text}\n</think>"


def choose_first_present(row: dict[str, Any], candidates: list[str]) -> str | None:
    for name in candidates:
        if clean_text(row.get(name)):
            return name
    return None


def make_messages_digest(messages: list[dict[str, str]]) -> str:
    payload = json.dumps(messages, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def estimate_tokens(messages: list[dict[str, str]]) -> int:
    chars = sum(len(message["content"]) for message in messages)
    return max(1, round(chars / 4))


def extract_response_only_text(messages: list[dict[str, str]]) -> str:
    responses = [message["content"] for message in messages if message["role"] == "assistant"]
    return "\n\n".join(response for response in responses if response)


def extract_row_metadata(row: dict[str, Any], excluded_keys: set[str]) -> dict[str, Any]:
    metadata = {key: value for key, value in row.items() if key not in excluded_keys}
    return json.loads(json.dumps(metadata, ensure_ascii=False, default=str))


def normalize_messages_strategy(row: dict[str, Any], normalization: dict[str, Any]) -> list[dict[str, str]]:
    raw_messages = row.get("messages")
    ensure(isinstance(raw_messages, list) and raw_messages, "messages strategy requires a non-empty messages list")
    drop_tags = bool(normalization.get("drop_explicit_reasoning_tags"))
    normalized: list[dict[str, str]] = []
    for message in raw_messages:
        ensure(isinstance(message, dict), "messages entries must be objects")
        role = normalize_role(message.get("role"))
        content = merge_reasoning(message.get("reasoning") or message.get("thinking"), message.get("content"), drop_tags)
        if role is None or not content:
            continue
        normalized.append({"role": role, "content": content})
    return normalized


def normalize_problem_solution_strategy(row: dict[str, Any], normalization: dict[str, Any]) -> list[dict[str, str]]:
    prompt = clean_text(row.get(normalization["prompt_column"]))
    reasoning = row.get(normalization.get("reasoning_column", ""))
    response = row.get(normalization["response_column"])
    assistant = merge_reasoning(reasoning, response, bool(normalization.get("drop_explicit_reasoning_tags")))
    return [{"role": "user", "content": prompt}, {"role": "assistant", "content": assistant}]


def normalize_conversations_strategy(row: dict[str, Any], normalization: dict[str, Any]) -> list[dict[str, str]]:
    raw_messages = row.get(normalization["conversation_column"])
    ensure(isinstance(raw_messages, list) and raw_messages, "conversations strategy requires a non-empty list")
    normalized: list[dict[str, str]] = []
    for message in raw_messages:
        ensure(isinstance(message, dict), "conversation entries must be objects")
        role = normalize_role(message.get("from") or message.get("role"))
        content = clean_text(message.get("value") or message.get("content"))
        if role is None or not content:
            continue
        normalized.append({"role": role, "content": content})
    return normalized


def normalize_prompt_response_strategy(row: dict[str, Any], normalization: dict[str, Any]) -> list[dict[str, str]]:
    prompt_column = choose_first_present(row, normalization["prompt_column_candidates"])
    response_column = choose_first_present(row, normalization["response_column_candidates"])
    ensure(prompt_column is not None, "prompt-response strategy could not resolve a prompt column")
    ensure(response_column is not None, "prompt-response strategy could not resolve a response column")
    system_prompt = clean_text(row.get("system"))
    prompt = clean_text(row.get(prompt_column))
    response = clean_text(row.get(response_column))
    messages: list[dict[str, str]] = []
    if system_prompt:
        messages.append({"role": "system", "content": system_prompt})
    messages.append({"role": "user", "content": prompt})
    messages.append({"role": "assistant", "content": response})
    return messages


def normalize_messages(row: dict[str, Any], source: dict[str, Any]) -> tuple[list[dict[str, str]], dict[str, Any]]:
    normalization = source["normalization"]
    strategy = normalization["strategy"]
    if strategy == "messages":
        messages = normalize_messages_strategy(row, normalization)
        excluded = {"messages"}
    elif strategy == "problem-solution-reasoning":
        messages = normalize_problem_solution_strategy(row, normalization)
        excluded = {
            normalization["prompt_column"],
            normalization["response_column"],
            normalization.get("reasoning_column", ""),
        }
    elif strategy == "conversations":
        messages = normalize_conversations_strategy(row, normalization)
        excluded = {normalization["conversation_column"]}
    elif strategy == "prompt-response":
        messages = normalize_prompt_response_strategy(row, normalization)
        excluded = {"system", *normalization["prompt_column_candidates"], *normalization["response_column_candidates"]}
    else:
        raise SystemExit(f"Unsupported normalization strategy: {strategy}")

    messages = [message for message in messages if message["content"]]
    ensure(messages, f"{source['id']}: normalized row is empty")
    ensure(any(message["role"] == "user" for message in messages), f"{source['id']}: normalized row is missing a user turn")
    ensure(any(message["role"] == "assistant" for message in messages), f"{source['id']}: normalized row is missing an assistant turn")
    metadata = extract_row_metadata(row, excluded)
    return messages, metadata


def build_example_record(
    manifest: dict[str, Any],
    source: dict[str, Any],
    config_name: str,
    split_name: str,
    row_index: int,
    messages: list[dict[str, str]],
    row_metadata: dict[str, Any],
) -> dict[str, Any]:
    response_only_text = extract_response_only_text(messages)
    contains_reasoning = any("<think>" in message["content"].lower() for message in messages if message["role"] == "assistant")
    return {
        "schema": "qwen35-workflow-trace-sft-example-v1",
        "round_id": manifest["round_id"],
        "source_id": source["id"],
        "hf_dataset": source["hf_dataset"],
        "group": source["group"],
        "commercial_status": source["commercial_status"],
        "normalization_strategy": source["normalization"]["strategy"],
        "config": config_name,
        "split": split_name,
        "row_index": row_index,
        "message_count": len(messages),
        "estimated_tokens": estimate_tokens(messages),
        "contains_reasoning": contains_reasoning,
        "response_only_text": response_only_text,
        "normalized_sha256": make_messages_digest(messages),
        "messages": messages,
        "source_row_metadata": row_metadata,
    }


def load_source_runtime_targets(metadata_path: Path) -> dict[str, dict[str, Any]]:
    if not metadata_path.exists():
        return {}
    payload = json.loads(metadata_path.read_text())
    sources = payload.get("sources", [])
    ensure(isinstance(sources, list), f"Invalid metadata payload: {metadata_path}")
    return {source["id"]: source for source in sources if isinstance(source, dict) and isinstance(source.get("id"), str)}
