# ============================================
# Stage 1: Build frontend (pnpm via corepack)
# ============================================
FROM node:24-alpine AS builder

WORKDIR /app

# 啟用 corepack 並啟用 package.json 中 "packageManager" 指定的 pnpm 版本
# 避免 pnpm 版本漂移導致 lockfile 格式不相容
RUN corepack enable

# Copy package files first for layer caching
COPY package.json pnpm-lock.yaml ./
# 提高 fetch 重試與超時，應付 ECONNRESET 等短暫網路問題
# --frozen-lockfile：lock 和 package.json 必須完全一致，否則 fail（等效 npm ci）
# --ignore-scripts：略過套件 postinstall（等效 npm ci --ignore-scripts）
RUN pnpm config set fetch-retries 5 && \
    pnpm config set fetch-retry-maxtimeout 120000 && \
    pnpm install --frozen-lockfile --ignore-scripts

# Copy source and build
COPY . .
RUN pnpm run build

# ============================================
# Stage 2: Serve with nginx (Brotli 支援)
# ============================================
FROM georgjung/nginx-brotli:1.29.5-alpine

# 版本釘選：nginx 1.29.5 + Alpine 3.23（2026-02-05 發佈）
# 僅精準升級有 CVE 的套件，避免全系統 apk upgrade 導致 Brotli ABI 不相容。
# - zlib: CVE-2026-22184
# - libexpat: CVE-2026-32767 (2.7.4 -> 2.7.5)
# - libpng: CVE-2026-33416, CVE-2026-33636 (1.6.55-r0 -> 1.6.56-r0)
# - musl/musl-utils: CVE-2026-40200 (1.2.5-r21 -> 1.2.5-r23, stack-based buffer overflow)
# - libcrypto3/libssl3: CVE-2026-31789 (CRITICAL, heap buffer overflow), CVE-2026-28387/28388/28389 (HIGH,
#   DANE TLSA use-after-free + CMS/delta NULL deref DoS) — 3.5.5-r0 → 3.5.6-r0，同主版本 ABI 相容
# 其他安全更新透過升級基底映像版號處理，見 docs/security.md。
RUN apk update && apk upgrade zlib libexpat libpng musl musl-utils libcrypto3 libssl3 && rm -rf /var/cache/apk/*

# 主配置：載入 Brotli 模組並 include conf.d
COPY nginx-main.conf /etc/nginx/nginx.conf
# Server block（含 brotli on / brotli_static 等）- 作為模板使用
# 放在 /etc/nginx/templates/ 而非 conf.d/，避免 conf.d 掛成 tmpfs 後模板被遮蔽
RUN mkdir -p /etc/nginx/templates
COPY nginx.conf /etc/nginx/templates/default.conf.template
# 共用安全標頭 snippet（HSTS/CSP/X-Frame-Options 等），由 nginx.conf 多處 include
COPY security-headers.conf /etc/nginx/snippets/security-headers.conf

# Copy build output from builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# 非 root 運行：設定目錄權限給 nginx 使用者
RUN chown -R nginx:nginx /usr/share/nginx/html \
    /var/cache/nginx /var/log/nginx /etc/nginx/conf.d && \
    touch /var/run/nginx.pid && \
    chown nginx:nginx /var/run/nginx.pid && \
    mkdir -p /tmp/nginx && chown nginx:nginx /tmp/nginx

USER nginx

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -qO- http://127.0.0.1:8080/ || exit 1

EXPOSE 8080

# 啟動流程：
#   1) 建立 /tmp/nginx temp 目錄（/tmp 是 tmpfs，容器啟動時為空，需 runtime 建立）
#   2) 以 envsubst 代入 ${API_BACKEND_URL}，將 template 渲染至 /etc/nginx/conf.d/default.conf
#      （/etc/nginx/conf.d 在 compose 以 tmpfs 掛載，可寫）
#   3) 啟動 nginx
CMD ["/bin/sh", "-c", "mkdir -p /tmp/nginx && envsubst '${API_BACKEND_URL}' < /etc/nginx/templates/default.conf.template > /etc/nginx/conf.d/default.conf && exec nginx -g 'daemon off;'"]
