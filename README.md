# RAM Bar

Uso de RAM no menu bar do macOS. Título no estilo `12,4 GB  61%` (SI, uma casa, vírgula). Clique no extra abre o painel preso à bandeja (usado / livre / total e apps pelo mais gastão).

[![CI](https://github.com/puppe1990/ram-bar/actions/workflows/ci.yml/badge.svg)](https://github.com/puppe1990/ram-bar/actions/workflows/ci.yml)

## Comandos

```bash
cd produtos/ram-bar
npm test          # node --test + cargo test --lib
npm run lint
npm run format:check
npm run ci        # prettier + eslint + tests
npm run dev         # Tauri 2
npm run install:app # /Applications + abrir no login do Mac
```

Pre-commit: lint-staged (ESLint + Prettier) e `npm test`.

A leitura vem do `sysinfo` (o mesmo recorte de used/available/total). Gigabyte é SI, como o Monitor de Atividade recente.

```
sysinfo → MemorySnapshot → tray title
                        └→ event "memory" → panel (preso ao extra)
```
