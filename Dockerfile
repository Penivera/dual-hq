# Multi-stage Dockerfile for internship-api

# --- Stage 1: Build binary ---
FROM rust:1.85-bookworm AS builder

WORKDIR /build

# Pre-compile dependencies for layer caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --bin internship-api && \
    rm -rf src

# Copy real source code and runtime assets
COPY src ./src
COPY assets ./assets
COPY pro_admin ./pro_admin

# Invalidate dummy artifacts and build release binaries
RUN touch src/main.rs src/lib.rs && \
    cargo build --release --bins && \
    cp /build/target/release/internship-api /usr/local/bin/internship-api && \
    cp /build/target/release/seed /usr/local/bin/seed

# --- Stage 2: Runtime image ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /usr/local/bin/internship-api /usr/local/bin/internship-api
COPY --from=builder /usr/local/bin/seed /usr/local/bin/seed

# Copy admin frontend assets and SeaORM Pro configuration
COPY --from=builder /build/assets /app/assets
COPY --from=builder /build/pro_admin /app/pro_admin

# Copy runtime entrypoint and healthcheck scripts
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
COPY healthcheck.sh /usr/local/bin/healthcheck.sh
RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/healthcheck.sh

ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8000
ENV ADMIN_ASSETS_PATH=/app/assets/admin

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD /usr/local/bin/healthcheck.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
