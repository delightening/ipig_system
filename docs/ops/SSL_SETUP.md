# SSL/TLS 設定指南

> iPig 系統目前透過 **Cloudflare Tunnel** 提供 HTTPS，無需自行管理憑證。  
> 本文件提供直接部署 Nginx + Let's Encrypt 的方案，供未使用 Cloudflare 的環境參考。

---

## 方案一：Cloudflare Tunnel（推薦，目前使用中）

無需任何 SSL 設定，Cloudflare 自動處理 TLS 終止。

## 方案二：Let's Encrypt + Certbot

### 1. 安裝 Certbot

```bash
apt-get update && apt-get install -y certbot
```

### 2. 申請憑證

```bash
certbot certonly --standalone -d your-domain.com --agree-tos -m admin@your-domain.com
```

### 3. 使用 SSL Nginx 配置

將 `frontend/nginx-ssl.conf.example` 複製為 `nginx-ssl.conf` 並修改域名。

### 4. 自動續期

```bash
# 加入 crontab
0 3 * * * certbot renew --quiet --post-hook "docker exec ipig-web nginx -s reload"
```

### 5. Docker Compose 掛載憑證

```yaml
services:
  web:
    volumes:
      - /etc/letsencrypt:/etc/letsencrypt:ro
      - ./frontend/nginx-ssl.conf:/etc/nginx/conf.d/default.conf:ro
    ports:
      - "443:8443"
      - "80:8080"
```

---

## TLS 最佳實踐

- 僅啟用 TLSv1.2 / TLSv1.3
- 啟用 OCSP Stapling
- 設定 HSTS header（已在 nginx.conf 中配置）
- 使用強加密套件（見 nginx-ssl.conf.example）
