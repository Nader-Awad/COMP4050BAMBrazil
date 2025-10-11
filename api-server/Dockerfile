# syntax = docker/dockerfile:1.6

FROM rust:1.83 AS builder

WORKDIR /app

# Install build dependencies for sqlx (Postgres) and TLS
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev libpq-dev ca-certificates postgresql \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src \
    && echo 'fn main() { println!("placeholder"); }' > src/main.rs \
    && cargo fetch \
    && rm -rf src

# Build application
COPY . .
ENV DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/bam
RUN set -eux; \
    for dir in /etc/postgresql/*; do version="$(basename "$dir")"; pg_ctlcluster "$version" main start; done; \
    su - postgres -c "dropdb --if-exists bam"; \
    su - postgres -c "createdb bam"; \
    su - postgres -c "psql -v ON_ERROR_STOP=1 -c \"ALTER USER postgres WITH PASSWORD 'postgres';\""; \
    for f in /app/migrations/*.sql; do \
        su - postgres -c "psql -v ON_ERROR_STOP=1 -d bam -f \"$f\""; \
    done; \
    cargo build --release; \
    for dir in /etc/postgresql/*; do version="$(basename "$dir")"; pg_ctlcluster "$version" main stop; done

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 libpq5 \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/bam /app/bam

ENV RUST_LOG=info

EXPOSE 3000

CMD ["./bam"]
