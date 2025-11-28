# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Vue 3 renderer in TypeScript; key UI lives in `src/components/ui`, layout shell in `src/components/layout`, shared helpers in `src/utils`, and shared shapes in `src/types`.
- `electron/`: Main process entry (`main.ts`), preload bridge (`preload.cts`), and database clients in `electron/db` (sqlite, postgres, mysql, mongodb).
- `public/` hosts static assets; `dist-electron/` is generated output for the main process; `scripts/` holds dev helpers (e.g., `run-dev-electron.cjs`).
- Configuration defaults belong in `config.toml.example`; copy to `config.toml` locally and keep secrets out of Git.

## Build, Test, and Development Commands
- `bun install` — install dependencies (project expects Bun for scripts).
- `bun run dev` — run renderer and main together with watch mode (Vite + Electron).
- `bun run dev:renderer` / `bun run dev:main` — run either side in isolation while iterating.
- `bun run build:electron` — type-check and emit `dist-electron/main.js` for the main process.
- `bun run build` — full production build: Electron main, Vue type-check (`vue-tsc`), Vite bundle, and packaging via `electron-builder`.
- `bun run preview` — serve the built renderer bundle for quick smoke checks.

## Coding Style & Naming Conventions
- TypeScript is `strict`; avoid `any` and prefer explicit return types and typed IPC payloads.
- Vue SFCs use `<script setup>`; components are PascalCase (`NebulaHeader.vue`) and live in matching filenames. Functions/vars are camelCase; enums/constants are UPPER_SNAKE_CASE.
- Keep renderer/main separation: expose only vetted APIs through `electron/preload.cts`, never direct Node access from Vue.
- Styling is Tailwind v4 + custom CSS tokens in `src/style.css`; favor utility classes over ad-hoc inline styles. Indent with two spaces and keep imports ordered (types before values).

## Testing Guidelines
- Automated tests are not wired yet; add unit/component specs as `*.spec.ts` under `src/__tests__` or alongside components.
- Use `bun run build` as a pre-flight to catch type errors and bundling issues; smoke-test the app with `bun run dev` and verify database connections with the sample config.
- When adding DB-dependent code, allow injectable connection strings so tests can stub or point to local containers instead of production services.

## Commit & Pull Request Guidelines
- Follow the existing short, imperative commit style (`change dev setup`, `huge refactor`); keep each commit scoped and avoid bundling unrelated changes.
- PRs should include: a concise summary, before/after screenshots or clips for UI changes, commands run (build/smoke), and linked issues or tasks.
- Call out any new config flags or migration steps (e.g., updates to `config.toml`) in the PR description, and ensure Electron preload/IPC changes are documented for reviewers.

## Security & Configuration Tips
- Do not commit credentials or connection strings; store them in untracked `config.toml` or environment variables.
- Keep the preload surface minimal; prefer typed, narrow IPC channels and validate all inputs from the renderer before hitting the database clients in `electron/db`.
- When adding dependencies, confirm they are compatible with the Electron runtime and avoid modules that require remote code execution or uncontrolled native binaries.
