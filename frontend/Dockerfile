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
# E2E 覆蓋率：docker-compose.test.yml 傳 ARG 進來，觸發 vite 產 hidden source map
ARG VITE_E2E_COVERAGE=0
ENV VITE_E2E_COVERAGE=${VITE_E2E_COVERAGE}
RUN pnpm run build

# ============================================
# Stage 2: Serve with nginx (Brotli 支援)
# ============================================
FROM georgjung/nginx-brotli:1.31.2-alpine3.23

# 版本釘選：nginx 1.31.0 + Alpine 3.23（2026-05-13 發佈，新 mainline series）
# 升級理由：1.29.x 在 1.31.0 釋出後等於 EOL（mainline 切到新 series，未來 CVE
# patch 只走 1.31.x）；R56 AWS migration 為 park 狀態（規劃中、未實作），不能
# 等 3 個月才補資安。
#
# 1.29.5 → 1.31.0 升級涵蓋的 nginx core CVE：
# - CVE-2026-42945 (Critical, heap buffer overflow in rewrite module → arbitrary code exec)
# - CVE-2026-42946 (HIGH, scgi/uwsgi buffer overread → memory disclosure)
# - CVE-2026-42934 (UTF-8 charset_map memory disclosure)
# - CVE-2026-42926 (HTTP/2 backend data injection via proxy_set_body)
# - CVE-2026-40460 (HIGH, HTTP/3 connection migration spoofing)
# - CVE-2026-40701 (HIGH, use-after-free in ssl_ocsp DNS resolution)
# - CVE-2026-27654 (alias 路徑穿越 — 我們沒用 alias 但 binary 層仍修)
#
# Config 相容性驗證（對應我們既有 directive）：
# - sub_filter / $request_id / brotli / gzip / limit_req / try_files / add_header → CHANGES 無變動
# - proxy_http_version 1.1 → 已顯式設定，1.29.7 default 變動無影響
# - listen 8080 http2 + 1.31 HTTP/2 嚴格 header reject → 無 WebSocket 使用，browser
#   client 不送 Connection/Upgrade over HTTP/2，無實際影響
# - DAV / MP4 / CRAM-MD5 modules → 全沒用
#
# 僅精準升級有 CVE 的套件，避免全系統 apk upgrade 導致 Brotli ABI 不相容。
# - zlib: CVE-2026-22184
# - libexpat: CVE-2026-32776 / 32777 / 32778 (DTD parsing + NULL deref, 2.7.4 -> 2.7.5)
#   (CodeRabbit PR #419：原 comment 誤寫 CVE-2026-32767，該 CVE 實際是 SiYuan 知識管理系統的漏洞，
#    apk upgrade libexpat 2.7.5 涵蓋的真實 CVE 編號為 32776/32777/32778)
# - libpng: CVE-2026-33416, CVE-2026-33636 (1.6.55-r0 -> 1.6.56-r0)
# - musl/musl-utils: CVE-2026-40200 (1.2.5-r21 -> 1.2.5-r23, stack-based buffer overflow)
# - libcrypto3/libssl3: CVE-2026-31789 (CRITICAL, heap buffer overflow), CVE-2026-28387/28388/28389 (HIGH,
#   DANE TLSA use-after-free + CMS/delta NULL deref DoS) — 3.5.5-r0 → 3.5.6-r0，同主版本 ABI 相容
# - nghttp2/nghttp2-libs: CVE-2026-27135 (HIGH, DoS via malformed HTTP/2 frames after session
#   termination) — 1.68.0-r0 → 1.68.1-r0
# - libxml2: CVE-2026-6732 (DoS via XSLT XML/XSD parsing) — 2.13.9-r0 → 2.13.9-r1
#   靜態檔服務未載入 ngx_http_xslt_module，但 binary 層一併補。（P4-6 2026-06-09）
# - c-ares: CVE-2026-33630 (HIGH, use-after-free / double-free in query-completion
#   handling，可經 ares_getaddrinfo() over TCP 遠端觸發) — 1.34.6-r0 → 1.34.8-r0
#   （upstream 1.34.7 修，Alpine 3.23 收錄於 1.34.8-r0，故 apk upgrade 即拉到修復版）。
#   nginx 雖有設定 resolver directive（nginx.conf:12，供 API proxy_pass 動態解析），
#   但用的是 nginx 自建 async DNS resolver 而非 c-ares，故 c-ares 漏洞路徑不可觸；
#   binary 仍在映像內，一併補以清 Trivy HIGH。（2026-07-10）
# - curl/libcurl: CVE-2026-5773 (HIGH, SMB 錯誤檔案傳輸), CVE-2026-6276 (HIGH, redirect
#   時 cookie 洩漏) — 8.19.0-r0 → 8.20.0-r0。為 base 自帶（HEALTHCHECK 用 wget 非 curl、
#   CLI 實際未用），比照上方 nghttp2/c-ares 的做法 binary 一併補以清 Trivy HIGH。（2026-07-15）
# 註：pcre2 (10.47-r0) 與 openssl (libcrypto3/libssl3 3.5.6-r0) 已由 georgjung base
#     2026 重建涵蓋（CVE-2025-58050 / 6 個 openssl CVE 自動修復），無需額外升級。
# 其他安全更新透過升級基底映像版號處理，見 docs/security.md。
RUN apk update && apk upgrade zlib libexpat libpng musl musl-utils libcrypto3 libssl3 nghttp2 nghttp2-libs libxml2 c-ares curl libcurl && rm -rf /var/cache/apk/*

# 主配置：載入 Brotli 模組並 include conf.d
COPY nginx-main.conf /etc/nginx/nginx.conf
# Server block（含 brotli on / brotli_static 等）- 作為模板使用
# 放在 /etc/nginx/templates/ 而非 conf.d/，避免 conf.d 掛成 tmpfs 後模板被遮蔽
RUN mkdir -p /etc/nginx/templates
COPY nginx.conf /etc/nginx/templates/default.conf.template
# 共用安全標頭 snippet（HSTS/CSP/X-Frame-Options 等），由 nginx.conf 多處 include
COPY security-headers.conf /etc/nginx/snippets/security-headers.conf
# 容器啟動腳本（R27-1+R27-2：原 240+ 字 CMD 拆出，含 API_BACKEND_URL 非空驗證）
# 合併 chown + chmod 至 COPY 一步，省 1 層；明確指定 nginx 為 owner（Gemini PR #217）
COPY --chown=nginx:nginx --chmod=755 docker-entrypoint.sh /docker-entrypoint.sh

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

# 啟動流程交給獨立腳本（見 docker-entrypoint.sh），方便維護 + 支援 fail-fast
# 驗證 API_BACKEND_URL（R27-1+R27-2）。
CMD ["/docker-entrypoint.sh"]
