# R24 — Observability 補強與 IP-level Safety Gate（2026-04-19）

> **狀態**：草案，待審閱後納入 `docs/TODO.md` R24 章節
> **背景**：2026-04-18 盤點後確認 ipig_system 已有 80% 監控/稽核基礎（R22 已實作攻擊偵測 + 4 種通知管道）。本計畫補齊剩餘 4 個 gap，不建立獨立 dashboard 服務。
> **捨棄方案**：`C:\System Coding\ipig-dashboard\DASH_SPEC.md` v1.0（獨立 Node+Hono dashboard）已作廢，原因見 §決策紀錄

---

## 1. 盤點總結

### 1.1 已運作的基礎設施（無需重做）
| 類別 | 已有內容 |
|------|---------|
| Metrics | Prometheus builder（`main.rs:125`）+ `/metrics` route（`routes/mod.rs:97`）+ Grafana 儀表板（`deploy/grafana_dashboard.json`，10 panel）+ alert rules（146 行）|
| Logging | `tracing-subscriber` + JSON 格式支援 + Docker json-file driver（prod 50m×5）|
| Security Audit | `user_activity_logs`（分區表 2026-2029）/ `login_events` / `user_sessions` / `security_alerts` / integrity hash chain |
| Real IP 識別 | `middleware/real_ip.rs` — `extract_real_ip_with_trust` 處理 CF-Connecting-IP → X-Real-IP → X-Forwarded-For → socket fallback，含 SEC-30 trust_proxy 切換與 9 個測試 |
| 攻擊偵測 | R22 完整實作：rate limit 事件記錄 / 403 偵測 / IDOR probe alert / brute force / 蜜罐 / Auto-deactivate user |
| 通知管道 | R22-9~13：Email / LINE Notify / Webhook / 排程重送（6h 掃描 open alert） |
| 資料出口稽核 | `handlers/data_export.rs:63` 已寫入 `user_activity_logs` |
| WAF | Cloudflare Tunnel + CF Dashboard WAF（`deploy/cloudflared-config.yml`）|
| Rate Limit | 4-tier（auth 30/min、API 600/min、write 120/min、upload 30/min）|
| RBAC | users / roles / permissions / user_roles（100+ 權限碼）|

### 1.2 已知缺口（本計畫處理）
| # | 項目 | 優先級 | 備註 |
|---|------|--------|------|
| R24-1 | IP blocklist 表 + middleware + 自動封鎖 IP | P0 | 延伸 R22-6：從封鎖 user → 封鎖 IP |
| R24-2 | 生產環境啟用 Loki + Promtail | P1 | = R22-14 Phase 2；docs/r22-log-aggregation.md 已評估完 |
| R24-3 | Alertmanager 通知管道啟用（infra 層 metric alert） | P2 | 與 R22 security 通知互補（一個管 metric、一個管 security event）|
| R24-4 | Grafana 安全 Dashboard（LogQL + SQL panel） | P2 | = R22-15 ⏸️，依賴 R24-2 |

---

## 2. R24-1：IP blocklist + 自動封鎖（P0）

### 2.1 目標
將 R22-6 的「偵測到 IDOR probe → 停用 user」擴充為「同時封鎖來源 IP」，並支援手動維護黑名單。

### 2.2 資料表

```sql
-- migrations/031_ip_blocklist.sql
CREATE TABLE ip_blocklist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address INET NOT NULL,
    reason TEXT NOT NULL,
    source TEXT NOT NULL,             -- 'R22-6_idor' / 'R22-1_ratelimit' / 'manual' / 'honeypot'
    blocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    blocked_until TIMESTAMPTZ,        -- NULL = permanent
    blocked_by UUID REFERENCES users(id),  -- NULL = auto
    hit_count BIGINT NOT NULL DEFAULT 0,
    last_hit_at TIMESTAMPTZ,
    unblocked_at TIMESTAMPTZ,
    unblocked_by UUID REFERENCES users(id),
    unblocked_reason TEXT,
    metadata JSONB
);
CREATE UNIQUE INDEX idx_ip_blocklist_active
    ON ip_blocklist (ip_address) WHERE unblocked_at IS NULL;
CREATE INDEX idx_ip_blocklist_expiring
    ON ip_blocklist (blocked_until) WHERE blocked_until IS NOT NULL AND unblocked_at IS NULL;
```

