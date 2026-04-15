# Research Report: Qwen3.5-4B train, tune, RAG để kéo gần kiểu Opus

**Ngày nghiên cứu:** 2026-04-15  
**Scope:** shortlist repo, dataset, model card, docs cho `Qwen3.5-4B` theo 3 lane: `reasoning distill`, `post-train practical`, `RAG`.

## Executive Summary

Muốn `Qwen3.5-4B` "gần Opus" theo nghĩa rộng là không thực tế. Muốn nó gần hơn trong scope hẹp như coding trên codebase riêng, phân tích docs nội bộ, reasoning có cấu trúc, trả lời có dẫn nguồn, thì có đường đi rõ.

Đường đi tốt nhất không phải chỉ nhồi thêm distill traces từ Opus. Nên đi tổ hợp:

1. `SFT` với dataset distill chất lượng cao từ Opus/Sonnet.
2. trộn thêm `reasoning/coding/math` open dataset để tránh overfit vào style riêng của Claude.
3. dùng `RAG` đúng bài với `embedding + reranker` cùng họ Qwen.
4. đo bằng benchmark nội bộ và task thực, không nhìn mỗi vài prompt đẹp.

## Methodology

- Search web trên Hugging Face, GitHub, Qwen docs.
- Ưu tiên official docs và model/data cards.
- Kiểm tra ngày: 2026-04-15.

## Key Findings

### 1. Core distill lane gần use case nhất

Nguồn anh đã đưa là đúng hướng. Sau khi rà thêm, bộ nên lấy đầu tiên:

- `TeichAI/Qwen3.5-4B-Claude-Opus-Reasoning-Distill`
  - Model card hữu ích để học trade-off distill.
  - Có báo benchmark tăng ở `IFEval`, `ARC`, `Winogrande`, nhưng tụt ở vài chỉ số factual như `MMLU`.
  - Link: https://huggingface.co/TeichAI/Qwen3.5-4B-Claude-Opus-Reasoning-Distill

- `Farseen0/opus-4.6-reasoning-sft-12k`
  - Bản hợp nhất khoảng `12k` mẫu reasoning từ 4 nguồn, đã chuẩn hóa `messages`.
  - Reasoning được bọc trong `<think>...</think>`, rất hợp train response-only.
  - Link: https://huggingface.co/datasets/Farseen0/opus-4.6-reasoning-sft-12k

- `Roman1111111/claude-opus-4.6-10000x`
  - Distill corpus lớn, khoảng `9.6k` mẫu theo data card.
  - Hợp làm bulk corpus nếu cần nhiều data hơn `TeichAI`.
  - Link: https://huggingface.co/datasets/Roman1111111/claude-opus-4.6-10000x

- `nohurry/Opus-4.6-Reasoning-3000x-filtered`
  - Bản filtered đáng tin hơn raw mirror.
  - Hợp để làm "quality-topup" cho curriculum.
  - Link: https://huggingface.co/datasets/nohurry/Opus-4.6-Reasoning-3000x-filtered

- `TeichAI/Claude-Opus-4.6-Reasoning-887x`
  - Có non-tool và tool-use traces.
  - Rất đáng dùng nếu mục tiêu là agent/coding assistant chứ không chỉ QA.
  - Link: https://huggingface.co/datasets/TeichAI/Claude-Opus-4.6-Reasoning-887x

- `TeichAI/Claude-Sonnet-4.6-Reasoning-1100x`
  - Bổ sung reasoning thiên analytical/general hơn.
  - Link: https://huggingface.co/datasets/TeichAI/Claude-Sonnet-4.6-Reasoning-1100x

- `TeichAI/claude-4.5-opus-high-reasoning-250x`
  - Ít mẫu nhưng đậm reasoning. Hợp làm high-quality capstone set.
  - Link: https://huggingface.co/datasets/TeichAI/claude-4.5-opus-high-reasoning-250x

### 2. Nguồn mở để tránh model thành "Opus style parroting"

Nếu chỉ train trên distill Claude, model sẽ dễ bị:

- mạnh ở format/suy luận bề mặt
- yếu factual breadth
- yếu coding/math thật
- dễ verbose, dài dòng, hoặc lặp

Nên trộn thêm:

- `open-r1/OpenR1-Math-Raw`
  - Corpus math reasoning lớn, verifier-backed.
  - Hợp cho SFT lane khó hoặc GRPO về sau.
  - Link: https://huggingface.co/datasets/open-r1/OpenR1-Math-Raw

