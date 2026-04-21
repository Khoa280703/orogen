#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from qwen35_workflow_trace_sft_manifest_utils import ensure


def load_json(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text())
    except FileNotFoundError as exc:
        raise SystemExit(f"Search-wave config not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Search-wave config contains invalid JSON: {exc}") from exc
    ensure(isinstance(payload, dict), "Search-wave config must be a JSON object")
    return payload


def resolve_workspace_path(workspace_root: Path, value: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return (workspace_root / path).resolve()


def build_lane_command(
    lane: dict[str, Any],
    runner_path: Path,
    base_config: Path,
    train_jsonl: Path,
    workspace_root: Path,
) -> list[str]:
    ensure(isinstance(lane.get("id"), str) and lane["id"], "Each lane requires a non-empty id")
    ensure(isinstance(lane.get("gpu_id"), int) and lane["gpu_id"] >= 0, f"{lane['id']}: gpu_id must be >= 0")
    ensure(isinstance(lane.get("report_output"), str) and lane["report_output"], f"{lane['id']}: report_output required")
    ensure(isinstance(lane.get("config_override"), dict), f"{lane['id']}: config_override must be an object")
    config_override = json.loads(json.dumps(lane["config_override"]))
    output_dir = config_override.get("output_dir")
    if isinstance(output_dir, str) and output_dir:
        config_override["output_dir"] = str(resolve_workspace_path(workspace_root, output_dir))
    report_output = str(resolve_workspace_path(workspace_root, lane["report_output"]))
    command = [
        sys.executable,
        "-u",
        str(runner_path),
        "--config",
        str(base_config),
        "--train-jsonl",
        str(train_jsonl),
        "--gpu-id",
        "0",
        "--report-output",
        report_output,
        "--config-override-json",
        json.dumps(config_override, ensure_ascii=False),
    ]
    if isinstance(lane.get("sample_limit"), int):
        command.extend(["--sample-limit", str(lane["sample_limit"])])
    if isinstance(lane.get("max_steps"), int):
        command.extend(["--max-steps", str(lane["max_steps"])])
    if isinstance(lane.get("max_sequence_length"), int):
        command.extend(["--max-sequence-length", str(lane["max_sequence_length"])])
    if isinstance(lane.get("seed"), int):
        command.extend(["--seed", str(lane["seed"])])
    if bool(lane.get("dry_run")):
        command.append("--dry-run")
    return command


def lane_log_path(report_output: str) -> Path:
    report_path = Path(report_output)
    if report_path.suffix == ".json":
        return report_path.with_suffix(".log")
    return report_path.with_name(f"{report_path.name}.log")


def load_gpu_status() -> list[dict[str, Any]]:
    gpu_query = subprocess.run(
        [
            "nvidia-smi",
            "--query-gpu=index,memory.total,memory.used,utilization.gpu",
            "--format=csv,noheader,nounits",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    process_query = subprocess.run(
        [
            "nvidia-smi",
            "--query-compute-apps=gpu_uuid,pid,process_name,used_memory",
            "--format=csv,noheader",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    uuid_query = subprocess.run(
        [
            "nvidia-smi",
            "--query-gpu=index,uuid",
            "--format=csv,noheader,nounits",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    uuid_by_index: dict[int, str] = {}
    for line in uuid_query.stdout.splitlines():
        if not line.strip():
            continue
        index_text, uuid = [part.strip() for part in line.split(",", 1)]
        uuid_by_index[int(index_text)] = uuid
    processes_by_uuid: dict[str, list[dict[str, Any]]] = {}
    for line in process_query.stdout.splitlines():
        if not line.strip():
            continue
        uuid, pid_text, process_name, used_memory_text = [part.strip() for part in line.split(",", 3)]
        used_memory_mb = int(used_memory_text.replace("MiB", "").strip())
        processes_by_uuid.setdefault(uuid, []).append(
            {
                "pid": int(pid_text),
                "process_name": process_name,
                "used_memory_mb": used_memory_mb,
            }
        )
    gpu_status: list[dict[str, Any]] = []
    for line in gpu_query.stdout.splitlines():
        if not line.strip():
            continue
        index_text, total_text, used_text, util_text = [part.strip() for part in line.split(",", 3)]
        index = int(index_text)
        total_mb = int(total_text)
        used_mb = int(used_text)
        gpu_status.append(
            {
                "gpu_id": index,
                "memory_total_mb": total_mb,
                "memory_used_mb": used_mb,
                "memory_free_mb": total_mb - used_mb,
                "utilization_gpu_pct": int(util_text),
                "processes": processes_by_uuid.get(uuid_by_index.get(index, ""), []),
            }
        )
    return gpu_status


def normalize_process_name(process_name: str) -> str:
    candidate = process_name.strip()
    if not candidate:
        return ""
    return Path(candidate).name or candidate


def filter_gpu_processes(
    processes: list[dict[str, Any]],
    ignored_process_names: set[str],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    if not ignored_process_names:
        return processes, []
    active: list[dict[str, Any]] = []
    ignored: list[dict[str, Any]] = []
    for process in processes:
        process_name = str(process.get("process_name", ""))
        normalized_name = normalize_process_name(process_name)
        if process_name in ignored_process_names or normalized_name in ignored_process_names:
            ignored.append(process)
            continue
        active.append(process)
    return active, ignored


def load_lane_report(report_path: Path) -> tuple[dict[str, Any] | None, str | None]:
    try:
        payload = json.loads(report_path.read_text())
    except FileNotFoundError:
        return None, "Lane report was not written"
    except json.JSONDecodeError as exc:
        return None, f"Lane report is not valid JSON: {exc}"
    if not isinstance(payload, dict):
        return None, "Lane report must be a JSON object"
    return payload, None


def resolve_summary_output_path(summary_output: Path, validate_only: bool) -> Path:
    if not validate_only:
        return summary_output
    suffix = summary_output.suffix or ".json"
    stem = summary_output.name[: -len(suffix)] if summary_output.suffix else summary_output.name
    return summary_output.with_name(f"{stem}-validate-only{suffix}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="configs/qwen35-workflow-trace-sft-search-wave-1.json")
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()

    wave_config_path = Path(args.config).resolve()
    wave_config = load_json(wave_config_path)
    ensure(wave_config.get("format") == "qwen35-sft-search-wave-v1", "Unsupported search-wave config format")
    ensure(isinstance(wave_config.get("lanes"), list) and wave_config["lanes"], "Search-wave config must contain lanes")
    lane_ids = [lane.get("id") for lane in wave_config["lanes"]]
    ensure(
        len(lane_ids) == len(set(lane_ids)),
        "Search-wave lane ids must be unique",
    )
    lane_gpu_ids = [lane.get("gpu_id") for lane in wave_config["lanes"]]
    ensure(
        len(lane_gpu_ids) == len(set(lane_gpu_ids)),
        "Search-wave lanes must not share the same physical gpu_id",
    )

    workspace_root = Path(__file__).resolve().parent.parent
    base_config = resolve_workspace_path(workspace_root, wave_config["base_config"])
    train_jsonl = resolve_workspace_path(workspace_root, wave_config["train_jsonl"])
    summary_output = resolve_summary_output_path(
        resolve_workspace_path(workspace_root, wave_config["summary_output"]),
        args.validate_only,
    )
    summary_output.parent.mkdir(parents=True, exist_ok=True)
    runner_path = Path(__file__).resolve().with_name("run-qwen35-workflow-trace-sft.py")
    ensure(base_config.exists(), f"Base config not found: {base_config}")
    ensure(train_jsonl.exists(), f"Training shard not found: {train_jsonl}")
    ensure(runner_path.exists(), f"Runner not found: {runner_path}")
    min_free_memory_mb = int(wave_config.get("min_free_memory_mb", 8192))
    require_exclusive_gpu = bool(wave_config.get("require_exclusive_gpu", True))
    ignored_process_names_raw = wave_config.get("ignored_process_names", [])
    ensure(
        isinstance(ignored_process_names_raw, list)
        and all(isinstance(item, str) and item.strip() for item in ignored_process_names_raw),
        "ignored_process_names must be a list[str] when provided",
    )
    ignored_process_names = {item.strip() for item in ignored_process_names_raw}
    gpu_status_by_id = {item["gpu_id"]: item for item in load_gpu_status()}

    lane_summaries: list[dict[str, Any]] = []
    processes: list[tuple[dict[str, Any], subprocess.Popen[bytes], Any, float, Path]] = []
    blocked_lane_ids: list[str] = []
    for lane in wave_config["lanes"]:
        command = build_lane_command(lane, runner_path, base_config, train_jsonl, workspace_root)
        report_output = str(resolve_workspace_path(workspace_root, lane["report_output"]))
        log_path = lane_log_path(report_output).resolve()
        log_path.parent.mkdir(parents=True, exist_ok=True)
        lane_min_free_memory_mb = int(lane.get("min_free_memory_mb", min_free_memory_mb))
        lane_summary = {
            "id": lane["id"],
            "gpu_id": lane["gpu_id"],
            "physical_gpu_id": lane["gpu_id"],
            "masked_runner_gpu_id": 0,
            "cuda_visible_devices": str(lane["gpu_id"]),
            "notes": lane.get("notes", ""),
            "seed": lane.get("seed"),
            "report_output": report_output,
            "log_path": str(log_path),
            "command": command,
            "execution_state": "pending",
            "min_free_memory_mb_required": lane_min_free_memory_mb,
        }
        gpu_status = gpu_status_by_id.get(lane["gpu_id"])
        ensure(gpu_status is not None, f"{lane['id']}: GPU {lane['gpu_id']} not found in nvidia-smi output")
        active_processes, ignored_processes = filter_gpu_processes(gpu_status["processes"], ignored_process_names)
        effective_gpu_status = dict(gpu_status)
        effective_gpu_status["processes"] = active_processes
        lane_summary["gpu_status"] = effective_gpu_status
        if ignored_processes:
            lane_summary["gpu_ignored_processes"] = ignored_processes
        lane_summary["gpu_preflight_ok"] = (
            gpu_status["memory_free_mb"] >= lane_min_free_memory_mb
            and (not require_exclusive_gpu or not active_processes)
        )
        if gpu_status["memory_free_mb"] < lane_min_free_memory_mb:
            lane_summary["gpu_preflight_reason"] = (
                f"GPU {lane['gpu_id']} only has {gpu_status['memory_free_mb']} MiB free; "
                f"requires >= {lane_min_free_memory_mb} MiB"
            )
        elif require_exclusive_gpu and active_processes:
            lane_summary["gpu_preflight_reason"] = (
                f"GPU {lane['gpu_id']} already has compute processes: "
                + ", ".join(
                    f"{process['process_name']}[{process['pid']}] {process['used_memory_mb']} MiB"
                    for process in active_processes
                )
            )
        lane_summaries.append(lane_summary)
    preflight_ok = all(lane.get("gpu_preflight_ok", True) for lane in lane_summaries)
    blocked_lane_ids = [str(lane["id"]) for lane in lane_summaries if not lane.get("gpu_preflight_ok", True)]
    launch_blocked = not preflight_ok
    if launch_blocked:
        blocked_state = "would-be-blocked-by-wave-preflight" if args.validate_only else "blocked-by-wave-preflight"
        blocked_lane_ids = [str(lane["id"]) for lane in lane_summaries if not lane.get("gpu_preflight_ok", True)]
        for lane_summary in lane_summaries:
            lane_summary["execution_state"] = blocked_state
    elif not args.validate_only:
        for lane_summary in lane_summaries:
            env = os.environ.copy()
            env["CUDA_VISIBLE_DEVICES"] = lane_summary["cuda_visible_devices"]
            report_path = Path(lane_summary["report_output"])
            report_path.unlink(missing_ok=True)
            log_path = Path(lane_summary["log_path"])
            log_path.unlink(missing_ok=True)
            log_handle = log_path.open("w", encoding="utf-8")
            started = time.time()
            process = subprocess.Popen(lane_summary["command"], stdout=log_handle, stderr=subprocess.STDOUT, env=env)
            lane_summary["execution_state"] = "running"
            processes.append((lane_summary, process, log_handle, started, log_path))
        for lane_summary, process, log_handle, started, _log_path in processes:
            return_code = process.wait()
            log_handle.close()
            lane_summary["return_code"] = return_code
            lane_summary["duration_s"] = round(time.time() - started, 3)
            lane_summary["ok"] = return_code == 0
            lane_summary["execution_state"] = "completed"
            report_path = Path(lane_summary["report_output"])
            lane_report, lane_report_error = load_lane_report(report_path)
            if lane_report is not None:
                lane_summary["report"] = lane_report
            if lane_report_error is not None:
                lane_summary["report_error"] = lane_report_error

    execution_ok = (
        (not args.validate_only)
        and preflight_ok
        and all(lane.get("ok", False) for lane in lane_summaries)
        and all("report" in lane and "report_error" not in lane for lane in lane_summaries)
    )
    validation_ok = preflight_ok
    status = "validate-only-pass" if args.validate_only and validation_ok else "validate-only-fail"
    if not args.validate_only:
        status = "completed-ok" if execution_ok else "blocked-preflight" if launch_blocked else "completed-with-failures"
    report = {
        "ok": execution_ok,
        "status": status,
        "config": str(wave_config_path),
        "wave_id": wave_config["wave_id"],
        "description": wave_config.get("description", ""),
        "validate_only": args.validate_only,
        "base_config": str(base_config),
        "train_jsonl": str(train_jsonl),
        "min_free_memory_mb": min_free_memory_mb,
        "require_exclusive_gpu": require_exclusive_gpu,
        "ignored_process_names": sorted(ignored_process_names),
        "validation_ok": validation_ok,
        "execution_ok": execution_ok,
        "execution_attempted": not args.validate_only and preflight_ok,
        "wave_launch_blocked": launch_blocked,
        "blocked_lane_ids": blocked_lane_ids,
        "gpu_preflight_ok": preflight_ok,
        "lane_count": len(lane_summaries),
        "lanes": lane_summaries,
    }
    summary_output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    if not args.validate_only and not preflight_ok:
        raise SystemExit("GPU preflight failed for one or more search-wave lanes.")


if __name__ == "__main__":
    main()