### 2.3 Middleware 位置與順序
- 新檔：`backend/src/middleware/ip_blocklist.rs`
- **實際既有 stack**（`routes/mod.rs:44-60`）：`write_rate_limit → csrf → auth → security_response_logger → handlers`
- **插入後（定稿）**：`ip_blocklist（新，最外層）→ write_rate_limit → csrf → auth → security_response_logger → handlers`
  - 封 IP 作為新的最外層，既有 4 層順序不動（R22 已驗證可靠）
  - 黑名單請求在最外層即 short-circuit，不消耗後續 middleware CPU
- **掛載範圍**：
  - `auth_middleware_stack`（`routes/mod.rs:44`，正門 API）：加 ip_blocklist ✅
  - `upload_middleware_stack`（`routes/mod.rs:76`，上傳入口）：加 ip_blocklist ✅
  - `health_route`（`routes/mod.rs:95`，/metrics 與 /api/health）：**不加**（Prometheus scrape 等監控探針來自信任網段，避免誤封機房自身的監控儀器）
- 來源 IP 識別：呼叫 **`middleware::real_ip::extract_real_ip_with_trust`**（既有檔，見 §2.7）
- 邏輯：
  1. `extract_real_ip_with_trust(headers, &socket_addr, trust_proxy)` 取得來源 IP `String`
  2. `.parse::<IpAddr>()` 轉型供 cache key 與 DB INET 欄位
  3. 查 in-memory cache（30s TTL，DashMap）→ miss 再查 DB
  4. hit active entry → return `451 Unavailable For Legal Reasons` + 遞增 `hit_count` + 更新 `last_hit_at`
  5. miss → 放行
- Cache invalidation：當 `AuditService` 新增/更新 blocklist 時主動通知 cache 重載（pub/sub channel 或直接呼叫）

### 2.4 與既有邏輯整合

#### R22-6 IDOR probe → 封 IP 24h
- **精確位置**：`middleware/response_logger.rs` 的 `trigger_idor_probe_alert()` line 125-170
- line 154-156 已有 `auto_block_user(pool, user_id_str, alert_id)`（停用使用者）
- **動作**：在 line 156 後並排新增 `auto_block_ip(pool, ip_str, alert_id, Some(Duration::hours(24)))`
- **呼叫鏈修改**：`check_idor_probe()`（line 69）目前未收 IP，需從 `security_response_logger` middleware 解析 header 後沿途傳 IP 進來
- Source: `R22-6_idor`, `blocked_until` = NOW()+24h

#### R22-1 auth rate limit → 封 IP 1h
- 精確位置留待 R24-1 動工時讀 `rate_limiter.rs` 定位 R22-5 auth escalation 函式
- Source: `R22-1_ratelimit`, `blocked_until` = NOW()+1h

#### R22-16 honeypot → 永久封鎖
- **精確位置**：`handlers/honeypot.rs` line 52-133
- IP 已由 `extract_real_ip_with_trust(request.headers(), &addr, state.config.trust_proxy_headers)` 於 line 57-60 取得
- **動作**：在 line 118 之後（security_alerts 寫入後、`SecurityNotifier::dispatch` 之前）加 `auto_block_ip(pool, ip_str, alert_id, None)`
- Source: `honeypot`, `blocked_until` = NULL（永久）

### 2.5 管理介面
- **新路由**：`GET / POST / PATCH /admin/audit/ip-blocklist`（沿用既有 `/admin/audit/*` 慣例，對齊 `/admin/audit/alerts` 的 handler 模式）
- **前端建議**：作為 `AdminAuditPage` 新 Tab — `frontend/src/pages/admin/components/IpBlocklistTab.tsx`，與 `AuditActivitiesTab` / `AuditAlertsTab` 等並列
- 功能：列表（active/history 切換）、手動新增、延長/解除、批次匯出

### 2.6 驗收
- [ ] 連續觸發 IDOR probe → IP 自動進入 blocklist，後續 request 被 middleware 擋下（不進 handler）
- [ ] Cloudflare Tunnel forward 的 `CF-Connecting-IP` 正確被識別為來源 IP（非 tunnel 內 IP）
- [ ] 管理員可手動解除，後續 request 放行
- [ ] `hit_count` / `last_hit_at` 正確累積
- [ ] `blocked_until` 到期後自動不再阻擋（無需 cron 清理，middleware 查詢已排除過期）

