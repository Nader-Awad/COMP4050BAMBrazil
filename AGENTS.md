# Repository Guidelines

## Project Structure & Module Organization
- `api-server/` hosts the Axum backend; request flow sits in `src/handlers`, `models`, `services`, and `middleware`, shared state in `lib.rs`, integration tests in `tests/`, and SQLx migrations in `migrations/`.
- `BAM_UI/` houses the Vite + React app; reusable UI sits in `src/components`, shared logic in `src/hooks` and `src/context`, and domain contracts in `src/types`.

## Build, Test, and Development Commands
- Docker stack: `make` (or `make up`) builds images and starts Postgres, the API, and the frontend; stop everything with `make down` and reclaim space with `make prune`.
- Backend: target Rust 1.82+ (see `rustup override set 1.82.0`); `cargo run` boots the API once `.env` is populated, and `DATABASE_URL=postgres://... sqlx migrate run` applies migrations.
- Backend quality: run `cargo fmt` and `cargo clippy --all-targets --all-features` before pushing.
- Frontend: inside `BAM_UI/` run `npm install`, then `npm run dev`, `npm run build`, and `npm run lint`.

## Coding Style & Naming Conventions
- Rust modules follow snake_case filenames and favour `Result<T, anyhow::Error>` with `thiserror` variants; document public handlers and keep them concise.
- TypeScript uses 2-space indentation, strict mode, and PascalCase components; rely on the configured aliases (e.g. `@components/*`) instead of relative paths.
- Keep Tailwind utility classes inline and lift recurring patterns into `src/lib` helpers instead of new CSS.

## Testing Guidelines
- Run `cargo test` for backend coverage and extend the async helpers in `api-server/tests/integration_tests.rs` instead of hitting real I/O or secrets.
- When routes require auth, reuse the `create_auth_request` pattern to keep tokens isolated from assertions.
- Frontend testing is not yet configured; at minimum run `npm run lint`, and add Vitest or React Testing Library coverage before merging major UI changes.

## Commit & Pull Request Guidelines
- Follow the local style: short, imperative summaries with optional scopes (e.g. `refactor(ui): split layout panels` or `calleum/21: update IA client wiring`) and reference issues when possible.
- Squash WIP commits before opening a PR; describe the change, call out migrations or env vars, and highlight backend versus frontend impact.
- Add UI screenshots or terminal captures when appropriate, and only request review after linting, formatting, and tests succeed locally.

## Environment & Configuration Tips
- Copy `.env.example` if present or set at least `DATABASE_URL`, `JWT_SECRET`, and the file storage paths required by `Config::from_env`; use `.env.local` for machine-specific overrides.
- Enable the `uuid-ossp` extension in PostgreSQL before running migrations, and keep `uploads/` writable when testing the file-store service.
