# Stage 1: Chef — pre-built image includes cargo-chef, no install step needed
FROM lukemathwalker/cargo-chef:latest-rust-1.97 AS chef
WORKDIR /app

# Stage 2: Planner — cargo metadata requires src/ to resolve [[bin]] / default-run targets,
# but the resulting recipe.json only encodes *dependency* info, so dep-cache layer is still
# invalidated only when Cargo.toml / Cargo.lock change (not on src changes).
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder — compile dependencies (cached), then source
FROM chef AS builder

# 限制 cargo 並行度：本機 prod 部署在筆電 WSL2（6 vCPU）內 build，滿載會把 CPU 推到 93°C、
# M.2 SSD 撞 SMART 溫控跳點（56°C）。預設 4 對 CI（ubuntu-latest = 4 vCPU）等同其原生預設、無損；
# 本機 6→4 壓掉尖峰。設在 builder 開頭，同時涵蓋下方 `cargo chef cook` 與 `cargo build`。
# 要覆寫（例如 self-hosted 大 runner 求快）：docker build --build-arg CARGO_BUILD_JOBS=N。
ARG CARGO_BUILD_JOBS=4
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}

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
COPY build.rs ./build.rs

# R30-22 follow-up：build-time 版本資訊注入給 handlers/version.rs。
# CI / 本地 build 透過 `--build-arg GIT_SHA=$(git rev-parse --short=12 HEAD)` 等傳入。
# 未提供時 build.rs 會 fallback 到 "unknown"（GIT_SHA / RUSTC_VERSION_RUNTIME）
# 或 build.rs 啟動時的 UTC ISO 8601（BUILD_TIME）。
ARG GIT_SHA=""
ARG BUILD_TIME=""
ENV GIT_SHA=${GIT_SHA}
ENV BUILD_TIME=${BUILD_TIME}

RUN cargo build --release --bin erp-backend --bin healthcheck --bin create_admin --bin create_guest --bin provision_legacy_pi_accounts

# Stage 4: Runtime — minimal Debian for user setup
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get upgrade -y && apt-get install -y \
    ca-certificates \
    libssl3 \
    libbz2-1.0 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false appuser && \
    mkdir -p /app/geoip /app/uploads && \
    chown -R appuser:appuser /app

COPY --from=builder /app/target/release/erp-backend /app/erp-backend
COPY --from=builder /app/target/release/healthcheck /app/healthcheck
COPY --from=builder /app/target/release/create_admin /app/create_admin
COPY --from=builder /app/target/release/create_guest /app/create_guest
COPY --from=builder /app/target/release/provision_legacy_pi_accounts /app/provision_legacy_pi_accounts
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
