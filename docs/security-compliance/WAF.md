# Web Application Firewall (WAF) 部署指南

> **版本：** v1.0  
> **基於：** OWASP ModSecurity Core Rule Set (CRS) v4  
> **映像：** `owasp/modsecurity-crs:4-nginx-alpine`

## 1. 架構概覽

```
Client → WAF (ModSecurity + Nginx) → web (Frontend Nginx) → api (Rust Backend)
         ↓ (:8443)                     ↓ (:80)                ↓ (:8000)
```

WAF 作為最外層反向代理，所有 HTTP 流量先經過 ModSecurity 規則引擎檢查，
通過後才轉發到後端的 `web` 服務。

## 2. 啟用方式

### 偵測模式（僅記錄，不阻擋）

初次部署建議先以偵測模式運行，觀察日誌確認無誤報後再切換：

```bash
docker compose -f docker-compose.yml -f docker-compose.waf.yml up -d
```

WAF 預設監聽 `localhost:8443`。

### 切換為阻擋模式

確認偵測模式運行穩定後，修改 `docker-compose.waf.yml` 中的環境變數：

```yaml
MODSEC_RULE_ENGINE: "On"   # 改為 On 即啟用阻擋
```

然後重啟：

```bash
docker compose -f docker-compose.yml -f docker-compose.waf.yml up -d waf
```

## 3. 保護範圍

OWASP CRS 提供以下防護：

| 類別 | 規則 ID 範圍 | 說明 |
|------|-------------|------|
| SQL Injection | 942xxx | 防止 SQL 注入攻擊 |
| XSS | 941xxx | 防止跨站腳本攻擊 |
| RCE | 932xxx | 防止遠端命令執行 |
| LFI/RFI | 930xxx/931xxx | 防止本地/遠端檔案包含 |
| Scanner Detection | 913xxx | 偵測自動化掃描工具 |
| Protocol Attack | 921xxx | HTTP 協定違規偵測 |
| Session Fixation | 943xxx | Session 固定攻擊防護 |
| Data Leakage | 950xxx/951xxx | 防止敏感資料外洩 |

## 4. 自訂排除規則

排除規則位於 `deploy/waf/` 目錄：

- `REQUEST-900-EXCLUSION-RULES-BEFORE-CRS.conf` — CRS 規則載入前的排除
- `RESPONSE-999-EXCLUSION-RULES-AFTER-CRS.conf` — CRS 規則載入後的排除

### 已配置的排除

| ID | 說明 |
|----|------|
| 1000 | 允許 JSON Content-Type |
| 1001 | 密碼欄位排除 SQL injection 檢查 |
| 1002 | TOTP 驗證碼排除誤報 |
| 1003 | 計畫書/觀察紀錄富文本排除 XSS 檢查 |
| 1004 | 檔案上傳端點放寬限制 |

## 5. 日誌與監控

查看 WAF 日誌：

```bash
docker compose -f docker-compose.yml -f docker-compose.waf.yml logs -f waf
```

ModSecurity 日誌會記錄所有觸發的規則，格式包含：
- 觸發的規則 ID 和訊息
- 請求來源 IP
- 請求 URI 和 Method
- Anomaly Score

### 日誌分析

搜尋被阻擋的請求：
```bash
docker logs ipig-waf 2>&1 | grep "ModSecurity: Access denied"
```

搜尋特定規則觸發：
```bash
docker logs ipig-waf 2>&1 | grep "id \"942"  # SQL injection 相關
```

## 6. Paranoia Level 調整

| Level | 說明 | 適用場景 |
|-------|------|---------|
| 1 | 基本保護，最少誤報 | **建議初始值** |
| 2 | 增加規則，可能有少量誤報 | 穩定運行後升級 |
| 3 | 嚴格檢查，需要更多排除規則 | 高安全需求 |
| 4 | 極嚴格，大量誤報 | 不建議用於生產 |

修改 `docker-compose.waf.yml` 中的 `PARANOIA` 環境變數即可調整。

## 7. 生產部署注意事項

1. **先偵測後阻擋**：務必先用 `DetectionOnly` 模式觀察至少 1-2 週
2. **備份排除規則**：排除規則是重要配置，應納入版本控制
3. **定期更新 CRS**：更新 WAF 映像以獲取最新規則
4. **資源限制**：已設定 CPU 1 核 / 記憶體 512MB 上限
5. **SSL/TLS**：生產環境建議在 WAF 前方使用 Cloudflare Tunnel 或 Let's Encrypt
