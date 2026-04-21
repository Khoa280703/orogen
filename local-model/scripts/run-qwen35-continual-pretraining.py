#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import traceback
from pathlib import Path
from typing import Any

import torch
from peft import LoraConfig, get_peft_model, prepare_model_for_kbit_training
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    BitsAndBytesConfig,
    DataCollatorForLanguageModeling,
    Trainer,
    TrainingArguments,
)

from qwen35_continual_pretraining_train_utils import (
    build_continual_pretraining_dataset,
    split_continual_pretraining_dataset,
    summarize_continual_pretraining_dataset,
)
from qwen35_workflow_trace_sft_train_utils import load_json, select_compute_dtype


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


def write_report(report_output: Path, report: dict[str, Any]) -> None:
    report_output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="configs/qwen35-continual-pretraining-pilot-round-1.json")
    parser.add_argument("--config-override-file", default="")
    parser.add_argument("--config-override-json", default="")
    parser.add_argument("--train-jsonl", default="")
    parser.add_argument("--output-dir", default="")
    parser.add_argument("--report-output", default="output/qwen35-continual-pretraining-pilot-report.json")
    parser.add_argument("--sample-limit", type=int, default=None)
    parser.add_argument("--max-steps", type=int, default=None)
    parser.add_argument("--max-sequence-length", type=int, default=None)
    parser.add_argument("--gpu-id", type=int, default=0)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    os.environ.setdefault("TOKENIZERS_PARALLELISM", "false")
    workspace_root = Path(__file__).resolve().parent.parent
    config_path = Path(args.config).resolve()
    report_output = Path(args.report_output).resolve()
    report_output.parent.mkdir(parents=True, exist_ok=True)
    report: dict[str, Any] = {
        "ok": False,
        "config": str(config_path),
        "config_override_file": str(Path(args.config_override_file).resolve()) if args.config_override_file else "",
        "config_override_json_applied": bool(args.config_override_json),
        "train_jsonl": "",
        "output_dir": "",
        "dry_run": args.dry_run,
        "gpu_id": args.gpu_id,
        "cuda_available": torch.cuda.is_available(),
    }
    try:
        train_config = load_json(config_path)
        if args.config_override_file:
            train_config = merge_config(train_config, load_json(Path(args.config_override_file).resolve()))
        if args.config_override_json:
            try:
                override_payload = json.loads(args.config_override_json)
            except json.JSONDecodeError as exc:
                raise SystemExit(f"config_override_json contains invalid JSON: {exc}") from exc
            if not isinstance(override_payload, dict):
                raise SystemExit("config_override_json must decode to a JSON object")
            train_config = merge_config(train_config, override_payload)

        train_jsonl_value = args.train_jsonl or train_config["input_jsonl"]
        train_jsonl_path = resolve_workspace_path(workspace_root, train_jsonl_value)
        output_dir = (
            Path(args.output_dir).resolve()
            if args.output_dir else resolve_workspace_path(workspace_root, train_config["output_dir"])
        )
        output_dir.mkdir(parents=True, exist_ok=True)
        effective_sample_limit = args.sample_limit if args.sample_limit is not None else (16 if args.dry_run else None)
        effective_max_steps = args.max_steps if args.max_steps is not None else (1 if args.dry_run else -1)
        effective_epochs = 1.0 if args.dry_run else float(train_config["optimization"]["epochs"])
        effective_max_sequence_length = (
            args.max_sequence_length
            if args.max_sequence_length is not None
            else (512 if args.dry_run else int(train_config["sequence"]["max_sequence_length"]))
        )
        effective_grad_accum = (
            min(4, int(train_config["batching"]["gradient_accumulation_steps"]))
            if args.dry_run else int(train_config["batching"]["gradient_accumulation_steps"])
        )
        effective_eval_batch_size = int(
            train_config["batching"].get(
                "per_device_eval_batch_size",
                train_config["batching"]["per_device_train_batch_size"],
            )
        )
        effective_logging_steps = 1 if args.dry_run else int(train_config["logging"]["logging_steps"])
        effective_save_strategy = "no" if args.dry_run else "steps"
        effective_save_steps = max(1, int(train_config["logging"]["save_steps"]))
        effective_eval_strategy = "steps"
        effective_eval_steps = 1 if args.dry_run else int(train_config["logging"].get("eval_steps", effective_save_steps))
        effective_save_total_limit = max(1, int(train_config["logging"]["save_total_limit"]))
        gradient_checkpointing = bool(train_config["batching"]["gradient_checkpointing"])

        report.update(
            {
                "train_jsonl": str(train_jsonl_path),
                "output_dir": str(output_dir),
                "sample_limit": effective_sample_limit,
                "max_steps": effective_max_steps,
                "num_train_epochs": effective_epochs,
                "max_sequence_length": effective_max_sequence_length,
                "logging_steps": effective_logging_steps,
                "effective_gradient_accumulation_steps": effective_grad_accum,
                "per_device_eval_batch_size": effective_eval_batch_size,
                "save_strategy": effective_save_strategy,
                "save_steps": effective_save_steps,
                "save_total_limit": effective_save_total_limit,
                "eval_strategy": effective_eval_strategy,
                "eval_steps": effective_eval_steps,
                "gradient_checkpointing": gradient_checkpointing,
                "launch_environment": {
                    "cuda_visible_devices": os.environ.get("CUDA_VISIBLE_DEVICES", ""),
                    "visible_cuda_device_count": torch.cuda.device_count() if torch.cuda.is_available() else 0,
                },
            }
        )

        if torch.cuda.is_available():
            torch.cuda.set_device(args.gpu_id)
            if torch.cuda.device_count() != 1:
                raise SystemExit(
                    "This CPT pilot runner expects exactly 1 visible CUDA device. "
                    "Launch with CUDA_VISIBLE_DEVICES=<single-gpu>."
                )
        compute_dtype = select_compute_dtype()
        tokenizer_path = resolve_workspace_path(workspace_root, train_config["tokenizer_path"])
        base_model_path = resolve_workspace_path(workspace_root, train_config["base_model_path"])
        report.update(
            {
                "base_model_path": str(base_model_path),
                "tokenizer_path": str(tokenizer_path),
                "compute_dtype": str(compute_dtype),
            }
        )

        tokenizer = AutoTokenizer.from_pretrained(str(tokenizer_path), trust_remote_code=True)
        if tokenizer.pad_token is None:
            tokenizer.pad_token = tokenizer.eos_token

        dataset, dataset_build = build_continual_pretraining_dataset(
            train_jsonl_path,
            tokenizer,
            effective_max_sequence_length,
            effective_sample_limit,
            min_doc_chars=int(train_config["sequence"]["min_document_chars"]),
            seed=args.seed,
        )
        train_dataset, eval_dataset, dataset_split = split_continual_pretraining_dataset(
            dataset,
            holdout_ratio=float(train_config["evaluation"]["holdout_ratio"]),
            min_eval_blocks=int(train_config["evaluation"]["min_eval_blocks"]),
            seed=args.seed,
        )
        report.update(
            {
                "dataset_build": dataset_build,
                "dataset": summarize_continual_pretraining_dataset(dataset),
                "dataset_split": dataset_split,
                "train_dataset": summarize_continual_pretraining_dataset(train_dataset),
                "eval_dataset": summarize_continual_pretraining_dataset(eval_dataset),
            }
        )

        quantization_config = None
        if bool(train_config["quantization"]["load_in_4bit"]) and torch.cuda.is_available():
            quantization_config = BitsAndBytesConfig(
                load_in_4bit=True,
                bnb_4bit_quant_type=train_config["quantization"]["bnb_4bit_quant_type"],
                bnb_4bit_use_double_quant=train_config["quantization"]["bnb_4bit_use_double_quant"],
                bnb_4bit_compute_dtype=compute_dtype,
            )
        model = AutoModelForCausalLM.from_pretrained(
            str(base_model_path),
            trust_remote_code=True,
            quantization_config=quantization_config,
            torch_dtype=compute_dtype if torch.cuda.is_available() else torch.float32,
            device_map={"": args.gpu_id} if torch.cuda.is_available() else None,
        )
        model.config.use_cache = False
        if quantization_config is not None:
            model = prepare_model_for_kbit_training(
                model,
                use_gradient_checkpointing=gradient_checkpointing,
            )
        peft_config = LoraConfig(
            r=int(train_config["lora"]["r"]),
            lora_alpha=int(train_config["lora"]["alpha"]),
            lora_dropout=float(train_config["lora"]["dropout"]),
            bias=train_config["lora"]["bias"],
            target_modules=train_config["lora"]["target_modules"],
            task_type="CAUSAL_LM",
        )
        model = get_peft_model(model, peft_config)
        training_args = TrainingArguments(
            output_dir=str(output_dir),
            per_device_train_batch_size=int(train_config["batching"]["per_device_train_batch_size"]),
            per_device_eval_batch_size=effective_eval_batch_size,
            gradient_accumulation_steps=effective_grad_accum,
            learning_rate=float(train_config["optimization"]["learning_rate"]),
            lr_scheduler_type=train_config["optimization"]["lr_scheduler_type"],
            warmup_ratio=float(train_config["optimization"]["warmup_ratio"]),
            weight_decay=float(train_config["optimization"]["weight_decay"]),
            max_grad_norm=float(train_config["optimization"]["max_grad_norm"]),
            num_train_epochs=effective_epochs,
            max_steps=effective_max_steps,
            logging_steps=effective_logging_steps,
            save_strategy=effective_save_strategy,
            save_steps=effective_save_steps,
            eval_strategy=effective_eval_strategy,
            eval_steps=effective_eval_steps,
            save_total_limit=effective_save_total_limit,
            load_best_model_at_end=not args.dry_run,
            metric_for_best_model="eval_loss",
            greater_is_better=False,
            bf16=torch.cuda.is_available() and compute_dtype == torch.bfloat16,
            fp16=torch.cuda.is_available() and compute_dtype == torch.float16,
            gradient_checkpointing=gradient_checkpointing,
            report_to="none",
            seed=args.seed,
        )
        trainer = Trainer(
            model=model,
            args=training_args,
            train_dataset=train_dataset,
            eval_dataset=eval_dataset,
            data_collator=DataCollatorForLanguageModeling(tokenizer=tokenizer, mlm=False),
        )
        train_result = trainer.train()
        eval_result = trainer.evaluate()
        trainer.save_model(str(output_dir))
        tokenizer.save_pretrained(str(output_dir))
        report.update(
            {
                "ok": True,
                "train_result": train_result.metrics,
                "eval_result": eval_result,
            }
        )
    except BaseException as exc:  # noqa: BLE001
        if isinstance(exc, KeyboardInterrupt):
            raise
        report["error"] = str(exc)
        report["error_type"] = type(exc).__name__
        report["traceback"] = traceback.format_exc()
        write_report(report_output, report)
        print(json.dumps(report, ensure_ascii=False, indent=2))
        raise SystemExit(1) from exc

    write_report(report_output, report)
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