### 2.7 來源 IP 識別：復用既有 `middleware/real_ip.rs`
- **不新建 util**（第一輪計畫的 `utils/request_ip.rs` 是盤點疏漏；既有實作完備，遵守 DRY）
- **既有檔**：`backend/src/middleware/real_ip.rs`
  - 函式：`extract_real_ip_with_trust(&HeaderMap, &SocketAddr, trust_proxy: bool) -> String`
  - 優先序：`CF-Connecting-IP` → `X-Real-IP` → `X-Forwarded-For`（首個）→ socket fallback
  - SEC-30：`trust_proxy` 參數已實作（避免偽造 header 繞過）
  - 測試：9 個 test case 覆蓋完整
- **既有呼叫端**（不動）：`middleware/rate_limiter.rs:144`、`handlers/honeypot.rs:57-60`
- **R24-1 新呼叫端**：
  - `middleware/ip_blocklist.rs`（新檔）
  - `middleware/response_logger.rs`（沿 call chain 把 IP 傳給 `check_idor_probe` / `trigger_idor_probe_alert`）
- **型別**：回傳 `String`，於呼叫端 `.parse::<IpAddr>()` 轉型供 INET 欄位與 cache key 使用

---

## 3. R24-2：生產環境啟用 Loki + Promtail（P1）

### 3.1 現況
- `docker-compose.logging.yml` 已定義 Loki + Promtail + Grafana datasource
- `docker-compose.prod.yml` **未引用** → 生產環境無集中式日誌
- 評估文件：`docs/r22-log-aggregation.md`（R22-14 [x] 完成）

### 3.2 動作
1. 於 `docker-compose.prod.yml` 新增 `include: docker-compose.logging.yml`（或複製 service 定義）
2. Promtail 設定調整：
   - 只收 `ipig-api`、`ipig-web` 兩個 container（排除 prometheus / alertmanager 自己的 log）
   - Labels：`container_name`, `environment=prod`, `log_level`
3. Loki 保留期：30 天
4. 磁碟預留：`/var/lib/loki` 至少 5GB，監控使用率

### 3.3 驗收
- [ ] `docker compose -f docker-compose.prod.yml up` 後 Grafana Explore 可查 `{container_name="ipig-api"}`
- [ ] JSON log（`RUST_LOG_FORMAT=json`）正確被 Loki 解析，可依 level / trace_id 篩選
- [ ] 30 天保留政策自動生效（超過自動刪除）

---

## 4. R24-3：Alertmanager 通知管道啟用（P2）

### 4.1 現況
- `monitoring/alertmanager/alertmanager.yml` 所有 receiver 的 webhook/email 都被註解
- 導致：Prometheus alert_rules.yml 的 146 行規則觸發後**無人收到**

### 4.2 職責劃分（關鍵：避免與 R22 通知重複）
| 通知來源 | 管道 | 管理邏輯 |
|---------|------|---------|
| **Prometheus metric alert**（CPU、延遲、錯誤率、DB pool）| **Alertmanager → Email/Webhook** | infra 層，本計畫處理 |
| **Security event alert**（IDOR probe、honeypot、brute force）| **SecurityNotifier → LINE/Email/Webhook**（R22-9~12）| 應用層，已完成 |

### 4.3 動作
1. 啟用 `default` receiver：webhook → `http://api:8000/api/webhooks/alertmanager`（新增此 endpoint 轉發至 LINE/Email，複用 R22 通知管道）
2. 啟用 `critical` receiver：email_configs，複用既有 SMTP secrets
3. 新增 `handlers/alertmanager_webhook.rs`（平檔，不開新 subdir）：接收 Alertmanager payload，轉譯後呼叫 `SecurityNotifier::dispatch()`
4. 設定去重與靜音：
   - group_wait: 30s
   - repeat_interval: critical=1h, warning=4h

