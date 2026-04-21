use futures::StreamExt;
use reqwest::StatusCode;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;

use crate::account::pool::CurrentAccount;
use crate::providers::types::{ChatMessage, ChatStreamEvent, ProviderError};

const DEFAULT_CODEX_UPSTREAM_BASE_URL: &str = "https://chatgpt.com/backend-api/codex/responses";
const DEFAULT_CODEX_ORIGINATOR: &str = "codex-cli";
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(180);
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const EMPTY_INPUT_PLACEHOLDER: &str = "...";
const DEFAULT_CODEX_INSTRUCTIONS: &str = r##"You are Codex, based on GPT-5. You are running as a coding agent in the Codex CLI on a user's computer.

## General

- When searching for text or files, prefer using `rg` or `rg --files` respectively because `rg` is much faster than alternatives like `grep`. (If the `rg` command is not found, then use alternatives.)

## Editing constraints

- Default to ASCII when editing or creating files. Only introduce non-ASCII or other Unicode characters when there is a clear justification and the file already uses them.
- Add succinct code comments that explain what is going on if code is not self-explanatory. You should not add comments like "Assigns the value to the variable", but a brief comment might be useful ahead of a complex code block that the user would otherwise have to spend time parsing out. Usage of these comments should be rare.
- Try to use apply_patch for single file edits, but it is fine to explore other options to make the edit if it does not work well. Do not use apply_patch for changes that are auto-generated (i.e. generating package.json or running a lint or format command like gofmt) or when scripting is more efficient (such as search and replacing a string across a codebase).
- You may be in a dirty git worktree.
    * NEVER revert existing changes you did not make unless explicitly requested, since these changes were made by the user.
    * If asked to make a commit or code edits and there are unrelated changes to your work or changes that you didn't make in those files, don't revert those changes.
    * If the changes are in files you've touched recently, you should read carefully and understand how you can work with the changes rather than reverting them.
    * If the changes are in unrelated files, just ignore them and don't revert them.
- Do not amend a commit unless explicitly requested to do so.
- While you are working, you might notice unexpected changes that you didn't make. If this happens, STOP IMMEDIATELY and ask the user how they would like to proceed.
- **NEVER** use destructive commands like `git reset --hard` or `git checkout --` unless specifically requested or approved by the user.

## Plan tool

When using the planning tool:
- Skip using the planning tool for straightforward tasks (roughly the easiest 25%).
- Do not make single-step plans.
- When you made a plan, update it after having performed one of the sub-tasks that you shared on the plan.

## Codex CLI harness, sandboxing, and approvals

The Codex CLI harness supports several different configurations for sandboxing and escalation approvals that the user can choose from.

Filesystem sandboxing defines which files can be read or written. The options for `sandbox_mode` are:
- **read-only**: The sandbox only permits reading files.
- **workspace-write**: The sandbox permits reading files, and editing files in `cwd` and `writable_roots`. Editing files in other directories requires approval.
- **danger-full-access**: No filesystem sandboxing - all commands are permitted.

Network sandboxing defines whether network can be accessed without approval. Options for `network_access` are:
- **restricted**: Requires approval
- **enabled**: No approval needed

