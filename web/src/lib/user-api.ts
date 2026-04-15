import { buildApiUrl } from "@/lib/api-base-url";

export interface UserModel {
  id: string;
  display_name?: string;
  description?: string | null;
  owned_by?: string;
}

export interface ChatModelOption {
  id: string;
  label: string;
  provider: string;
  description?: string | null;
}

export interface ImageModelOption {
  id: string;
  label: string;
  provider: string;
  description?: string | null;
}

export interface ConversationListItem {
  id: number;
  user_id: number;
  title: string | null;
  model_slug: string | null;
  active: boolean;
  created_at: string | null;
  updated_at: string | null;
  message_count: number;
}

export interface ConversationMessage {
  id: number;
  conversation_id: number;
  role: string;
  content: string;
  model_slug: string | null;
  provider_slug: string | null;
  tokens_used: number;
  created_at: string | null;
}

export interface ConversationDetail {
  conversation: {
    id: number;
    user_id: number;
    title: string | null;
    model_slug: string | null;
    active: boolean;
    created_at: string | null;
    updated_at: string | null;
  };
  messages: ConversationMessage[];
}

export interface GeneratedImage {
  id: string;
  url: string;
}

export interface GeneratedVideo {
  id: string;
  url: string;
  model_name?: string | null;
  resolution_name?: string | null;
}

export interface ImageGenerationRecord {
  id: number;
  prompt: string;
  model_slug: string;
  status: string;
  images: GeneratedImage[];
  error_message: string | null;
  created_at: string | null;
}

export interface VideoGenerationRecord {
  created: number;
  data: GeneratedVideo[];
  mode: string;
  duration_seconds: number;
  resolution: string;
  errors: string[];
}

const DEFAULT_CHAT_MODELS: ChatModelOption[] = [
  { id: "grok-3", label: "Grok 3", provider: "grok", description: "Balanced default model for fast everyday chat." },
  { id: "grok-4", label: "Grok 4", provider: "grok", description: "Higher quality answers for demanding tasks." },
];

const DEFAULT_IMAGE_MODELS: ImageModelOption[] = [
  { id: "imagine-x-1", label: "Imagine X1", provider: "grok", description: "Fast image generation for everyday creative work." },
  { id: "imagine-x-1-pro", label: "Imagine X1 Pro", provider: "grok", description: null },
];

function isImageModel(modelId: string): boolean {
  return modelId.toLowerCase().startsWith("imagine");
}

export function resolveProviderFromModelId(
  modelId?: string | null,
  providerHint?: string | null
): string {
  const hinted = providerHint?.trim().toLowerCase();
  if (hinted) {
    return hinted;
  }

  const normalized = modelId?.trim().toLowerCase() || "";
  if (
    normalized.startsWith("gpt") ||
    normalized.startsWith("o1") ||
    normalized.startsWith("o3") ||
    normalized.startsWith("o4")
  ) {
    return "openai";
  }
  if (normalized.startsWith("claude")) {
    return "claude";
  }
  if (normalized.startsWith("gemini")) {
    return "gemini";
  }
  if (normalized.startsWith("qwen") || normalized.startsWith("qwq")) {
    return "qwen";
  }
  if (normalized.startsWith("imagine") || normalized.startsWith("grok")) {
    return "grok";
  }

  return "grok";
}

function isHtmlPayload(value: string): boolean {
  const trimmed = value.trim().toLowerCase();
  return trimmed.startsWith("<!doctype html") || trimmed.startsWith("<html");
}

async function parseResponseBody<T>(response: Response): Promise<T> {
  if (response.status === 204 || response.headers.get("content-length") === "0") {
    return undefined as T;
  }

  const contentType = response.headers.get("content-type") || "";
  const text = await response.text();

  if (!text.trim()) {
    return undefined as T;
  }

  if (contentType.includes("application/json")) {
    return JSON.parse(text) as T;
  }

  if (isHtmlPayload(text)) {
    throw new Error("API returned HTML instead of JSON");
  }

  return text as T;
}

async function readErrorMessage(response: Response): Promise<string> {
  try {
    const contentType = response.headers.get("content-type") || "";

    if (contentType.includes("application/json")) {
      const text = await response.text();
      if (!text.trim()) {
        return `API error: ${response.status}`;
      }

      const data = JSON.parse(text);
      if (typeof data?.error === "string") return data.error;
      if (typeof data?.error?.message === "string") return data.error.message;
      if (typeof data?.message === "string") return data.message;
      return JSON.stringify(data);
    }

    const text = await response.text();
    if (isHtmlPayload(text)) {
      return `API error: ${response.status} - API returned HTML instead of JSON`;
    }

    return text.trim() || `API error: ${response.status}`;
  } catch {
    return `API error: ${response.status}`;
  }
}