- `open-thoughts/OpenThoughts3-1.2M`
  - Reasoning corpus lớn cho math, code, science.
  - Link: https://huggingface.co/datasets/open-thoughts/OpenThoughts3-1.2M

- `open-thoughts/OpenThoughts-Agent-v1-SFT`
  - Hợp nếu muốn model làm việc kiểu agent, tool-use, terminal, debugging.
  - Link: https://huggingface.co/datasets/open-thoughts/OpenThoughts-Agent-v1-SFT

- `OpenDataArena/ODA-Math-460k`
  - Quan trọng ở chỗ pipeline lọc/verifier tốt; đáng học cách curate hơn là chỉ lấy raw.
  - Dataset: https://huggingface.co/datasets/OpenDataArena/ODA-Math-460k
  - Model card tham chiếu: https://huggingface.co/OpenDataArena/Qwen3-8B-ODA-Math-460k

- `mrm8488/FineTome-single-turn`
  - Dùng để giữ chat quality và bớt bias sang CoT-only.
  - Link: https://huggingface.co/datasets/mrm8488/FineTome-single-turn

### 3. Official tuning lane của Qwen

- Qwen official Unsloth guide
  - Có hướng dẫn finetune Qwen với Unsloth.
  - Có khuyến nghị thực dụng: trộn khoảng `75% reasoning` và `25% non-reasoning` để giữ ability mà không phá quality hội thoại.
  - Link: https://qwen.readthedocs.io/en/latest/training/unsloth.html

- Qwen official Axolotl guide
  - Hợp hơn nếu đi xa tới `RLHF / RM / PRM`.
  - Link: https://qwen.readthedocs.io/en/latest/training/axolotl.html

- `modelscope/easydistill`
  - Toolkit đáng xem nếu muốn distill teacher-student bài bản hơn SFT thường.
  - Link: https://github.com/modelscope/easydistill

### 4. RAG lane sẽ cho hiệu quả thực hơn train thêm weights

Nếu use case chính là trả lời trên tài liệu, codebase, spec nội bộ thì `RAG` thường cho gain lớn hơn thêm vài nghìn mẫu SFT.

- `QwenLM/Qwen3-Embedding`
  - Official repo cho embedding/reranker của Qwen3.
  - Có embedding model `0.6B/4B/8B` và reranker `0.6B/4B/8B`.
  - Có ghi chú dùng instruction có thể cải thiện retrieval thêm khoảng `1%–5%`.
  - Link: https://github.com/QwenLM/Qwen3-Embedding

- `Qwen/Qwen3-Embedding-4B`
  - Official embedding model để làm retrieval lane cùng họ Qwen.
  - Link: https://huggingface.co/Qwen/Qwen3-Embedding-4B

- `PA-RAG`
  - Public pipeline cho preference alignment in RAG.
  - Có release `58.9k` SFT data và preference data cho `informativeness`, `robustness`, `citation quality`.
  - Link: https://github.com/wujwyi/PA-RAG

- `PotatoHD404/QwenRag`
  - Repo thực dụng, dùng luôn `Qwen3-Embedding-4B` và `Qwen3-Reranker-4B`.
  - Hợp để học stack chạy thật.
  - Link: https://github.com/PotatoHD404/QwenRag

## Comparative Analysis

### A. Top dataset nên tải ngay

1. `Farseen0/opus-4.6-reasoning-sft-12k`
   - Lý do: sạch, chuẩn hóa, sát target.
2. `Roman1111111/claude-opus-4.6-10000x`
   - Lý do: tăng volume.
3. `nohurry/Opus-4.6-Reasoning-3000x-filtered`
   - Lý do: top-up quality.
4. `TeichAI/Claude-Opus-4.6-Reasoning-887x`
   - Lý do: tool-use traces.
5. `open-thoughts/OpenThoughts3-1.2M`
   - Lý do: tránh style overfit, tăng reasoning breadth.
6. `mrm8488/FineTome-single-turn`
   - Lý do: giữ chat quality.

### B. Top repo nên đọc kỹ

1. `Qwen official Unsloth docs`
2. `Qwen official Axolotl docs`
3. `QwenLM/Qwen3-Embedding`
4. `PA-RAG`
5. `modelscope/easydistill`

### C. Thứ không nên làm

- không SFT full vào toàn bộ raw Opus-distill mà không lọc
- không bỏ hẳn non-reasoning/chat data
- không kỳ vọng `4B` match `Opus` cross-domain
- không nhồi factual knowledge vào weights nếu use case chủ yếu là private docs/codebase

## Recommended Data Mix

Đây là mix practical cho round đầu, suy ra từ các nguồn trên:

