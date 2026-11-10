# RAM Bar

Tauri 2 menu-bar extra. Identifier `com.gestaobem.ram-bar`. Vanilla `web/` (no TypeScript; public formatters use JSDoc).

## Caveats

- SI GB = 10^9, one decimal, Brazilian comma. Dual formatters in `web/js/format.mjs` and `src-tauri/src/memory.rs` with matching goldens. Do not merge them.
- Livre = `total − used`. sysinfo `available_memory` can be 0 on Apple when compressor pages exceed free+inactive+purgeable.
- Details lists apps grouped by `.app` bundle stem (Chrome Helper → Google Chrome), sorted by RSS biggest first, cap 25.
- Skip `set_title` when the formatted title is unchanged; rebuilding NSStatusItem closes an open tray menu.
- Cargo feature `desktop` (default) gates Tauri. Ubuntu CI: `cargo test --lib --no-default-features`. `build.rs` calls `tauri_build` only with `CARGO_FEATURE_DESKTOP`.
- Accessory + `LSUIElement`. Left-click the extra hangs a borderless panel under it (CodexBar-style). The mouseUp of that click lands on the extra: ignore blur for 400ms so the panel stays. Later blur/Escape hides. Sair is in the panel. Panel chrome matches CodexBar: frosted `#111319`, accent `#4e80ee`, cards `#181a1f`, green→red usage meter. Convert the extra rect with **that extra's monitor scale** (`scale_for_tray`); the panel window may be parked on another display (1x Dell vs 2x laptop) and `window.scale_factor()` would misplace it.
- Extra mark is the RAM tab 2×2 grid as a black+alpha template (`tray_grid.rs`, `icon_as_template`) next to the title.
- Panel tabs: RAM (gastões) and Disco (boot volume `/` + other volumes). Disk size is SI TB from 1e12, else GB. Skip Preboot/VM/Update/iSCPreboot/Hardware/Data/`/dev`. APFS `/` and Data report the same container size — drop Data and collapse identical name+size twins. Dual formatters in `format.mjs` and `disk.rs`.
- Login: LaunchAgent `com.gestaobem.ram-bar` in `~/Library/LaunchAgents`, KeepAlive false. The running process writes it only when exe is inside a `.app`. `npm run install:app` copies to `/Applications` (fallback `~/Applications`) and writes the agent. `tauri dev` does not register.
- Local: `npm run dev`. Gate: `npm run ci`.

## Code style

- Functions: 4-20 lines. Split if longer.
- Files: under 500 lines. Split by responsibility.
- One thing per function, one responsibility per module (SRP).
- Names: specific and unique. Avoid `data`, `handler`, `Manager`.
  Prefer names that return <5 grep hits in the codebase.
- Types: explicit. No `any`. JS public APIs get JSDoc; Rust public APIs get rustdoc.
- Dual formatters are the allowed duplication. Extract anything else.
- Early returns over nested ifs. Max 2 levels of indentation for control flow.
- Exception messages must include the offending value and expected shape.

## Comments

- Keep intent/provenance comments. Don't strip them on refactor.
- Write WHY, not WHAT. Skip `// increment counter` above `i++`.
- Docstrings on public functions: intent + one usage example.
- Reference issue numbers / commit SHAs when a line exists because
  of a specific bug or upstream constraint.

## Tests

- Tests run with a single command: `npm test`
  (`node --test test/*.test.mjs` + `cargo test --manifest-path src-tauri/Cargo.toml --lib --no-default-features --quiet`).
- Every new function gets a test. Bug fixes get a regression test.
- Formatters: `@faker-js/faker` (JS) and crate `fake` (Rust). Build `MemorySnapshot` / `{used,total}` literals; do not call sysinfo except `live_snapshot_has_nonzero_total`.
- Tests must be F.I.R.S.T: fast, independent, repeatable,
  self-validating, timely.

## Dependencies

- Inject dependencies through constructor/parameter, not global/import.
- `read_snapshot` is the sysinfo wrapper. Keep formatters pure.
- Centralize config (hosts, model names, feature flags) — one source of truth.

## Structure

- Follow Tauri 2: `src-tauri/` crate, `web/` frontend, `test/*.test.mjs`.
- `memory.rs` snapshot + format; `disk.rs` volumes + TB/GB; `app_spend.rs` group/rank by `.app` bundle; `login_agent.rs` LaunchAgent plist; `panel_anchor.rs` hang-under-extra origin; `tray_grid.rs` extra template; `main.rs` tray/poll/panel; `format.mjs` panel copy; `panel-anchor.mjs` dual of the origin; `panel.js` invoke/listen glue.
- Prefer small focused modules over god files.

## Formatting

- Prettier (`.prettierrc.json`), ESLint, `cargo fmt`. Don't discuss style.
- `npm run ci` = format:check + lint + test.

## Logging

- No observability logger. Ignore tray `set_*` errors so the 2s poller keeps running.

## Defensive programming

Local sysinfo poller. Do not add retries, timeouts, circuit breakers, or rate limits.
