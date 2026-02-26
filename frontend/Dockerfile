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
FROM georgjung/nginx-brotli:alpine

# 注意：不執行 apk upgrade，因為升級 nginx 本身會與映像中預編譯的
# brotli 動態模組（.so）產生 ABI 不相容，導致 nginx 無法載入模組而啟動失敗。
# 如需安全更新，請改為拉取更新版本的基底映像（georgjung/nginx-brotli:alpine）。
RUN apk update && rm -rf /var/cache/apk/*

# 主配置：載入 Brotli 模組並 include conf.d
COPY nginx-main.conf /etc/nginx/nginx.conf
# Server block（含 brotli on / brotli_static 等）
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy build output from builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -qO- http://localhost:80/ || exit 1

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