- `45%` Opus/Sonnet distill:
  - `Farseen0`
  - `Roman1111111`
  - `nohurry`
  - `TeichAI 887x/1100x`
- `30%` open reasoning/code/math:
  - `OpenThoughts3`
  - `OpenR1-Math-Raw`
  - `ODA-Math-460k`
- `25%` non-reasoning / chat quality:
  - `FineTome-single-turn`

Nếu mục tiêu thiên `agent coding`, thay `10%` math bằng `OpenThoughts-Agent-v1-SFT`.

## Practical Recommendations For 3x RTX 3090

### Round 1: SFT an toàn

- Base: `Qwen3.5-4B`
- Cách train: `LoRA/QLoRA`, `response-only`
- Context train thực tế: `8k` hoặc `16k`, đừng lao vào `128k+` ngay
- Epoch: `1` hoặc thấp
- Mục tiêu: sửa reasoning style, không phá base model

### Round 2: domain-topup

- Thêm data riêng của anh:
  - code review tốt
  - bug fix traces tốt
  - docs QA nội bộ
  - prompt -> answer có kiểm chứng
- Ưu tiên quality hơn số lượng

### Round 3: RAG

- Embed bằng `Qwen3-Embedding-4B`
- Rerank bằng `Qwen3-Reranker-4B`
- Chunk by semantics, không chỉ fixed-token
- Bật citation requirement trong prompt template

### Round 4: Eval + iterate

- Eval theo 4 nhóm:
  - reasoning
  - coding
  - retrieval-grounded QA
  - instruction following
- Chấm cả `accuracy`, `verbosity`, `citation faithfulness`, `tool-use correctness`

## Key Risks

- **Legal/data provenance**
  - Nhiều dataset distill từ model proprietary. Nên review legal trước khi dùng commercial.
- **Over-distillation**
  - Model dễ "nói giống Opus" nhưng không thật sự nghĩ tốt hơn.
- **Catastrophic forgetting**
  - Nếu thiếu non-reasoning mix, chat quality dễ tụt.
- **RAG neglect**
  - Bỏ qua embedding/reranker sẽ phí công, nhất là với model 4B.

## Recommended Next Steps

1. Tải 6 dataset lõi ở mục shortlist.
2. Chuẩn hóa hết về một schema `messages + answer`.
3. Dedupe theo prompt/answer gần giống.
4. Chọn `10k–30k` mẫu round đầu, không train full raw dump.
5. Chạy `QLoRA response-only` round 1.
6. Setup `Qwen3-Embedding-4B + Qwen3-Reranker-4B` cho lane RAG song song.
7. Xây internal eval set tối thiểu `200–500` bài.

## Sources

- https://huggingface.co/TeichAI/Qwen3.5-4B-Claude-Opus-Reasoning-Distill
- https://huggingface.co/datasets/Farseen0/opus-4.6-reasoning-sft-12k
- https://huggingface.co/datasets/Roman1111111/claude-opus-4.6-10000x
- https://huggingface.co/datasets/nohurry/Opus-4.6-Reasoning-3000x-filtered
- https://huggingface.co/datasets/TeichAI/Claude-Opus-4.6-Reasoning-887x
- https://huggingface.co/datasets/TeichAI/Claude-Sonnet-4.6-Reasoning-1100x
- https://huggingface.co/datasets/TeichAI/claude-4.5-opus-high-reasoning-250x
- https://huggingface.co/datasets/open-r1/OpenR1-Math-Raw
- https://huggingface.co/datasets/open-thoughts/OpenThoughts3-1.2M
- https://huggingface.co/datasets/open-thoughts/OpenThoughts-Agent-v1-SFT
- https://huggingface.co/datasets/OpenDataArena/ODA-Math-460k
- https://huggingface.co/OpenDataArena/Qwen3-8B-ODA-Math-460k
- https://huggingface.co/datasets/mrm8488/FineTome-single-turn
- https://qwen.readthedocs.io/en/latest/training/unsloth.html
- https://qwen.readthedocs.io/en/latest/training/axolotl.html
- https://github.com/modelscope/easydistill
- https://github.com/QwenLM/Qwen3-Embedding
- https://huggingface.co/Qwen/Qwen3-Embedding-4B
- https://github.com/wujwyi/PA-RAG
- https://github.com/PotatoHD404/QwenRag

## Unresolved Questions

1. Anh muốn target lane nào hơn: `coding agent`, `doc QA`, hay `general reasoning`.
2. Có giới hạn license/commercial nào không.
3. Anh muốn `Vietnamese-first` hay `multilingual`.