Approvals are your mechanism to get user consent to run shell commands without the sandbox. Possible configuration options for `approval_policy` are
- **untrusted**: The harness will escalate most commands for user approval, apart from a limited allowlist of safe "read" commands.
- **on-failure**: The harness will allow all commands to run in the sandbox (if enabled), and failures will be escalated to the user for approval to run again without the sandbox.
- **on-request**: Commands will be run in the sandbox by default, and you can specify in your tool call if you want to escalate a command to run without sandboxing. (Note that this mode is not always available. If it is, you'll see parameters for it in the `shell` command description.)
- **never**: This is a non-interactive mode where you may NEVER ask the user for approval to run commands. Instead, you must always persist and work around constraints to solve the task for the user. You MUST do your utmost best to finish the task and validate your work before yielding. If this mode is paired with `danger-full-access`, take advantage of it to deliver the best outcome for the user. Further, in this mode, your default testing philosophy is overridden: Even if you don't see local patterns for testing, you may add tests and scripts to validate your work. Just remove them before yielding.

When you are running with `approval_policy == on-request`, and sandboxing enabled, here are scenarios where you'll need to request approval:
- You need to run a command that writes to a directory that requires it (e.g. running tests that write to /var)
- You need to run a GUI app (e.g., open/xdg-open/osascript) to open browsers or files.
- You are running sandboxed and need to run a command that requires network access (e.g. installing packages)
- If you run a command that is important to solving the user's query, but it fails because of sandboxing, rerun the command with approval. ALWAYS proceed to use the `sandbox_permissions` and `justification` parameters - do not message the user before requesting approval for the command.
- You are about to take a potentially destructive action such as an `rm` or `git reset` that the user did not explicitly ask for
- (for all of these, you should weigh alternative paths that do not require approval)

When `sandbox_mode` is set to read-only, you'll need to request approval for any command that isn't a read.

You will be told what filesystem sandboxing, network sandboxing, and approval mode are active in a developer or user message. If you are not told about this, assume that you are running with workspace-write, network sandboxing enabled, and approval on-failure.

Although they introduce friction to the user because your work is paused until the user responds, you should leverage them when necessary to accomplish important work. If the completing the task requires escalated permissions, Do not let these settings or the sandbox deter you from attempting to accomplish the user's task unless it is set to "never", in which case never ask for approvals.

When requesting approval to execute a command that will require escalated privileges:
  - Provide the `sandbox_permissions` parameter with the value `"require_escalated"`
  - Include a short, 1 sentence explanation for why you need escalated permissions in the justification parameter

## Special user requests

- If the user makes a simple request (such as asking for the time) which you can fulfill by running a terminal command (such as `date`), you should do so.
- If the user asks for a "review", default to a code review mindset: prioritise identifying bugs, risks, behavioural regressions, and missing tests. Findings must be the primary focus of the response - keep summaries or overviews brief and only after enumerating the issues. Present findings first (ordered by severity with file/line references), follow with open questions or assumptions, and offer a change-summary only as a secondary detail. If no findings are discovered, state that explicitly and mention any residual risks or testing gaps.

## Frontend tasks
When doing frontend design tasks, avoid collapsing into "AI slop" or safe, average-looking layouts.
Aim for interfaces that feel intentional, bold, and a bit surprising.
- Typography: Use expressive, purposeful fonts and avoid default stacks (Inter, Roboto, Arial, system).
- Color & Look: Choose a clear visual direction; define CSS variables; avoid purple-on-white defaults. No purple bias or dark mode bias.
- Motion: Use a few meaningful animations (page-load, staggered reveals) instead of generic micro-motions.
- Background: Don't rely on flat, single-color backgrounds; use gradients, shapes, or subtle patterns to build atmosphere.
- Overall: Avoid boilerplate layouts and interchangeable UI patterns. Vary themes, type families, and visual languages across outputs.
- Ensure the page loads properly on both desktop and mobile

Exception: If working within an existing website or design system, preserve the established patterns, structure, and visual language.

## Presenting your work and final message

You are producing plain text that will later be styled by the CLI. Follow these rules exactly. Formatting should make results easy to scan, but not feel mechanical. Use judgment to decide how much structure adds value.

- Default: be very concise; friendly coding teammate tone.
- Ask only when needed; suggest ideas; mirror the user's style.
- For substantial work, summarize clearly; follow final‑answer formatting.
- Skip heavy formatting for simple confirmations.
- Don't dump large files you've written; reference paths only.
- No "save/copy this file" - User is on the same machine.
- Offer logical next steps (tests, commits, build) briefly; add verify steps if you couldn't do something.
- For code changes:
  * Lead with a quick explanation of the change, and then give more details on the context covering where and why a change was made. Do not start this explanation with "summary", just jump right in.
  * If there are natural next steps the user may want to take, suggest them at the end of your response. Do not make suggestions if there are no natural next steps.
  * When suggesting multiple options, use numeric lists for the suggestions so the user can quickly respond with a single number.
- The user does not command execution outputs. When asked to show the output of a command (e.g. `git show`), relay the important details in your answer or summarize the key lines so the user understands the result.

### Final answer structure and style guidelines

- Plain text; CLI handles styling. Use structure only when it helps scanability.
- Headers: optional; short Title Case (1-3 words) wrapped in **…**; no blank line before the first bullet; add only if they truly help.
- Bullets: use - ; merge related points; keep to one line when possible; 4–6 per list ordered by importance; keep phrasing consistent.
- Monospace: backticks for commands/paths/env vars/code ids and inline examples; use for literal keyword bullets; never combine with **.
- Code samples or multi-line snippets should be wrapped in fenced code blocks; include an info string as often as possible.
- Structure: group related bullets; order sections general → specific → supporting; for subsections, start with a bolded keyword bullet, then items; match complexity to the task.
- Tone: collaborative, concise, factual; present tense, active voice; self‑contained; no "above/below"; parallel wording.
- Don'ts: no nested bullets/hierarchies; no ANSI codes; don't cram unrelated keywords; keep keyword lists short—wrap/reformat if long; avoid naming formatting styles in answers.
- Adaptation: code explanations → precise, structured with code refs; simple tasks → lead with outcome; big changes → logical walkthrough + rationale + next actions; casual one-offs → plain sentences, no headers/bullets.
- File References: When referencing files in your response follow the below rules:
  * Use inline code to make file paths clickable.
  * Each reference should have a stand alone path. Even if it's the same file.
  * Accepted: absolute, workspace-relative, a/ or b/ diff prefixes, or bare filename/suffix.
  * Optionally include line/column (1-based): :line[:column] or #Lline[Ccolumn] (column defaults to 1).
  * Do not use URIs like file://, vscode://, or https://.
  * Do not provide range of lines
  * Examples: src/app.ts, src/app.ts:42, b/server/index.js#L10, C:\repo\project\main.rs:12:5"##;

#[derive(Clone, Debug)]
pub struct CodexClient {
    upstream_base_url: String,
    originator: String,
    user_agent: String,
}

impl Default for CodexClient {
    fn default() -> Self {
        Self::new(
            DEFAULT_CODEX_UPSTREAM_BASE_URL.to_string(),
            DEFAULT_CODEX_ORIGINATOR.to_string(),
            default_codex_user_agent(),
        )
    }
}

impl CodexClient {
    pub fn new(upstream_base_url: String, originator: String, user_agent: String) -> Self {
        Self {
            upstream_base_url: trim_trailing_slash(&upstream_base_url),
            originator: originator.trim().to_string(),
            user_agent: user_agent.trim().to_string(),
        }
    }

    pub async fn send_request_stream(
        &self,
        account: &CurrentAccount,
        model: &str,
        messages: &[ChatMessage],
        system_prompt: &str,
    ) -> Result<mpsc::UnboundedReceiver<ChatStreamEvent>, ProviderError> {
        let access_token = account
            .codex_tokens()
            .map_err(ProviderError::Network)?
            .access_token
            .trim()
            .to_string();
        if access_token.is_empty() {
            return Err(ProviderError::Unauthorized);
        }

        let client = build_http_client(account.proxy_url.as_ref())?;
        let request_body = build_codex_request_body(model, messages, system_prompt);
        let session_id = resolve_session_id(account, messages);

        let response = client
            .post(&self.upstream_base_url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {access_token}"),
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .header("originator", self.originator.as_str())
            .header(reqwest::header::USER_AGENT, self.user_agent.as_str())
            .header("session_id", session_id)
            .json(&request_body)
            .send()
            .await
            .map_err(|error| classify_transport_error(error, account.proxy_url.as_ref()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(classify_codex_status(status, &body));
        }

        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut parser = SseParser::default();
            let mut stream = response.bytes_stream();
            let mut saw_output = false;
            let mut saw_sse_activity = false;
            let mut terminal_event_seen = false;

            while let Some(chunk) = stream.next().await {
                let bytes = match chunk {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        let _ = tx.send(ChatStreamEvent::Error(ProviderError::UpstreamTransient(
                            format!("Codex upstream stream failed: {error}"),
                        )));
                        return;
                    }
                };

                let events = match parser.push(&bytes) {
                    Ok(events) => events,
                    Err(error) => {
                        let _ = tx.send(ChatStreamEvent::Error(ProviderError::UpstreamTransient(
                            error,
                        )));
                        return;
                    }
                };

                if !events.is_empty() {
                    saw_sse_activity = true;
                }

                if forward_sse_events(
                    &tx,
                    &events,
                    &mut saw_output,
                    &mut saw_sse_activity,
                    &mut terminal_event_seen,
                ) {
                    return;
                }
            }

            if let Some(event) = match parser.finish() {
                Ok(event) => event,
                Err(error) => {
                    let _ = tx.send(ChatStreamEvent::Error(ProviderError::UpstreamTransient(
                        error,
                    )));
                    return;
                }
            } {
                saw_sse_activity = true;
                if forward_sse_events(
                    &tx,
                    &[event],
                    &mut saw_output,
                    &mut saw_sse_activity,
                    &mut terminal_event_seen,
                ) {
                    return;
                }
            }

            if let Some(final_event) =
                synthesize_terminal_event(saw_output, saw_sse_activity, terminal_event_seen)
            {
                let _ = tx.send(final_event);
            }
        });

        Ok(rx)
    }
}

