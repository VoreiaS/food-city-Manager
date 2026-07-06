# Backend Dockerfile — multi-stage build for smaller production image.

# -----------------------------------------------------------------------------
# Stage 1: build
# -----------------------------------------------------------------------------
FROM rust:1.96-slim AS builder

WORKDIR /app

# System deps for sqlx + openssl
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release --bin food-city-backend

# -----------------------------------------------------------------------------
# Stage 2: development image (with cargo-watch)
# -----------------------------------------------------------------------------
FROM rust:1.96-slim AS development

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-watch

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations

EXPOSE 8080

CMD ["cargo", "watch", "-x", "run"]

# -----------------------------------------------------------------------------
# Stage 3: production runtime (distroless)
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS production

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/food-city-backend /app/food-city-backend
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080

CMD ["/app/food-city-backend"]
