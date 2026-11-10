# RAM Bar

Tauri 2 accessory que mostra RAM usada e o percentual no menu bar do macOS. Identifier `com.gestaobem.ram-bar`. Frontend vanilla em `web/`. Formatação SI (1 GB = 10^9 bytes, vírgula) em `web/js/format.mjs` e `src-tauri/src/memory.rs`.

Um comando de teste: `npm test` (na pasta `produtos/ram-bar`). CI: `npm run ci` (Prettier, ESLint, testes). Pre-commit husky: `lint-staged` + `npm test`.

Cargo feature `desktop` (default) porta o Tauri; `lib.rs` expõe `memory` para o CI Ubuntu não precisar de GTK. `build.rs` chama `tauri_build` só com `CARGO_FEATURE_DESKTOP`.

Local: `npm run dev` (tauri dev). App de menu bar: `LSUIElement` + `ActivationPolicy::Accessory`. Clique abre o menu; **Detalhes** mostra o painel.
