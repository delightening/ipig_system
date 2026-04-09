# Stage 1: Chef — install cargo-chef for dependency caching
FROM rust:1.92 AS chef

RUN cargo install cargo-chef
WORKDIR /app

# Stage 2: Planner — analyze dependencies
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder — compile dependencies (cached), then source
FROM chef AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies only (this layer is cached when only source changes)
# Filter cargo-chef/Cargo warnings: edition deprecation, unused manifest key (plugin)
ENV SQLX_OFFLINE=true
RUN bash -c 'set -o pipefail; cargo chef cook --release --recipe-path recipe.json 2>&1 | (grep -vE "warning:.*(edition is set on|unused manifest key:)" || true)'

# Now copy real source and build the application
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
COPY resources ./resources

RUN cargo build --release --bin erp-backend --bin healthcheck --bin create_admin --bin create_guest

# Stage 4: Runtime — minimal Debian for user setup
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get upgrade -y && apt-get install -y \
    ca-certificates \
    libssl3 \
    libbz2-1.0 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false appuser && \
    mkdir -p /app/geoip && \
    chown -R appuser:appuser /app

COPY --from=builder /app/target/release/erp-backend /app/erp-backend
COPY --from=builder /app/target/release/healthcheck /app/healthcheck
COPY --from=builder /app/target/release/create_admin /app/create_admin
COPY --from=builder /app/target/release/create_guest /app/create_guest
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/resources /app/resources

# Stage 5: Distroless — production image (libbz2 for zip crate bzip2 feature)
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=runtime /etc/passwd /etc/passwd
COPY --from=runtime /etc/group /etc/group
COPY --from=runtime /app /app
COPY --from=runtime /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=runtime /usr/lib/x86_64-linux-gnu/libbz2.so.1.0* /lib/x86_64-linux-gnu/

USER appuser

ENV RUST_LOG=info

EXPOSE 8000

HEALTHCHECK --interval=10s --timeout=5s --start-period=30s --retries=5 \
  CMD ["/app/healthcheck"]

CMD ["./erp-backend"]
