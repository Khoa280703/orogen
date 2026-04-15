export type ParsedChatStreamEvent =
  | { type: "token"; content: string }
  | { type: "thinking"; content: string }
  | { type: "done" }
  | { type: "error"; message: string };

function parseEventBlock(block: string): ParsedChatStreamEvent | null {
  const lines = block.split("\n");
  let eventType = "message";
  const dataLines: string[] = [];

  for (const rawLine of lines) {
    const line = rawLine.trimEnd();
    if (line.startsWith("event:")) {
      eventType = line.slice(6).trim();
      continue;
    }
    if (line.startsWith("data:")) {
      dataLines.push(line.slice(5).trim());
    }
  }

  if (!dataLines.length) {
    return null;
  }

  const payload = dataLines.join("\n");
  const parsed = payload ? JSON.parse(payload) : {};

  if (eventType === "token") {
    return { type: "token", content: String(parsed?.content || "") };
  }
  if (eventType === "thinking") {
    return { type: "thinking", content: String(parsed?.content || "") };
  }
  if (eventType === "error") {
    return { type: "error", message: String(parsed?.message || "Streaming failed") };
  }
  if (eventType === "done") {
    return { type: "done" };
  }

  return null;
}

export async function* readChatStream(
  response: Response
): AsyncGenerator<ParsedChatStreamEvent> {
  if (!response.body) {
    throw new Error("Streaming response body is missing");
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    buffer += decoder.decode(value || new Uint8Array(), { stream: !done });

    const blocks = buffer.split("\n\n");
    buffer = blocks.pop() || "";

    for (const block of blocks) {
      const event = parseEventBlock(block.trim());
      if (event) {
        yield event;
      }
    }

    if (done) {
      const tail = buffer.trim();
      if (tail) {
        const event = parseEventBlock(tail);
        if (event) {
          yield event;
        }
      }
      return;
    }
  }
}