fn synthesize_terminal_event(
    saw_output: bool,
    saw_sse_activity: bool,
    terminal_event_seen: bool,
) -> Option<ChatStreamEvent> {
    if terminal_event_seen {
        return None;
    }

    if saw_output {
        // Let the consumer surface treat a post-output truncation as an unexpected end
        // instead of synthesizing a false success.
        return None;
    }

    if saw_sse_activity {
        return Some(ChatStreamEvent::Error(ProviderError::UpstreamTransient(
            "Codex upstream stream ended without a usable terminal event.".to_string(),
        )));
    }

    Some(ChatStreamEvent::Error(ProviderError::UpstreamTransient(
        "No response received from upstream in time. Check the proxy or account and try again."
            .to_string(),
    )))
}

fn build_http_client(proxy_url: Option<&String>) -> Result<reqwest::Client, ProviderError> {
    let mut builder = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT);

    if let Some(url) = proxy_url {
        let proxy = reqwest::Proxy::all(url)
            .map_err(|error| ProviderError::ProxyFailed(format!("Invalid proxy: {error}")))?;
        builder = builder.proxy(proxy);
    }

    builder.build().map_err(|error| {
        if proxy_url.is_some() {
            ProviderError::ProxyFailed(format!("Build Codex HTTP client failed: {error}"))
        } else {
            ProviderError::Network(format!("Build Codex HTTP client failed: {error}"))
        }
    })
}

