# Codex Models Reference

Updated: 2026-04-21 21:05:00 +07

Primary source:
- OpenAI official models page: https://developers.openai.com/api/docs/models/all

Purpose:
- Keep one markdown file with the full Codex-related model picture.
- Separate `officially listed` from `currently exposed in local catalog`.
- Mark which models actually work with the current Codex account wired into this DuanAI instance.

## 1. Official OpenAI Coding Models

These are the models listed under the `Coding` section on the OpenAI official `All models` page at the time of writing.

| Slug | Display Name | Deprecated | Local test with current Codex account |
| --- | --- | --- | --- |
| `gpt-5-codex` | GPT-5-Codex | No | Fail: upstream says not supported with ChatGPT account |
| `gpt-5.3-codex` | GPT-5.3-Codex | No | OK |
| `gpt-5.2-codex` | GPT-5.2-Codex | No | Fail: upstream says not supported with ChatGPT account |
| `gpt-5.1-codex` | GPT-5.1 Codex | No | Fail: upstream says not supported with ChatGPT account |
| `gpt-5.1-codex-max` | GPT-5.1-Codex-Max | No | Fail: upstream says not supported with ChatGPT account |
| `gpt-5.1-codex-mini` | GPT-5.1 Codex mini | No | Fail: upstream says not supported with ChatGPT account |
| `codex-mini-latest` | codex-mini-latest | Yes | Fail: upstream says not supported with ChatGPT account |

## 2. Local DuanAI Codex Catalog

The catalog is now intentionally curated down to the slugs that were confirmed working with the current ChatGPT-backed Codex account.

| Slug | Type | Current local result |
| --- | --- | --- |
| `gpt-5.2` | General model routed through Codex provider | OK |
| `gpt-5.3-codex` | Official OpenAI Coding model | OK |
| `gpt-5.4` | Frontier / general model routed through Codex provider | OK |
| `gpt-5.4-mini` | Frontier / general model routed through Codex provider | OK |

## 3. Additional OpenAI Slugs Checked Against Current Codex Setup

These are not all part of the local Codex catalog today, but they were checked because they appear on the official OpenAI `All models` page and are relevant to planning future sync.

### 3.1 Present in OpenAI docs but not currently in local Codex catalog

The local gateway returned `Unknown or inactive model` for these slugs.

- `gpt-5.4-pro`
- `gpt-5.4-nano`
- `gpt-5-mini`
- `gpt-5-nano`
- `gpt-4.1`
- `o3-deep-research`
- `o4-mini-deep-research`
- `gpt-oss-120b`
- `gpt-oss-20b`
- `gpt-5.2-pro`
- `gpt-5-pro`
- `o3-pro`
- `o3`
- `o4-mini`
- `gpt-4.1-mini`
- `gpt-4.1-nano`
- `o1-pro`
- `computer-use-preview`
- `gpt-4o-mini-search-preview`
- `gpt-4o-search-preview`
- `gpt-4.5-preview`
- `o3-mini`
- `o1`
- `o1-mini`
- `o1-preview`
- `gpt-4o`
- `gpt-4o-mini`
- `gpt-4-turbo`
- `babbage-002`
- `chatgpt-4o-latest`
- `davinci-002`
- `gpt-3.5-turbo`
- `gpt-4`
- `gpt-4-turbo-preview`
- `gpt-5.3-chat-latest`
- `gpt-5.2-chat-latest`
- `gpt-5.1-chat-latest`
- `gpt-5-chat-latest`

### 3.2 Official slugs that fail because of current account type

The local gateway reached the upstream, but the upstream rejected these models for the current account with errors of the form:

`The '<model>' model is not supported when using Codex with a ChatGPT account.`

- `gpt-5`
- `gpt-5-codex`
- `gpt-5.2-codex`
- `gpt-5.1`
- `gpt-5.1-codex`
- `gpt-5.1-codex-max`
- `gpt-5.1-codex-mini`
- `codex-mini-latest`

## 4. What Actually Works Right Now

With the current Codex account connected to this DuanAI instance, the following models returned successful `200 OK` responses through:

- `POST /v1/responses`
- local gateway base: `http://127.0.0.1:3069`

Working now:
- `gpt-5.4`
- `gpt-5.4-mini`
- `gpt-5.3-codex`
- `gpt-5.2`

Unsupported slugs are no longer meant to stay active in the local Codex catalog.

## 5. Practical Conclusions

1. The official OpenAI `Coding` list currently has 7 Codex models.
2. The local DuanAI Codex catalog should stay on the 4 runtime-confirmed slugs only.
3. The main blocker is not just local catalog sync. The upstream account type matters.
4. A ChatGPT-backed Codex account can see some Codex/general models but is explicitly blocked from several official Codex slugs.

## 6. Recommended Next Step

Current catalog policy:

- keep only confirmed-working slugs public:
  - `gpt-5.4`
  - `gpt-5.4-mini`
  - `gpt-5.3-codex`
  - `gpt-5.2`
- move unsupported slugs out of the active Codex catalog until a compatible upstream account type is added
- do not auto-publish new Codex slugs without a real runtime test
