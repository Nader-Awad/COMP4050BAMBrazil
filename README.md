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

1. Copy `.env.example` to `.env` (if present) and populate required secrets such as `DATABASE_URL`, `JWT_SECRET`, and file-storage paths.
2. Run `make` (alias for `make up`) from the repository root:
   - Builds the backend/ frontend images.
   - Starts `db` (PostgreSQL), `api` (Axum server on `http://localhost:3000`), and `frontend` (Vite build served by Nginx on `http://localhost:5173`).
3. Stop the stack with `make down` when you are done.

## Useful Commands

| Command | Description |
| ------- | ----------- |
| `make` / `make up` | Build images and start all services defined in `docker-compose.yml`. |
| `make down` | Stop containers and remove default networks (volumes are kept). |
| `make build` | Rebuild images without starting the stack. |
| `make logs` | Tail logs from all services. |
| `make ps` | Show container status. |
| `make prune` | Stop the stack then run `docker system prune -af --volumes` to reclaim space. |

> **Tip:** If you modify backend SQL queries or migrations, rebuild the `api` image with `docker compose build api` (or `make build`) so compile-time SQL checks run against the updated schema.

## Local Development (Optional)

If you prefer not to use Docker:

- Backend: ensure PostgreSQL is running and execute `cargo run` from `api-server/`. Run migrations with `sqlx migrate run`.
- Frontend: inside `BAM_UI/`, install dependencies (`npm install`) and start Vite with `npm run dev`.

Refer to `AGENTS.md` for contributor guidelines, coding standards, and testing expectations.