fn build_codex_request_body(model: &str, messages: &[ChatMessage], system_prompt: &str) -> Value {
    let (resolved_model, reasoning_effort) = resolve_codex_model(model);
    let input = build_codex_input(messages);
    let mut body = json!({
        "model": resolved_model,
        "input": input,
        "stream": true,
        "store": false,
        "instructions": resolve_codex_instructions(system_prompt),
    });

    if let Some(effort) = reasoning_effort {
        body["reasoning"] = json!({ "effort": effort, "summary": "auto" });
        if effort != "none" {
            body["include"] = json!(["reasoning.encrypted_content"]);
        }
    }

    body
}

fn resolve_codex_instructions(system_prompt: &str) -> String {
    let trimmed = system_prompt.trim();
    if trimmed.is_empty() {
        DEFAULT_CODEX_INSTRUCTIONS.to_string()
    } else {
        trimmed.to_string()
    }
}

fn build_codex_input(messages: &[ChatMessage]) -> Value {
    let mut items = Vec::new();

    for message in messages {
        let text = message.content.trim();
        if text.is_empty() {
            continue;
        }

        let content_type = if message.role == "assistant" {
            "output_text"
        } else {
            "input_text"
        };

        items.push(json!({
            "type": "message",
            "role": normalize_role(&message.role),
            "content": [{ "type": content_type, "text": text }],
        }));
    }

    if items.is_empty() {
        items.push(json!({
            "type": "message",
            "role": "user",
            "content": [{ "type": "input_text", "text": EMPTY_INPUT_PLACEHOLDER }],
        }));
    }

    Value::Array(items)
}

