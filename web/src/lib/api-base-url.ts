const DEFAULT_SERVER_API_ORIGIN = "http://127.0.0.1:3069";
const BROWSER_API_PREFIX = "/backend";

function normalizeOrigin(origin: string): string {
  return origin.replace(/\/+$/, "");
}

function normalizeEndpoint(endpoint: string): string {
  return endpoint.startsWith("/") ? endpoint : `/${endpoint}`;
}

export function getServerApiOrigin(): string {
  return normalizeOrigin(
    process.env.INTERNAL_API_URL ||
      process.env.API_URL ||
      process.env.NEXT_PUBLIC_API_URL ||
      DEFAULT_SERVER_API_ORIGIN
  );
}

export function getApiBaseUrl(): string {
  return typeof window === "undefined" ? getServerApiOrigin() : BROWSER_API_PREFIX;
}

export function buildApiUrl(endpoint: string): string {
  return `${getApiBaseUrl()}${normalizeEndpoint(endpoint)}`;
}
