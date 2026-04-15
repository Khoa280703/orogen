import { NextRequest, NextResponse } from "next/server";
import {
  isDevAuthEnabled,
  listDevUsers,
  readCurrentUserIdFromToken,
} from "@/lib/dev-auth";

export const dynamic = "force-dynamic";

export async function GET(request: NextRequest) {
  if (!isDevAuthEnabled()) {
    return NextResponse.json({ error: "Not found" }, { status: 404 });
  }

  try {
    const users = await listDevUsers();
    const currentUserId = readCurrentUserIdFromToken(
      request.cookies.get("access_token")?.value
    );

    return NextResponse.json({
      users,
      currentUserId,
    });
  } catch (error) {
    return NextResponse.json(
      {
        error: error instanceof Error ? error.message : "Failed to load dev users",
      },
      { status: 500 }
    );
  }
}
