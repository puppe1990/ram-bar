# RAM Bar

Uso de RAM no menu bar do macOS. Título no estilo `12,4 GB  61%` (SI, uma casa, vírgula). Clique abre usado / livre / total; **Detalhes** mostra o painel.

[![CI](https://github.com/puppe1990/ram-bar/actions/workflows/ci.yml/badge.svg)](https://github.com/puppe1990/ram-bar/actions/workflows/ci.yml)

## Comandos

```bash
cd produtos/ram-bar
npm test          # node --test + cargo test --lib
npm run lint
npm run format:check
npm run ci        # prettier + eslint + tests
npm run dev       # Tauri 2
```

Pre-commit: lint-staged (ESLint + Prettier) e `npm test`.

A leitura vem do `sysinfo` (o mesmo recorte de used/available/total). Gigabyte é SI, como o Monitor de Atividade recente.