### 4.4 驗收
- [ ] 人為觸發一筆 test alert（e.g. `curl POST /api/test/fake-500`）→ 3 秒內收到通知
- [ ] Silence 機制可用（Alertmanager UI）
- [ ] 不與 R22 security 通知重複（infra metric 走 Alertmanager，security event 走 SecurityNotifier）

---

## 5. R24-4：Grafana 安全 Dashboard（P2，依賴 R24-2）

### 5.1 依賴與前置作業
- Loki 已在 prod 啟用（R24-2）
- **Grafana datasources 現況**：`deploy/grafana/provisioning/datasources/prometheus.yml` 僅 Prometheus 一個，**需新增 2 個 datasource YAML**：
  - `deploy/grafana/provisioning/datasources/loki.yml`（url: `http://loki:3100`）— 供 R24-4 403 rate panel 與後續 LogQL query
  - `deploy/grafana/provisioning/datasources/postgres.yml`（`grafana_readonly` user 連 ipig DB）— 供 R24-4 的 SQL panel
- **需建 Postgres read-only 使用者**：新增 migration 或手動 SQL 建立 `grafana_readonly` role，GRANT SELECT 於 `security_alerts`、`login_events`、`user_activity_logs`、`ip_blocklist` 等稽核相關表（禁止 DML 避免 Grafana 誤改資料）

### 5.2 新增 Dashboard：`deploy/grafana_security_dashboard.json`
6 個 panel（沿用 ProductTable 風格，不放在既有 `grafana_dashboard.json`）：

| Panel | Query Source | Visualization |
|-------|-------------|---------------|
| Security Alerts 時間線（24h） | Postgres `security_alerts` | Time series, color by severity |
| Active IP Blocklist | Postgres `ip_blocklist WHERE unblocked_at IS NULL` | Stat + Table |
| Login Anomaly（24h） | Postgres `login_events WHERE is_anomalous_*` | Table |
| 403 Permission Denied rate（1h bucket） | Loki `{container="ipig-api"} \|= "PERMISSION_DENIED"` | Bar chart |
| Honeypot hits（7d） | Postgres `user_activity_logs WHERE event_type = 'HONEYPOT_HIT'` | Stat + Geo-map（若 GeoIP 可用）|
| Top 10 IPs by event | Postgres aggregate from `user_activity_logs` | Bar chart |

### 5.3 驗收
- [ ] 儀表板可於 Grafana 列表找到
- [ ] 6 個 panel 皆顯示資料
- [ ] 時間範圍篩選對所有 panel 生效
- [ ] Panel 可點擊展開至詳細 explore 查詢

---

## 6. 執行順序

| 階段 | 項目 | 依賴 | 預估工時 |
|------|------|------|---------|
| 1 | R24-1（IP blocklist） | 無 | 1 天（migration + middleware + 3 處整合 + Tab 前端；real_ip.rs 復用省時間但需傳 IP 沿 call chain） |
| 2 | R24-2（Loki prod） | 無 | 0.5 天（compose + 驗證）|
| 3 | R24-3（Alertmanager 通知） | R22 SecurityNotifier（已完成）| 0.5 天（webhook handler + config）|
| 4 | R24-4（Security dashboard） | R24-2 | 0.6 天（Loki + Postgres datasource YAML + `grafana_readonly` user + JSON dashboard）|

**總計：2.6 天**（單人，不含測試與部署驗證）

---

