# Research Report: Max-IQ roadmap cho Qwen3.5-4B khi chạy dưới Claude Code

**Ngày nghiên cứu:** 2026-04-15  
**Scope:** thiết kế roadmap mạnh nhất có thể cho `Qwen3.5-4B` khi `tool-use` lớp runtime đã có sẵn qua `Claude Code` hoặc orchestration tương đương.

## Executive Summary

Nếu `Qwen3.5-4B` chạy như "bộ não" bên trong một runtime đã có tool belt, terminal, file access, search, test runner, thì mục tiêu training phải đổi.

Không cần ưu tiên dạy model phát ra tool-call syntax như một agent độc lập nữa. Cái đáng tối ưu hơn là:

1. hiểu khi nào cần tool
2. biết lập kế hoạch nhiều bước
3. biết đọc kết quả tool và sửa hướng
4. biết grounded reasoning trên docs/code/output thật
5. biết tự kiểm tra, critique, revise

Nói ngắn: khi gắn vào `Claude Code`, "IQ tăng mạnh nhất" đến từ `reasoning quality + retrieval quality + verifier loops + evaluation`, không phải từ việc train thêm để model biết gọi một JSON tool schema.

## Core Reframe

### Tool-use đã có thì còn phải train gì

Nếu runtime đã:

- inject tool descriptions
- điều phối lời gọi tool
- trả kết quả tool về context
- quản lý permission/sandbox/session

thì model cần giỏi ở 4 việc sau:

- **Tool selection judgment**
  - biết lúc nào nên đọc file, chạy test, search, hay không cần tool
- **Observation digestion**
  - biết đọc log, traceback, diff, spec, benchmark output
- **Planning and repair**
  - biết chia bài toán, cập nhật giả thuyết, đổi chiến lược
- **Grounded answer synthesis**
  - biết trả lời bám dữ liệu vừa quan sát, không hallucinate

### Hệ quả

Do đó, training data ưu tiên không còn là:

- pure chat
- pure tool schema dumps
- pure style distillation

Mà là:

- bug report -> inspect -> patch -> verify
- failing test -> investigate -> fix -> rerun
- spec/doc/code context -> answer with citations
- plan -> execute -> critique -> revise
- retrieval result -> grounded synthesis

## What actually moves the ceiling

### 1. Continual pretraining vẫn là nền mạnh nhất

Muốn model `4B` "nghĩ tốt hơn" thật, continual pretraining trên corpus sạch vẫn có leverage lớn nhất.

Ưu tiên corpus:

- code chất lượng cao
- PR discussion tốt
- issue -> fix narratives
- design docs
- API docs
- textbook CS/math
- technical troubleshooting traces

Không cần vội train theo context cực dài. Quan trọng là chất lượng signal.

### 2. SFT phải chuyển từ "reasoning monologue" sang "work trace"

Khi đã có runtime tool-use, SFT tốt nên giống:

- observe context
- propose plan
- inspect artifacts
- reason from evidence
- produce patch or grounded answer

Tức là train trên `workflow traces`, không chỉ train trên `final polished answer`.

### 3. Preference tuning nên tối ưu judgement, không chỉ style

Nên xây preference pairs theo các trục:

- grounded hơn
- ít hallucination hơn
- ngắn hơn nhưng đủ
- quyết định dùng tool đúng hơn
- biết nói "chưa đủ bằng chứng" đúng lúc
- biết revise khi evidence mới mâu thuẫn

### 4. RL nên là verifier-based

Với model nhỏ, RL không verifier là dễ overfit style và tự lừa.

Verifier phù hợp:

- code: test pass
- bugfix: reproduce -> fixed
- retrieval QA: answer cite đúng chunk
- math: exact answer / checker
- planning: completion score trên task harness

### 5. RAG là hệ số nhân lớn nhất cho factual/domain IQ

Nếu task thực là docs/code/spec/private corpus, thì `RAG + reranker + citation` gần như bắt buộc.

Model `4B` sau tune vẫn không giữ factual breadth ngang model lớn. Nhưng `4B + good retrieval` có thể trông "khôn" hơn hẳn trên việc thực.

### 6. Inference-time loops cực đáng giá

Không quan tâm thời gian thì phải tận dụng:

- draft -> critique -> revise
- plan -> execute -> verify
- multi-sample -> rerank
- retrieve -> answer -> check citation -> rewrite

Đây là cách rẻ nhất để vắt thêm chất lượng từ model nhỏ sau khi training xong.

## Recommended training target distribution

Khi tool runtime đã có sẵn, tôi sẽ phân bố objective như sau:

- `25%` continual pretraining trên code/docs/troubleshooting corpora
- `25%` high-quality reasoning distill
- `20%` workflow traces kiểu coding/debugging/research
- `15%` grounded QA / citation / RAG-answering
- `10%` preference data cho truthfulness + judgement
- `5%` pure tool-call or format-following samples

Lý do:

- tool syntax không còn là bottleneck chính
- judgement mới là bottleneck

## Data types worth collecting

### A. Coding/repair traces

- issue description
- relevant files
- failed test output
- patch
- final verification

### B. Research traces

- user ask
- search snippets / docs excerpts
- comparison
- recommendation with uncertainty

### C. Grounded QA traces

- retrieved chunks
- answer
- citation spans
- abstain/refusal when evidence thiếu

### D. Critique/revision traces

- initial answer
- critique
- revised answer
- final reason why revised is better

### E. Compression traces

- long logs/docs
- concise but faithful summary

Loại data này sát runtime `Claude Code` hơn nhiều so với "teacher answer only".

## Architecture recommendation

### Runtime split

- `Qwen3.5-4B tuned`
  - reasoning core
- `Claude Code runtime`
  - tool execution / file ops / shell / search / orchestration
- `RAG subsystem`
  - retrieval + rerank + citation pack
- `Verifier subsystem`
  - test runner / static checks / correctness checks
- `Reflection layer`
  - critique + revise loop

### Mental model

Không xem model là agent full-stack tự trị.

Xem nó là:

- planner
- analyst
- patch proposer
- grounded synthesizer

còn execution và external memory do runtime đảm nhiệm.

## Max-IQ roadmap

### Stage 0: build eval first

Không có eval thì mọi "khôn hơn" chỉ là cảm giác.

Nên có 5 buckets:

- coding fix
- code understanding
- grounded docs QA
- research/synthesis
- plan/repair multi-step

### Stage 1: continual pretraining

Mục tiêu:

- tăng nền technical prior
- tăng khả năng đọc spec/code/log

Nguồn:

- code sạch
- docs API
- troubleshooting knowledge
- high-signal longform technical text

### Stage 2: SFT on reasoning + workflow traces

Mix:

- Opus/Sonnet distill
- open reasoning
- coding/debugging traces
- grounded QA
- concise chat quality

### Stage 3: preference tuning

Tối ưu:

- groundedness
- less hallucination
- better tool judgement
- concise final output

### Stage 4: verifier RL

Chỉ dùng task nào chấm được:

- tests
- exact match
- citation faithfulness
- task completion harness

### Stage 5: RAG stack

- retrieval
- reranking
- chunk stitching
- evidence packing
- citation-aware prompting

### Stage 6: reflection loops

- self-critique
- revise
- optional multi-sample rerank

## Priority order if time is unlimited

Nếu phải xếp theo impact thực tế khi tool-use đã có sẵn:

1. `Eval`
2. `RAG + reranker + grounding`
3. `Continual pretraining`
4. `Workflow-trace SFT`
5. `Preference tuning`
6. `Verifier RL`
7. `Inference-time critique/revise`
8. `Tool syntax tuning`

Tool syntax tuning gần cuối vì runtime đã lo phần này.

## What to de-prioritize

- train quá nhiều mẫu chỉ để model phát đúng schema tool call
- long-CoT chỉ để nhìn "thông minh"
- benchmark vanity không sát task thật
- context length chasing từ đầu
- publish checkpoint sớm trước khi eval grounded QA/coding

## Strong recommendations

1. Tách rõ hai thứ:
   - model IQ
   - system IQ
2. Với `4B`, system IQ sẽ đóng góp cực lớn.
3. Nếu runtime là `Claude Code`, hãy train model cho:
   - planning
   - evidence reading
   - grounded synthesis
   - repair loops
4. Dùng `RAG` như external memory lâu dài.
5. Dùng verifier như external judge lâu dài.

## Concrete next actions

1. Xây eval set agent-style.
2. Thu workflow traces thay vì chỉ QA pairs.
3. Chạy continual pretraining nhỏ trước SFT lớn.
4. Tune model cho grounded coding/docs tasks.
5. Tích hợp RAG + reranker.
6. Bật critique/revise loop trong serving.

## Unresolved Questions

1. Runtime `Claude Code` của anh có inject search/web tool ổn định không.
2. Anh muốn strongest lane ở `coding`, `research`, hay `docs QA`.
3. Có chấp nhận hệ thống nhiều tầng chậm nhưng mạnh hơn không.
