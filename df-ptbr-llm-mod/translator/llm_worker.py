import json
import sqlite3
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

MODEL = "qwen2.5:3b"
OLLAMA_URL = "http://localhost:11434/api/chat"

BASE_DIR = Path(__file__).resolve().parents[1]  # .../df-ptbr-llm-mod
DATA_DIR = BASE_DIR / "data"
PENDING_FILE = DATA_DIR / "pending.txt"
DB_FILE = DATA_DIR / "cache.db"


def log(msg: str) -> None:
    print(f"[LLM WORKER] {msg}")


def ensure_db() -> sqlite3.Connection:
    conn = sqlite3.connect(DB_FILE)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS translations (src TEXT PRIMARY KEY, dst TEXT NOT NULL)"
    )
    return conn


def read_pending() -> list[str]:
    if not PENDING_FILE.exists():
        return []
    raw = PENDING_FILE.read_text(encoding="utf-8", errors="replace").splitlines()
    # clear file eagerly to avoid reprocessing if we crash later
    PENDING_FILE.write_text("", encoding="utf-8")
    return [line.strip() for line in raw if line.strip()]


def translate_line(text: str) -> str | None:
    payload = {
        "model": MODEL,
        "stream": False,
        "messages": [
            {
                "role": "system",
                "content": (
                    "You are a professional translator. Translate Dwarf Fortress UI/game text "
                    "from English to Brazilian Portuguese (pt-BR). Use natural Brazilian Portuguese, "
                    "preserve names of dwarves, places, items and deities, preserve formatting, "
                    "and avoid literal translations that sound unnatural."
                ),
            },
            {"role": "user", "content": text},
        ],
    }
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        OLLAMA_URL, data=data, headers={"Content-Type": "application/json"}
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            obj = json.load(resp)
            content = obj.get("message", {}).get("content", "")
            if content:
                return content.strip()
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as exc:
        log(f"Erro ao traduzir '{text}': {exc}")
    return None


def save_translations(conn: sqlite3.Connection, items: list[tuple[str, str]]) -> None:
    if not items:
        return
    conn.executemany(
        "INSERT OR REPLACE INTO translations (src, dst) VALUES (?, ?)", items
    )
    conn.commit()


def main() -> int:
    log(f"Iniciando. Base: {BASE_DIR}")
    log(f"Usando modelo: {MODEL}")

    pending = read_pending()
    if not pending:
        log("Nada para traduzir.")
        return 0

    log(f"{len(pending)} novas linhas para traduzir.")
    conn = ensure_db()

    translated: list[tuple[str, str]] = []
    for line in pending:
        log(f"Traduzindo: '{line}'")
        result = translate_line(line)
        if result:
            translated.append((line, result))
        time.sleep(0.05)  # leve respiro

    save_translations(conn, translated)
    log(f"{len(translated)} traduções salvas no cache.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
