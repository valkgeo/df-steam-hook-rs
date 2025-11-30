# df-ptbr-llm-mod translator

Este worker lê `data/pending.txt`, chama o Ollama (`qwen2.5:3b`) e grava traduções em `data/cache.db`.

## Requisitos
- Python 3.10+
- Ollama rodando localmente e com o modelo `qwen2.5:3b` disponível.

## Uso
```bash
cd df-ptbr-llm-mod/translator
python llm_worker.py
```

Mensagens de log mostram quantas linhas foram processadas e salvas no cache. O hook Rust lerá `cache.db` na próxima vez que a mesma string aparecer no jogo.