## 7. 決策紀錄

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-19 | 放棄獨立 dash 服務 | 盤點後發現 ipig_system 已有 80% 基礎設施；獨立服務 = 跨程序延遲（異常即封鎖需 inline）+ 多一組部署/認證/維運 |
| 2026-04-19 | 「異常即封鎖」在 middleware 內實作 | inline 封鎖延遲 < 1ms；跨服務 webhook 為秒級，攻擊者已發出數百 request |
| 2026-04-19 | Dashboard UX 沿用 Grafana，不自建 | Grafana 已運作，有成熟 panel / alert / query 能力 |
| 2026-04-19 | infra alert（Alertmanager）與 security alert（R22 SecurityNotifier）分開 | 職責清晰：Prometheus 看系統健康、R22 看應用攻擊 |
| 2026-04-19 | R24-1 封鎖以 IP 為主（而非 user） | R22-6 已處理 user 停用；IP 封鎖補齊「未登入攻擊者」的防護 |
| 2026-04-19 | 信任來源 IP 以 `CF-Connecting-IP`（Cloudflare）優先 | 生產走 CF Tunnel，直接取 `X-Forwarded-For` 最右可被偽造 |
| 2026-04-19 | 不加 R24-5 定期清理 cron | Partial index 已排除過期 entry；索引膨脹半年內可忽略；MVP 優先 |
| 2026-04-19 | Grafana security dashboard 獨立 `grafana_security_dashboard.json` | diff 易讀、可獨立停用；Grafana provisioning 支援多檔 |
| 2026-04-19 | Alertmanager webhook 用平檔 `handlers/alertmanager_webhook.rs` | 僅 1 endpoint，不開新 subdir；未來多於 3 個 webhook 再重構為 subdir |
| 2026-04-19 | Migration 編號 `031_ip_blocklist.sql`（接續 030）| 既有遞增慣例 |
| 2026-04-19 | IP 提取復用 `middleware/real_ip.rs`，不新建 `utils/request_ip.rs` | 既有實作涵蓋 CF/X-Real-IP/XFF/fallback + SEC-30 trust_proxy + 9 個測試；重造違反 DRY（盤點疏漏修正） |
| 2026-04-19 | 管理路由命名改為 `/admin/audit/ip-blocklist` | 沿用既有 `/admin/audit/*` 慣例，避免新開 `/admin/security/` namespace |
| 2026-04-19 | `/metrics` 與 `/api/health` 繼續 bypass ip_blocklist | 監控探針（Prometheus scrape）來自信任網段，不應被誤封 |
| 2026-04-19 | `ip_blocklist` 插為新的最外層，既有 4 層 stack 順序不動 | 最小侵入；既有 `write_rate_limit→csrf→auth→security_response_logger` 經 R22 驗證可靠 |

---

## 8. 審閱定稿（2026-04-19）

### 第一輪審閱（上午，6 項）

| # | 項目 | 結果 |
|---|------|------|
| 1 | R24-1 schema 符合既有慣例 | ✅ 已驗證：UUID + gen_random_uuid / INET / TIMESTAMPTZ / partial unique index / JSONB 皆有前例 |
| 2 | Middleware 掛載順序 | ✅（第二輪修訂，見下）|
| 3 | Webhook handler 位置 | ✅ 平檔 `handlers/alertmanager_webhook.rs`（不開 subdir） |
| 4 | Dashboard JSON 合併/分開 | ✅ 分開為 `grafana_security_dashboard.json` |
| 5 | R24-5 定期清理 cron | ✅ 不加，middleware 查詢自動排除過期 |
| 6 | 來源 IP 抽共用 util | ✅（第二輪修訂，見下）|

### 第二輪審閱（動工前交叉驗證，8 項修正）

用 Explore agent 把計畫對 codebase 的假設與實際檔案比對後，發現 8 處需修正：

| # | 原計畫 | 修正後 |
|---|--------|--------|
| F1 | middleware 順序描述錯誤 | 修為 `ip_blocklist（新最外層）→ write_rate_limit → csrf → auth → security_response_logger → handlers`；見 §2.3 |
| F2 | 新建 `utils/request_ip.rs` | 復用既有 `middleware/real_ip.rs`（含 9 個測試、SEC-30 trust_proxy）；見 §2.7 |
| F3 | R22-6 整合點模糊 | 精確指向 `response_logger.rs:154-156`，在 `auto_block_user` 後並排新增 `auto_block_ip`；IP 需沿 call chain 傳入；見 §2.4 |
| F4 | R22-1 整合點模糊 | 動工時再精確定位 rate_limiter.rs 中 R22-5 auth escalation 函式 |
| F5 | 路由 `/admin/security/blocklist` | 改為 `/admin/audit/ip-blocklist` 以符合既有慣例；見 §2.5 |
| F6 | 獨立 `IpBlocklistPage.tsx` | 建議整合為 `AdminAuditPage` 新 Tab；見 §2.5 |
| F7 | Grafana datasources「需確認」 | 明列需新增 `loki.yml` + `postgres.yml` + 建 `grafana_readonly` user；見 §5.1 |
| F8 | 總工時 2.5 天 | 微調為 2.6 天；見 §6 |

計畫經兩輪審閱後可動工。
