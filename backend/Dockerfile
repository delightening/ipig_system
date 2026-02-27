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
RUN cargo build --release --bin erp-backend --bin healthcheck --bin create_admin

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies and security upgrades
RUN apt-get update && apt-get upgrade -y && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create appuser (since distroless doesn't have useradd, we do it here)
RUN useradd -r -s /bin/false appuser && \
    mkdir -p /app/geoip && \
    chown -R appuser:appuser /app

# Copy binaries and migrations
COPY --from=builder /app/target/release/erp-backend /app/erp-backend
COPY --from=builder /app/target/release/healthcheck /app/healthcheck
COPY --from=builder /app/target/release/create_admin /app/create_admin
COPY --from=builder /app/migrations /app/migrations

# Using distroless for final stage to minimize vulnerabilities
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

# Copy from runtime stage (which has correctly set up files)
COPY --from=runtime /etc/passwd /etc/passwd
COPY --from=runtime /etc/group /etc/group
COPY --from=runtime /app /app
COPY --from=runtime /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

USER appuser

# Set environment
ENV RUST_LOG=info

EXPOSE 8000

# Health check using compiled binary (distroless has no shell)
HEALTHCHECK --interval=10s --timeout=5s --start-period=30s --retries=5 \
  CMD ["/app/healthcheck"]

CMD ["./erp-backend"]
