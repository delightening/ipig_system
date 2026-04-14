# ============================================
# Stage 1: Build frontend
# ============================================
FROM node:24-alpine AS builder

WORKDIR /app

# Copy package files first for layer caching
COPY package.json package-lock.json ./
# 提高 fetch 重試與超時，應付 ECONNRESET 等短暫網路問題
RUN npm config set fetch-retries 5 && \
    npm config set fetch-retry-maxtimeout 120000 && \
    npm ci --ignore-scripts

# Copy source and build
COPY . .
RUN npm run build

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
# 其他安全更新透過升級基底映像版號處理，見 docs/security.md。
RUN apk update && apk upgrade zlib libexpat libpng musl musl-utils && rm -rf /var/cache/apk/*

# 主配置：載入 Brotli 模組並 include conf.d
COPY nginx-main.conf /etc/nginx/nginx.conf
# Server block（含 brotli on / brotli_static 等）- 作為模板使用
COPY nginx.conf /etc/nginx/conf.d/default.conf.template

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

# 使用 envsubst 在運行時替換環境變數（如果配置被 volume mount 覆蓋則跳過），然後啟動 nginx
# 使用 || true 確保即使 envsubst 失敗（例如因為 volume mount 是只讀的）也能繼續啟動
CMD ["/bin/sh", "-c", "(envsubst '${API_BACKEND_URL}' < /etc/nginx/conf.d/default.conf.template > /etc/nginx/conf.d/default.conf 2>/dev/null || true) && exec nginx -g 'daemon off;'"]
