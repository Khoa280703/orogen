import { NextResponse } from "next/server";
import { buildApiUrl } from "@/lib/api-base-url";

export async function POST(request: Request) {
  try {
    const contentType = request.headers.get("content-type") || "";
    let idToken = "";

    if (contentType.includes("application/json")) {
      const body = await request.json();
      idToken = body.id_token || "";
    } else {
      const formData = await request.formData();
      idToken = String(formData.get("id_token") || "");
    }

    if (!idToken) {
      return NextResponse.json({ error: "Missing id_token" }, { status: 400 });
    }

    const response = await fetch(buildApiUrl("/auth/google"), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ id_token: idToken }),
    });

    if (!response.ok) {
      const error = await response.text();
      return NextResponse.json({ error: error || "Authentication failed" }, { status: response.status });
    }

    const data = await response.json();
    const redirectResponse = !contentType.includes("application/json");

    if (redirectResponse) {
      const redirect = NextResponse.redirect(new URL("/chat", request.url));
      redirect.cookies.set("access_token", data.access_token, {
        path: "/",
        httpOnly: true,
        maxAge: 60 * 60 * 24 * 30,
      });
      return redirect;
    }

    const json = NextResponse.json({
      success: true,
      user: data.user,
      redirect: "/chat",
    });
    json.cookies.set("access_token", data.access_token, {
      path: "/",
      httpOnly: true,
      maxAge: 60 * 60 * 24 * 30,
    });
    return json;
  } catch (error) {
    console.error("Google auth error:", error);
    return NextResponse.json({ error: "Authentication failed" }, { status: 500 });
  }
}

export async function GET(request: Request) {
  const origin = new URL(request.url).origin;
  const clientId =
    process.env.GOOGLE_CLIENT_ID || process.env.NEXT_PUBLIC_GOOGLE_CLIENT_ID;

  if (!clientId) {
    return NextResponse.redirect(
      `${origin}/login?error=${encodeURIComponent("Google login is not configured.")}`
    );
  }

  const params = new URLSearchParams({
    client_id: clientId,
    redirect_uri: `${origin}/api/auth/google`,
    response_type: "id_token",
    response_mode: "form_post",
    scope: "openid email profile",
    prompt: "consent",
  });

  return NextResponse.redirect(
    `https://accounts.google.com/o/oauth2/v2/auth?${params.toString()}`
  );
}
