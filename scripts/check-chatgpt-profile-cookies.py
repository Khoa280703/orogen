#!/usr/bin/env python3

import sqlite3
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: python3 scripts/check-chatgpt-profile-cookies.py <profile-dir>", file=sys.stderr)
        return 1

    profile_dir = Path(sys.argv[1])
    cookie_db = profile_dir / "Default" / "Cookies"
    if not cookie_db.exists():
      print(f'{{"ok": false, "error": "Cookie DB not found", "cookieDb": "{cookie_db}"}}')
      return 1

    con = sqlite3.connect(f"file:{cookie_db}?mode=ro", uri=True)
    try:
        cur = con.cursor()
        cur.execute(
            """
            select host_key, name
            from cookies
            where host_key like '%chatgpt.com%'
               or host_key like '%openai.com%'
               or host_key like '%chat.openai.com%'
            order by host_key, name
            """
        )
        rows = cur.fetchall()
    finally:
        con.close()

    if not rows:
        print('{"ok": false, "cookies": [], "message": "No ChatGPT/OpenAI cookies found in this profile."}')
        return 2

    print('{"ok": true, "message": "Found ChatGPT/OpenAI cookies in this profile.", "cookies": [')
    for index, (host_key, name) in enumerate(rows):
        suffix = "," if index < len(rows) - 1 else ""
        print(f'  {{"host": "{host_key}", "name": "{name}"}}{suffix}')
    print("]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
