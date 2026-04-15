import { NextRequest, NextResponse } from "next/server";
import { isDevAuthEnabled, listDevUsers, signDevAccessToken } from "@/lib/dev-auth";

export const dynamic = "force-dynamic";

export async function POST(request: NextRequest) {
  if (!isDevAuthEnabled()) {
    return NextResponse.json({ error: "Not found" }, { status: 404 });
  }

  try {
    const body = (await request.json()) as { userId?: number };
    if (typeof body.userId !== "number") {
      return NextResponse.json({ error: "userId is required" }, { status: 400 });
    }

    const users = await listDevUsers();
    const user = users.find((item) => item.id === body.userId);
    if (!user) {
      return NextResponse.json({ error: "User not found" }, { status: 404 });
    }

    const response = NextResponse.json({
      success: true,
      user: {
        id: user.id,
        email: user.email,
        name: user.name,
      },
    });

    response.cookies.set("access_token", signDevAccessToken(user.id, user.email), {
      path: "/",
      httpOnly: true,
      maxAge: 60 * 60 * 24 * 30,
    });

    return response;
  } catch (error) {
    return NextResponse.json(
      {
        error: error instanceof Error ? error.message : "Failed to switch user",
      },
      { status: 500 }
    );
  }
}
