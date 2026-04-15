import { NextRequest, NextResponse } from "next/server";
import { getServerApiOrigin } from "@/lib/api-base-url";

export const dynamic = "force-dynamic";

const REQUEST_BODY_METHODS = new Set(["POST", "PUT", "PATCH", "DELETE"]);
const REQUEST_HEADER_BLACKLIST = new Set(["host", "connection", "content-length", "cookie"]);
const RESPONSE_HEADER_BLACKLIST = new Set(["connection", "content-encoding", "transfer-encoding"]);

function buildUpstreamUrl(request: NextRequest): string {
  const upstreamPath = request.nextUrl.pathname.replace(/^\/backend/, "") || "/";
  const search = request.nextUrl.search || "";
  return `${getServerApiOrigin()}${upstreamPath}${search}`;
}

function copyRequestHeaders(request: NextRequest): Headers {
  const headers = new Headers();

  request.headers.forEach((value, key) => {
    if (!REQUEST_HEADER_BLACKLIST.has(key.toLowerCase())) {
      headers.set(key, value);
    }
  });

  const accessToken = request.cookies.get("access_token")?.value;
  if (accessToken && !headers.has("authorization")) {
    headers.set("authorization", `Bearer ${accessToken}`);
  }

  return headers;
}

function copyResponseHeaders(response: Response): Headers {
  const headers = new Headers();

  response.headers.forEach((value, key) => {
    if (!RESPONSE_HEADER_BLACKLIST.has(key.toLowerCase())) {
      headers.set(key, value);
    }
  });

  return headers;
}

async function forwardRequest(request: NextRequest): Promise<NextResponse> {
  try {
    const init: RequestInit = {
      method: request.method,
      headers: copyRequestHeaders(request),
      cache: "no-store",
    };

    if (REQUEST_BODY_METHODS.has(request.method)) {
      init.body = await request.text();
    }

    const upstreamResponse = await fetch(buildUpstreamUrl(request), init);

    return new NextResponse(upstreamResponse.body, {
      status: upstreamResponse.status,
      headers: copyResponseHeaders(upstreamResponse),
    });
  } catch (error) {
    console.error("Backend proxy failed:", error);
    return NextResponse.json(
      { error: "Upstream API unavailable" },
      { status: 502 }
    );
  }
}

export async function GET(request: NextRequest) {
  return forwardRequest(request);
}

export async function POST(request: NextRequest) {
  return forwardRequest(request);
}

export async function PUT(request: NextRequest) {
  return forwardRequest(request);
}

export async function PATCH(request: NextRequest) {
  return forwardRequest(request);
}

export async function DELETE(request: NextRequest) {
  return forwardRequest(request);
}

export async function OPTIONS(request: NextRequest) {
  return forwardRequest(request);
}