export async function userApiFetch(
  endpoint: string,
  options: RequestInit = {},
  config: { redirectOnUnauthorized?: boolean } = {}
): Promise<Response> {
  const { redirectOnUnauthorized = true } = config;
  const response = await fetch(buildApiUrl(endpoint), {
    ...options,
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
  });

  if (response.status === 401) {
    if (redirectOnUnauthorized && typeof window !== "undefined") {
      window.location.href = "/login";
    }
    throw new Error("Unauthorized");
  }

  if (!response.ok) {
    throw new Error(await readErrorMessage(response));
  }

  return response;
}

export async function userApiRequest<T>(
  endpoint: string,
  options: RequestInit = {},
  config: { redirectOnUnauthorized?: boolean } = {}
): Promise<T> {
  const response = await userApiFetch(endpoint, options, config);
  return parseResponseBody<T>(response);
}

export async function hasActiveUserSession(): Promise<boolean> {
  try {
    const response = await fetch(buildApiUrl("/user/me"), {
      credentials: "include",
    });
    return response.ok;
  } catch {
    return false;
  }
}

export async function listChatModels(): Promise<ChatModelOption[]> {
  try {
    const payload = await userApiRequest<{ data?: UserModel[] }>("/v1/models");
    if (payload.data?.length) {
      const chatModels = payload.data
        .filter((model) => !isImageModel(model.id))
        .map((model) => ({
          id: model.id,
          label: model.display_name || model.id,
          provider: resolveProviderFromModelId(model.id, model.owned_by),
          description: model.description ?? null,
        }));

      if (chatModels.length) {
        return chatModels;
      }
    }
  } catch {
    // Fall back to defaults.
  }

  return DEFAULT_CHAT_MODELS;
}

export async function createConversation(
  model?: string,
  title?: string,
  config?: { redirectOnUnauthorized?: boolean }
): Promise<ConversationDetail["conversation"]> {
  return userApiRequest<ConversationDetail["conversation"]>("/api/chat/conversations", {
    method: "POST",
    body: JSON.stringify({
      ...(model ? { model } : {}),
      ...(title ? { title } : {}),
    }),
  }, config);
}

export async function listConversations(
  limit = 30,
  offset = 0,
  config?: { redirectOnUnauthorized?: boolean }
): Promise<ConversationListItem[]> {
  return userApiRequest<ConversationListItem[]>(
    `/api/chat/conversations?limit=${limit}&offset=${offset}`,
    {},
    config
  );
}

export async function getConversation(
  conversationId: number | string
): Promise<ConversationDetail> {
  return userApiRequest<ConversationDetail>(`/api/chat/conversations/${conversationId}`);
}

export async function deleteConversation(
  conversationId: number | string
): Promise<void> {
  await userApiRequest<void>(`/api/chat/conversations/${conversationId}`, {
    method: "DELETE",
  });
}

export async function sendMessageStream(
  conversationId: number | string,
  content: string,
  model?: string,
  options?: { signal?: AbortSignal }
): Promise<Response> {
  return userApiFetch(`/api/chat/conversations/${conversationId}/messages`, {
    method: "POST",
    body: JSON.stringify(model ? { content, model } : { content }),
    signal: options?.signal,
    headers: {
      Accept: "text/event-stream",
    },
  });
}

export async function listImageModels(): Promise<ImageModelOption[]> {
  try {
    const payload = await userApiRequest<{ data?: UserModel[] }>("/v1/models");
    if (payload.data?.length) {
      const imageModels = payload.data
        .filter((model) => isImageModel(model.id))
        .map((model) => ({
          id: model.id,
          label: model.display_name || model.id,
          provider: resolveProviderFromModelId(model.id, model.owned_by),
          description: model.description ?? null,
        }));

      if (imageModels.length) {
        return imageModels;
      }
    }
  } catch {
    // Fall back to defaults.
  }

  return DEFAULT_IMAGE_MODELS;
}

export async function generateImages(
  prompt: string,
  model?: string
): Promise<ImageGenerationRecord> {
  return userApiRequest<ImageGenerationRecord>("/api/images/generate", {
    method: "POST",
    body: JSON.stringify(model ? { prompt, model } : { prompt }),
  });
}

export async function listImageHistory(
  limit = 20,
  offset = 0
): Promise<ImageGenerationRecord[]> {
  return userApiRequest<ImageGenerationRecord[]>(
    `/api/images/history?limit=${limit}&offset=${offset}`
  );
}

export async function getImageGeneration(
  generationId: number | string
): Promise<ImageGenerationRecord> {
  return userApiRequest<ImageGenerationRecord>(`/api/images/history/${generationId}`);
}

export async function generateVideos(input: {
  prompt?: string;
  image_url?: string;
  model?: string;
  mode?: string;
  aspect_ratio?: string;
  duration_seconds?: number;
  resolution?: string;
}): Promise<VideoGenerationRecord> {
  return userApiRequest<VideoGenerationRecord>("/api/videos/generate", {
    method: "POST",
    body: JSON.stringify(input),
  });
}