fn resolve_codex_model(model: &str) -> (String, Option<&'static str>) {
    const EFFORT_LEVELS: [&str; 5] = ["none", "low", "medium", "high", "xhigh"];

    for effort in EFFORT_LEVELS {
        let suffix = format!("-{effort}");
        if model.ends_with(&suffix) {
            return (model.trim_end_matches(&suffix).to_string(), Some(effort));
        }
    }

    if model.contains("codex") {
        (model.to_string(), Some("low"))
    } else {
        (model.to_string(), None)
    }
}

fn normalize_role(role: &str) -> &str {
    match role {
        "assistant" => "assistant",
        _ => "user",
    }
}

fn resolve_session_id(account: &CurrentAccount, messages: &[ChatMessage]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account.provider_slug.as_bytes());
    hasher.update(account.name.as_bytes());
    if let Some(id) = account.id {
        hasher.update(id.to_string().as_bytes());
    }
    if let Some(seed) = messages
        .iter()
        .find(|message| message.role == "assistant")
        .or_else(|| messages.last())
        .map(|message| message.content.as_bytes())
    {
        hasher.update(seed);
    }

    let digest = hasher.finalize();
    let suffix = digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sess_{suffix}")
}

fn classify_transport_error(error: reqwest::Error, proxy_url: Option<&String>) -> ProviderError {
    if error.is_timeout() {
        return ProviderError::UpstreamTransient(
            "Codex upstream request timed out before a response arrived.".to_string(),
        );
    }

    if proxy_url.is_some() {
        ProviderError::ProxyFailed(format!("Proxy request failed: {error}"))
    } else {
        ProviderError::Network(format!("Codex upstream request failed: {error}"))
    }
}

fn classify_codex_status(status: StatusCode, body: &str) -> ProviderError {
    let body_preview = body.trim().chars().take(300).collect::<String>();
    let lower = body.to_ascii_lowercase();

    match status.as_u16() {
        401 => ProviderError::Unauthorized,
        403 if lower.contains("cloudflare") || lower.contains("cf-ray") => ProviderError::CfBlocked,
        403 if lower.contains("unauthorized") || lower.contains("forbidden") => {
            ProviderError::Unauthorized
        }
        429 => ProviderError::RateLimited,
        400 if lower.contains("failed to look up session id")
            || lower.contains("invalid-credentials")
            || lower.contains("unauthenticated")
            || lower.contains("expired")
            || lower.contains("invalid token") =>
        {
            ProviderError::Unauthorized
        }
        408 | 409 | 423 | 425 | 500..=599 => ProviderError::UpstreamTransient(format!(
            "Codex upstream returned {}: {}",
            status, body_preview
        )),
        _ => ProviderError::Network(format!(
            "Codex upstream returned {}: {}",
            status, body_preview
        )),
    }
}

