import { NextResponse } from "next/server";
import { buildApiUrl } from "@/lib/api-base-url";

export async function GET() {
  try {
    const res = await fetch(buildApiUrl("/api/plans"), {
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (!res.ok) {
      return NextResponse.json(
        { error: "Failed to fetch plans" },
        { status: res.status }
      );
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error("Failed to fetch plans:", error);
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    );
  }
}
