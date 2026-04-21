#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Any

import torch
from peft import LoraConfig, TaskType
from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig
from trl import SFTConfig, SFTTrainer

from qwen35_workflow_trace_sft_train_utils import build_prompt_completion_dataset, load_json, select_compute_dtype, summarize_dataset


def merge_config(base: dict[str, Any], override: dict[str, Any]) -> dict[str, Any]:
    merged = dict(base)
    for key, value in override.items():
        if isinstance(value, dict) and isinstance(merged.get(key), dict):
            merged[key] = merge_config(merged[key], value)
        else:
            merged[key] = value
    return merged


def resolve_workspace_path(workspace_root: Path, value: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return (workspace_root / path).resolve()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="configs/qwen35-workflow-trace-sft-round-1.json")
    parser.add_argument("--config-override-file", default="")
    parser.add_argument("--config-override-json", default="")
    parser.add_argument("--train-jsonl", default="output/qwen35-workflow-trace-sft-search-ready-shard.jsonl")
    parser.add_argument("--output-dir", default="")
    parser.add_argument("--report-output", default="output/qwen35-workflow-trace-sft-dry-run-report.json")
    parser.add_argument("--sample-limit", type=int, default=None)
    parser.add_argument("--max-steps", type=int, default=None)
    parser.add_argument("--max-sequence-length", type=int, default=None)
    parser.add_argument("--gpu-id", type=int, default=0)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    os.environ.setdefault("TOKENIZERS_PARALLELISM", "false")
    config_path = Path(args.config).resolve()
    train_jsonl_path = Path(args.train_jsonl).resolve()
    report_output = Path(args.report_output).resolve()
    report_output.parent.mkdir(parents=True, exist_ok=True)
    workspace_root = Path(__file__).resolve().parent.parent

    train_config = load_json(config_path)
    if args.config_override_file:
        override_path = Path(args.config_override_file).resolve()
        train_config = merge_config(train_config, load_json(override_path))
    if args.config_override_json:
        try:
            override_payload = json.loads(args.config_override_json)
        except json.JSONDecodeError as exc:
            raise SystemExit(f"config_override_json contains invalid JSON: {exc}") from exc
        if not isinstance(override_payload, dict):
            raise SystemExit("config_override_json must decode to a JSON object")
        train_config = merge_config(train_config, override_payload)
    output_dir = (
        Path(args.output_dir).resolve()
        if args.output_dir else resolve_workspace_path(workspace_root, train_config["output_dir"])
    )
    output_dir.mkdir(parents=True, exist_ok=True)
    effective_sample_limit = args.sample_limit if args.sample_limit is not None else (32 if args.dry_run else None)
    effective_max_steps = args.max_steps if args.max_steps is not None else (1 if args.dry_run else -1)
    effective_num_train_epochs = 1.0 if args.dry_run else float(train_config["optimization"]["epochs"])
    effective_max_sequence_length = (
        args.max_sequence_length if args.max_sequence_length is not None else (
            2048 if args.dry_run else train_config["sequence"]["max_sequence_length"]
        )
    )
    effective_grad_accumulation = (
        min(4, train_config["batching"]["gradient_accumulation_steps"])
        if args.dry_run else train_config["batching"]["gradient_accumulation_steps"]
    )
    effective_logging_steps = 1 if args.dry_run else train_config["logging"]["logging_steps"]
    effective_save_strategy = "no" if args.dry_run else "steps"
    effective_save_steps = max(1, int(train_config["logging"]["save_steps"]))
    effective_save_total_limit = max(1, int(train_config["logging"]["save_total_limit"]))
    effective_eval_strategy = "no" if args.dry_run else train_config["logging"]["eval_strategy"]
    effective_gradient_checkpointing = (
        True if args.dry_run else bool(train_config["batching"]["gradient_checkpointing"])
    )

    if torch.cuda.is_available():
        torch.cuda.set_device(args.gpu_id)
    compute_dtype = select_compute_dtype()
    use_4bit = bool(train_config["quantization"]["load_in_4bit"]) and torch.cuda.is_available()
    quantization_config = None
    if use_4bit:
        quantization_config = BitsAndBytesConfig(
            load_in_4bit=True,
            bnb_4bit_quant_type=train_config["quantization"]["bnb_4bit_quant_type"],
            bnb_4bit_use_double_quant=train_config["quantization"]["bnb_4bit_use_double_quant"],
            bnb_4bit_compute_dtype=compute_dtype,
        )

    tokenizer_path = resolve_workspace_path(workspace_root, train_config["tokenizer_path"])
    base_model_path = resolve_workspace_path(workspace_root, train_config["base_model_path"])
    chat_template_path = resolve_workspace_path(workspace_root, train_config["sequence"]["chat_template_path"])
    tokenizer = AutoTokenizer.from_pretrained(str(tokenizer_path), trust_remote_code=True)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    dataset, dataset_build_stats = build_prompt_completion_dataset(
        train_jsonl_path,
        tokenizer,
        effective_sample_limit,
        seed=args.seed,
    )
    dataset_summary = summarize_dataset(dataset)
    model = AutoModelForCausalLM.from_pretrained(
        str(base_model_path),
        trust_remote_code=True,
        quantization_config=quantization_config,
        torch_dtype=compute_dtype if torch.cuda.is_available() else torch.float32,
        device_map={"": args.gpu_id} if torch.cuda.is_available() else None,
    )
    model.config.use_cache = False

    lora_config = LoraConfig(
        task_type=TaskType.CAUSAL_LM,
        r=train_config["lora"]["r"],
        lora_alpha=train_config["lora"]["alpha"],
        lora_dropout=train_config["lora"]["dropout"],
        bias=train_config["lora"]["bias"],
        target_modules=train_config["lora"]["target_modules"],
    )
    sft_config = SFTConfig(
        output_dir=str(output_dir),
        per_device_train_batch_size=train_config["batching"]["per_device_train_batch_size"],
        gradient_accumulation_steps=effective_grad_accumulation,
        learning_rate=train_config["optimization"]["learning_rate"],
        lr_scheduler_type=train_config["optimization"]["lr_scheduler_type"],
        warmup_ratio=train_config["optimization"]["warmup_ratio"],
        weight_decay=train_config["optimization"]["weight_decay"],
        max_grad_norm=train_config["optimization"]["max_grad_norm"],
        num_train_epochs=effective_num_train_epochs,
        max_steps=effective_max_steps,
        logging_steps=effective_logging_steps,
        save_strategy=effective_save_strategy,
        save_steps=effective_save_steps,
        save_total_limit=effective_save_total_limit,
        eval_strategy=effective_eval_strategy,
        report_to="none",
        bf16=torch.cuda.is_available() and compute_dtype == torch.bfloat16,
        fp16=torch.cuda.is_available() and compute_dtype == torch.float16,
        gradient_checkpointing=effective_gradient_checkpointing,
        max_length=effective_max_sequence_length,
        chat_template_path=str(chat_template_path),
        packing=False,
        completion_only_loss=True,
        dataset_num_proc=None,
        dataset_kwargs={"skip_prepare_dataset": False},
        seed=args.seed,
    )
    trainer = SFTTrainer(
        model=model,
        args=sft_config,
        train_dataset=dataset,
        processing_class=tokenizer,
        peft_config=lora_config,
    )
    train_result = trainer.train()
    trainer.save_model(str(output_dir))

    report = {
        "ok": True,
        "config": str(config_path),
        "config_override_file": str(Path(args.config_override_file).resolve()) if args.config_override_file else "",
        "config_override_json_applied": bool(args.config_override_json),
        "train_jsonl": str(train_jsonl_path),
        "output_dir": str(output_dir),
        "dry_run": args.dry_run,
        "sample_limit": effective_sample_limit,
        "max_steps": effective_max_steps,
        "num_train_epochs": effective_num_train_epochs,
        "max_sequence_length": effective_max_sequence_length,
        "logging_steps": effective_logging_steps,
        "save_strategy": effective_save_strategy,
        "save_steps": effective_save_steps,
        "save_total_limit": effective_save_total_limit,
        "eval_strategy": effective_eval_strategy,
        "gradient_checkpointing": effective_gradient_checkpointing,
        "gpu_id": args.gpu_id,
        "cuda_available": torch.cuda.is_available(),
        "base_model_path": str(base_model_path),
        "tokenizer_path": str(tokenizer_path),
        "chat_template_path": str(chat_template_path),
        "compute_dtype": str(compute_dtype),
        "dataset_build_stats": dataset_build_stats,
        "dataset": dataset_summary,
        "train_result": train_result.metrics,
    }
    report_output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
