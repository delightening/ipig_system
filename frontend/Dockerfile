# ============================================
# Stage 1: Build frontend
# ============================================
FROM node:22-alpine AS builder

WORKDIR /app

# Copy package files first for layer caching
COPY package.json package-lock.json ./
RUN npm ci --ignore-scripts

# Copy source and build
COPY . .
RUN npm run build

# ============================================
# Stage 2: Serve with nginx (Brotli 支援)
# ============================================
FROM georgjung/nginx-brotli:1.29.5-alpine

# 版本釘選：nginx 1.29.5 + Alpine 3.23（2026-02-05 發佈）
# 不執行 apk upgrade — 會與預編譯 Brotli 動態模組 ABI 不相容。
# 安全更新透過升級基底映像版號處理，見 docs/security.md。
RUN apk update && rm -rf /var/cache/apk/*

# 主配置：載入 Brotli 模組並 include conf.d
COPY nginx-main.conf /etc/nginx/nginx.conf
# Server block（含 brotli on / brotli_static 等）- 作為模板使用
COPY nginx.conf /etc/nginx/conf.d/default.conf.template

# Copy build output from builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -qO- http://127.0.0.1:80/ || exit 1

EXPOSE 80

# 使用 envsubst 在運行時替換環境變數（如果配置被 volume mount 覆蓋則跳過），然後啟動 nginx
# 使用 || true 確保即使 envsubst 失敗（例如因為 volume mount 是只讀的）也能繼續啟動
CMD ["/bin/sh", "-c", "(envsubst '${API_BACKEND_URL}' < /etc/nginx/conf.d/default.conf.template > /etc/nginx/conf.d/default.conf 2>/dev/null || true) && exec nginx -g 'daemon off;'"]
