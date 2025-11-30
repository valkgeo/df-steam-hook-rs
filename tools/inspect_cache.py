import sqlite3
from pathlib import Path


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    data_dir = repo_root / "df-ptbr-llm-mod" / "data"
    db_file = data_dir / "cache.db"

    if not db_file.exists():
        print(f"cache.db não encontrado em {db_file}")
        return

    conn = sqlite3.connect(db_file)
    cur = conn.cursor()
    cur.execute("SELECT COUNT(*) FROM translations")
    total = cur.fetchone()[0]
    print(f"Total de traduções: {total}")

    cur.execute("SELECT src, dst FROM translations LIMIT 10")
    for src, dst in cur.fetchall():
        print(f"src: {src!r} -> dst: {dst!r}")


if __name__ == "__main__":
    main()