fn trim_trailing_slash(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

fn default_codex_user_agent() -> String {
    format!(
        "codex-cli/1.0.18 ({}; {})",
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

#[derive(Default)]
struct SseParser {
    pending: Vec<u8>,
    current_event: Option<String>,
    current_data: Vec<String>,
}

#[derive(Clone)]
struct ParsedSseEvent {
    event: Option<String>,
    data: String,
}

impl SseParser {
    fn push(&mut self, chunk: &[u8]) -> Result<Vec<ParsedSseEvent>, String> {
        self.pending.extend_from_slice(chunk);
        let mut events = Vec::new();

        while let Some(pos) = self.pending.iter().position(|byte| *byte == b'\n') {
            let line = self.pending.drain(..=pos).collect::<Vec<_>>();
            let decoded = decode_sse_line(&line)?;
            if let Some(event) = self.process_line(decoded) {
                events.push(event);
            }
        }

        Ok(events)
    }

    fn finish(&mut self) -> Result<Option<ParsedSseEvent>, String> {
        if !self.pending.is_empty() {
            let decoded = decode_sse_line(&self.pending.clone())?;
            self.pending.clear();
            if let Some(event) = self.process_line(decoded) {
                return Ok(Some(event));
            }
        }

        Ok(self.flush_event())
    }

    fn process_line(&mut self, line: String) -> Option<ParsedSseEvent> {
        if line.is_empty() {
            return self.flush_event();
        }
        if line.starts_with(':') {
            return None;
        }
        if let Some(value) = line.strip_prefix("event:") {
            self.current_event = Some(value.trim().to_string());
            return None;
        }
        if let Some(value) = line.strip_prefix("data:") {
            self.current_data.push(value.trim_start().to_string());
        }
        None
    }

    fn flush_event(&mut self) -> Option<ParsedSseEvent> {
        if self.current_event.is_none() && self.current_data.is_empty() {
            return None;
        }

        Some(ParsedSseEvent {
            event: self.current_event.take(),
            data: self.current_data.drain(..).collect::<Vec<_>>().join("\n"),
        })
    }
}

fn decode_sse_line(bytes: &[u8]) -> Result<String, String> {
    std::str::from_utf8(bytes)
        .map_err(|error| format!("Invalid UTF-8 in Codex stream: {error}"))
        .map(|line| line.trim_end_matches(['\r', '\n']).to_string())
}

fn forward_sse_events(
    tx: &mpsc::UnboundedSender<ChatStreamEvent>,
    events: &[ParsedSseEvent],
    saw_output: &mut bool,
    saw_sse_activity: &mut bool,
    terminal_event_seen: &mut bool,
) -> bool {
    for event in events {
        let mapped = match map_sse_event(event, saw_output, saw_sse_activity, terminal_event_seen) {
            Ok(mapped) => mapped,
            Err(error) => {
                let _ = tx.send(ChatStreamEvent::Error(error));
                return true;
            }
        };
        let Some(mapped) = mapped else {
            continue;
        };

        let stop = matches!(mapped, ChatStreamEvent::Done | ChatStreamEvent::Error(_));
        if tx.send(mapped).is_err() {
            return true;
        }
        if stop {
            return true;
        }
    }

    false
}

fn map_sse_event(
    event: &ParsedSseEvent,
    saw_output: &mut bool,
    saw_sse_activity: &mut bool,
    terminal_event_seen: &mut bool,
) -> Result<Option<ChatStreamEvent>, ProviderError> {
    if event.data.trim().is_empty() {
        return Ok(None);
    }
    if event.data.trim() == "[DONE]" {
        *terminal_event_seen = true;
        return Ok(Some(ChatStreamEvent::Done));
    }

    let payload: Value = serde_json::from_str(&event.data).map_err(|error| {
        ProviderError::UpstreamTransient(format!(
            "Invalid Codex SSE payload for event {:?}: {error}",
            event.event
        ))
    })?;
    let event_type = payload
        .get("type")
        .and_then(Value::as_str)
        .or(event.event.as_deref())
        .unwrap_or_default();
    *saw_sse_activity = true;

    match event_type {
        "response.output_text.delta" => {
            let text = payload
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if text.is_empty() {
                return Ok(None);
            }
            *saw_output = true;
            Ok(Some(ChatStreamEvent::Token(text)))
        }
        "response.output_text.done" => {
            if *saw_output {
                return Ok(None);
            }
            let text = payload
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if text.is_empty() {
                return Ok(None);
            }
            *saw_output = true;
            Ok(Some(ChatStreamEvent::Token(text)))
        }
        "response.reasoning_summary_text.delta" => {
            let text = payload
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if text.is_empty() {
                return Ok(None);
            }
            Ok(Some(ChatStreamEvent::Thinking(text)))
        }
        "response.reasoning_summary_text.done" => {
            let text = payload
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if text.is_empty() {
                return Ok(None);
            }
            Ok(Some(ChatStreamEvent::Thinking(text)))
        }
        "response.completed" | "response.done" => {
            if !*saw_output {
                if let Some(text) = extract_completed_output_text(&payload) {
                    *saw_output = true;
                    return Ok(Some(ChatStreamEvent::Token(text)));
                }
            }
            *terminal_event_seen = true;
            Ok(Some(ChatStreamEvent::Done))
        }
        "response.failed" | "error" => {
            *terminal_event_seen = true;
            Ok(Some(ChatStreamEvent::Error(classify_codex_event_error(
                &payload,
            ))))
        }
        "response.in_progress"
        | "response.created"
        | "response.content_part.added"
        | "response.content_part.done"
        | "response.output_item.added"
        | "response.output_item.done"
        | "response.reasoning_summary_part.added"
        | "response.reasoning_summary_part.done" => Ok(None),
        _ if !event_type.is_empty() => {
            tracing::debug!(event_type, "Ignoring unsupported Codex SSE event");
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn extract_completed_output_text(payload: &Value) -> Option<String> {
    let outputs = payload
        .get("response")
        .and_then(|response| response.get("output"))
        .and_then(Value::as_array)?;

    let text = outputs
        .iter()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("message"))
        .filter_map(|item| item.get("content").and_then(Value::as_array))
        .flatten()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("output_text"))
        .filter_map(|item| item.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("");

    if text.is_empty() { None } else { Some(text) }
}

fn extract_error_message(payload: &Value) -> String {
    payload
        .get("error")
        .and_then(|error| error.get("message").or_else(|| Some(error)))
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .get("response")
                .and_then(|response| response.get("error"))
                .and_then(|error| error.get("message").or_else(|| Some(error)))
                .and_then(Value::as_str)
        })
        .or_else(|| payload.get("message").and_then(Value::as_str))
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "Codex upstream request failed.".to_string())
}

fn classify_codex_event_error(payload: &Value) -> ProviderError {
    let message = extract_error_message(payload);
    let lower = message.to_ascii_lowercase();

    if lower.contains("failed to look up session id")
        || lower.contains("invalid-credentials")
        || lower.contains("unauthenticated")
        || lower.contains("expired")
        || lower.contains("invalid token")
    {
        ProviderError::Unauthorized
    } else if lower.contains("cloudflare") || lower.contains("cf-ray") {
        ProviderError::CfBlocked
    } else if lower.contains("rate limit") || lower.contains("too many requests") {
        ProviderError::RateLimited
    } else {
        ProviderError::UpstreamTransient(message)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_CODEX_INSTRUCTIONS, ParsedSseEvent, build_codex_request_body,
        classify_codex_status, map_sse_event, synthesize_terminal_event,
    };
    use reqwest::StatusCode;

    use crate::providers::types::{ChatStreamEvent, ProviderError};

    #[test]
    fn strips_reasoning_suffix_from_model() {
        let body = build_codex_request_body(
            "gpt-5.3-codex-high",
            &[crate::providers::types::ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            "",
        );

        assert_eq!(body["model"], "gpt-5.3-codex");
        assert_eq!(body["reasoning"]["effort"], "high");
    }

    #[test]
    fn maps_invalid_session_to_unauthorized() {
        let error = classify_codex_status(
            StatusCode::BAD_REQUEST,
            "{\"error\":{\"message\":\"Failed to look up session id\"}}",
        );

        assert!(matches!(error, ProviderError::Unauthorized));
    }

    #[test]
    fn injects_default_instructions_when_system_prompt_missing() {
        let body = build_codex_request_body(
            "gpt-5.4",
            &[crate::providers::types::ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            "",
        );

        assert_eq!(body["instructions"], DEFAULT_CODEX_INSTRUCTIONS);
    }

    #[test]
    fn maps_response_failed_event_to_retryable_provider_error() {
        let mut saw_output = false;
        let mut saw_sse_activity = false;
        let mut terminal_event_seen = false;

        let event = ParsedSseEvent {
            event: Some("response.failed".to_string()),
            data: r#"{"type":"response.failed","error":{"message":"Too many requests from Codex upstream"}}"#
                .to_string(),
        };

        let mapped = map_sse_event(
            &event,
            &mut saw_output,
            &mut saw_sse_activity,
            &mut terminal_event_seen,
        )
        .unwrap();

        assert!(terminal_event_seen);
        assert!(matches!(
            mapped,
            Some(ChatStreamEvent::Error(ProviderError::RateLimited))
        ));
    }

    #[test]
    fn maps_output_text_done_when_delta_is_missing() {
        let mut saw_output = false;
        let mut saw_sse_activity = false;
        let mut terminal_event_seen = false;

        let event = ParsedSseEvent {
            event: Some("response.output_text.done".to_string()),
            data: r#"{"type":"response.output_text.done","text":"final answer"}"#.to_string(),
        };

        let mapped = map_sse_event(
            &event,
            &mut saw_output,
            &mut saw_sse_activity,
            &mut terminal_event_seen,
        )
        .unwrap();

        assert!(saw_output);
        assert!(matches!(
            mapped,
            Some(ChatStreamEvent::Token(text)) if text == "final answer"
        ));
    }

    #[test]
    fn ignores_output_text_done_after_delta_already_arrived() {
        let mut saw_output = true;
        let mut saw_sse_activity = false;
        let mut terminal_event_seen = false;

        let event = ParsedSseEvent {
            event: Some("response.output_text.done".to_string()),
            data: r#"{"type":"response.output_text.done","text":"duplicate answer"}"#.to_string(),
        };

        let mapped = map_sse_event(
            &event,
            &mut saw_output,
            &mut saw_sse_activity,
            &mut terminal_event_seen,
        )
        .unwrap();

        assert!(mapped.is_none());
    }

    #[test]
    fn invalid_sse_json_after_output_maps_to_upstream_transient() {
        let mut saw_output = true;
        let mut saw_sse_activity = true;
        let mut terminal_event_seen = false;

        let event = ParsedSseEvent {
            event: Some("response.failed".to_string()),
            data: "{oops-invalid-json}".to_string(),
        };

        let mapped = map_sse_event(
            &event,
            &mut saw_output,
            &mut saw_sse_activity,
            &mut terminal_event_seen,
        );

        assert!(matches!(
            mapped,
            Err(ProviderError::UpstreamTransient(message))
                if message.contains("Invalid Codex SSE payload for event Some(\"response.failed\")")
        ));
        assert!(!terminal_event_seen);
    }

    #[test]
    fn does_not_synthesize_success_for_post_output_truncation() {
        let final_event = synthesize_terminal_event(true, true, false);

        assert!(final_event.is_none());
    }

    #[test]
    fn synthesizes_error_when_sse_activity_has_no_terminal_event() {
        let final_event = synthesize_terminal_event(false, true, false);

        assert!(matches!(
            final_event,
            Some(ChatStreamEvent::Error(ProviderError::UpstreamTransient(message)))
                if message == "Codex upstream stream ended without a usable terminal event."
        ));
    }
}
