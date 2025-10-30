# IA Service Integration

Place the IA service source code in this directory. The Docker Compose override (`docker-compose.ia.yml`) expects:

- A `Dockerfile` at `IA/Dockerfile` that builds the IA container image.
- An environment file at `IA/.env` containing any variables required by the IA service.

When present, run `make up-ia` from the repository root to start the combined BAM + IA stack. The IA container shares the PostgreSQL instance defined in the root `docker-compose.yml` (`postgres://postgres:postgres@db:5432/bam`).

To rebuild the IA image without starting containers, run `make build-ia`.
