# BAM Brazil Toolkit

This repository contains the Bioscope Allocation & Management (BAM) tooling used in COMP4050. It is split into two subsystems:

- `api-server/`: Rust (Axum + SQLx) backend providing booking, session, and IA integration endpoints.
- `BAM_UI/`: React + Vite frontend that visualises bookings, approvals, and analytics.

Both services are containerised and can be orchestrated together with Docker Compose and the provided `Makefile`.

## Prerequisites

- Docker 24+ (includes Docker Compose v2)
- GNU Make
- (Optional for local builds) Rust 1.82+ and Node 20+ if you prefer to run the services outside containers.

## Quick Start with Docker

1. Copy the environment templates if you plan to override defaults:
   - `cp api-server/.env.example api-server/.env` (optional — the backend falls back to `.env.example`, but an explicit `.env` is recommended for secrets).
   - `cp BAM_UI/.env.example BAM_UI/.env` and update `VITE_API_URL` if the API will be hosted elsewhere.
2. Run `make` (alias for `make up`) from the repository root:
   - Builds the backend/frontend images.
   - Starts `db` (PostgreSQL), `api` (Axum server on `http://localhost:3000`), and `frontend` (static Vite bundle served by Nginx on `http://localhost:5173`).
3. Stop the stack with `make down` when you are done.

For backend-focused development, run `make dev`. It starts only the `api` and `db` containers with `IA_MOCK_MODE=true`, letting you run the React app locally via `npm run dev` without pulling in the IA service.

To include the IA service, place their source in `IA/`, provide an `IA/.env`, and run `make up-ia`. This composes the BAM stack with the IA container while reusing the shared PostgreSQL database.

> **Default logins** (seeded via the initial migration)
> - `admin@bam.edu` / `admin123`
> - `teacher@bam.edu` / `teacher123`
> - `student@bam.edu` / `student123`

## Useful Commands

| Command | Description |
| ------- | ----------- |
| `make` / `make up` | Build images and start all services defined in `docker-compose.yml`. |
| `make dev` | Start `api` and `db` only with `IA_MOCK_MODE=true` for local frontend development. |
| `make down` | Stop containers and remove default networks (volumes are kept). |
| `make build` | Rebuild images without starting the stack. |
| `make up-ia` | Start the full BAM stack plus the IA service using `docker-compose.ia.yml`. |
| `make build-ia` | Build images for the BAM + IA stack without starting containers. |
| `make logs-ia` | Tail logs from the combined BAM + IA services. |
| `make down-ia` | Stop the combined BAM + IA stack. |
| `make logs` | Tail logs from all services. |
| `make ps` | Show container status. |
| `make prune` | Stop the stack then run `docker system prune -af --volumes` to reclaim space. |

> **Tip:** If you modify backend SQL queries or migrations, rebuild the `api` image with `docker compose build api` (or `make build`) so compile-time SQL checks run against the updated schema.
> The backend runs migrations automatically on startup (`sqlx::migrate!()`). If you need to apply them manually, run `sqlx migrate run` inside `api-server/`.

## Local Development (Optional)

If you prefer not to use Docker:

### Backend (`api-server/`)
- Ensure PostgreSQL is running and that `DATABASE_URL` is exported (or present in `.env`).
- Run migrations with `sqlx migrate run` (install `sqlx-cli` with `cargo install sqlx-cli --no-default-features --features rustls`).
- Start the server with `cargo run`. The service listens on `http://localhost:3000` by default and seeds the admin/teacher/student accounts if they do not already exist.

### Frontend (`BAM_UI/`)
- Install dependencies with `npm install`.
- Configure `VITE_API_URL` in `.env` (defaults to `http://localhost:3000`).
- Launch the dev server with `npm run dev` and open `http://localhost:5173`.

Tokens are stored in `localStorage`; the frontend fetch client attaches them as bearer tokens and automatically refreshes when needed.

Refer to `AGENTS.md` for contributor guidelines, coding standards, and testing expectations.

## IA Integration

- IA collaborators should place their project (including `Dockerfile` and supporting assets) inside the `IA/` directory. See `IA/README.md` for expectations.
- `docker-compose.ia.yml` layers the IA container on top of the core stack while reusing the shared PostgreSQL service.
- Use `make up-ia`/`make down-ia` to run or stop the combined environment, and `make build-ia` when you only need to rebuild images.
- The standard BAM development flow (`make dev` + `npm run dev`) remains available and does not require the IA codebase.
