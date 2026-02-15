# Build stage
FROM rust:1.92 AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy and build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/erp-backend /app/erp-backend
COPY --from=builder /app/migrations /app/migrations

# 以非 root 用戶運行（SEC-08）
RUN useradd -r -s /bin/false appuser && \
    mkdir -p /app/geoip && \
    chown -R appuser:appuser /app
USER appuser

# Set environment
ENV RUST_LOG=info

EXPOSE 3000

CMD ["./erp-backend"]
