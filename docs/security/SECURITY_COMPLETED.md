# 已完成資安項目清單

> 產出日期：2026-04-20
> 來源：`docs/TODO.md` + `docs/PROGRESS.md` §9 + git log
> 合計：**46 項** 橫跨認證授權、密碼會話、SQL/XSS/CSRF、Rate Limit、稽核告警、CI/Secrets、合規七大面向

| 項目 | 功能性 | 日期 | 如何完成 |
|------|--------|------|----------|
| P0-2 SQL 字串拼接修復 | 防 SQL Injection | 2026-03-20 | `core.rs:139` 改參數化查詢；`data_import.rs` 表名/欄名白名單 + `debug_assert` |
| R7-1 SQL 拼接修復 | 防 SQL Injection | 2026-03-08 | `data_import.rs` `format!()` 改參數化查詢 |
| R16-22 動態 SQL 修復 | 防 SQL Injection | 2026-03-28 | `services/stock.rs` 4 處改用 `sqlx::QueryBuilder` |
| R7-2 密碼洩露修復 | 憑證外洩防護 | 2026-03-08 | `create_admin.rs` 不再將密碼明文印至 stdout |
| R9-4 歡迎信安全改善 | 憑證外洩防護 | 2026-03-15 | 改用密碼重設連結取代 email 寄明文密碼 |
| R10-L7 密碼複雜度 | 密碼強度 | 2026-03-21 | ≥10 字元 + 大小寫 + 數字 + 黑名單 + 強度指示器 |
| P3-1 SEC-33 敏感操作二級認證 | 授權強化 | 2026-02-25 | confirm-password + reauth token，刪除/重設/模擬登入/刪角色皆需重輸密碼 |
| P5-3 SEC-39 TOTP 2FA | 多因子認證 | 2026-02-28 | 後端 totp-rs + 4 API + 備用碼；前端 QR Code + Profile 管理 |
| R9-1 IDOR 漏洞修復 | 物件授權 | 2026-03-15 | `download_attachment` / `list_attachments` 加 entity_type 權限檢查 |
| P1-32 Session 逾時預警 | 會話管理 | — | `SessionTimeoutWarning` 到期前 60s 倒數 Dialog |
| R7-5 Auth Rate Limit 降低 | 暴力破解防護 | 2026-03-08 | 認證端點 100→30/min |
| R10-M5 CSRF 強化 | CSRF 防護 | 2026-03-21 | Signed Double Submit Cookie + 8 測試 |
| R16-11 CSRF env var guard | CSRF 防護 | 2026-03-28 | `DISABLE_CSRF_FOR_TESTS` 加 production guard |
| R12-7 CSRF Token 客戶端刷新 | CSRF 防護 | 2026-03-25 | 403 偵測 → GET `/auth/me` 刷新 cookie → 自動重試 |
| R16-3 稽核日誌 PDF XSS | 防 XSS | 2026-03-28 | `useAuditLogExport.ts` 加入 `escapeHtml()` |
| R16-12 HSTS header | 傳輸安全 | 2026-03-28 | `startup/server.rs` 加 `Strict-Transport-Security`，gate on `cookie_secure` |
| P5-4 SEC-40 WAF | 應用層防火牆 | 2026-02-28 | 改由 Cloudflare WAF 處理（移除 ModSecurity overlay） |
| R9-3 DB 錯誤碼修正 | 錯誤資訊洩漏 | 2026-03-15 | 23505→409 / 23503/23502/23514→400 |
| P1-M0 稽核日誌匯出 API | 稽核合規 | 2026-03-01 | `GET /admin/audit-logs/export?format=csv\|json`，權限 `audit.logs.export` |
| P2-M4 稽核 UI 使用者篩選 | 稽核可用性 | 2026-03-01 | AuditLogsPage 新增操作者篩選 |
| R22-1 Rate limit 事件寫 DB | 攻擊偵測 | 2026-04-14 | `middleware/rate_limiter.rs` 觸發時呼叫 `AuditService::log_security_event()`，4 tier 全覆蓋 |
| R22-2 AI key rate limit 記錄 | 攻擊偵測 | 2026-04-14 | `ai_auth.rs` deactivated/expired/rate_limited 三事件寫 DB |
| R22-5 Auth rate limit 升級告警 | 主動告警 | 2026-04-14 | 同 IP 超閾值 → critical alert + 去重 + 主動通知 |
| R22-8 告警閾值設定化 | 可維運性 | 2026-04-14 | `security_alert_config` 表 + `AlertThresholdService` 60s cache |
| R22-9 通知管道抽象層 | 告警整合 | 2026-04-14 | `SecurityNotifier::dispatch()` 讀取 `security_notification_channels` |
| R22-17 Admin 安全事件 Tab | 稽核 UI | 2026-04-14 | 11 種 event_type 篩選，`SecurityEventsTab` 元件 |
| R24-3 Alertmanager infra 通知 | 基礎設施告警 | 2026-04-18 | webhook → `/api/webhooks/alertmanager`（Bearer token）→ R22 `SecurityNotifier::dispatch()` |
| R24-4 Grafana security dashboard | 安全可視化 | 2026-04-18 | 6 panel + Loki/Postgres datasource + `grafana_readonly` role |
| R17-1 CI 日誌檔含測試密碼 | 憑證外洩防護 | 2026-03-29 | `git filter-repo` 移除歷史 + `logs_*/` 加入 `.gitignore` |
| R16-13 CI 硬編碼 fallback 密碼 | 憑證外洩防護 | 2026-03-28 | `ci.yml` 移除 fallback，secrets 必填 |
| R9-C2 CI 密碼改 GitHub Secrets | 憑證管理 | 2026-03-23 | `JWT_SECRET` / `DEV_USER_PASSWORD` / `ADMIN_INITIAL_PASSWORD` 改 GitHub Secrets 參照 |
| R10-M10 Prometheus/Grafana 認證 | 監控面板授權 | 2026-03-21 | 環境變數密碼 + 本機綁定確認 |
| R25-1 Trivy 容器掃描 | 供應鏈安全 | 2026-04-20 | `.github/workflows/ci.yml:382-432` 每次 build 掃 OS/base image CVE，critical/high 自動 fail |
| R25-2 security.txt (RFC 9116) | 漏洞回報 | 2026-04-20 | `/.well-known/security.txt` 提供聯絡管道 |
| R25-3 CSP report-uri 端點 | XSS 偵測 | 2026-04-20 | `POST /api/v1/csp-report` 收集真實 XSS 嘗試 → `security_alerts` |
| R25-4 Secret scanning in CI | 供應鏈安全 | 2026-04-20 | gitleaks-action 掃 commit，防 API key / token 進版控 |
| R25 (E-2) Gotenberg timeout | 服務韌性 | 2026-04-20 | commit `3849f68` |
| R25 (E-3) GPG startup validation | 啟動完整性 | 2026-04-20 | commit `3849f68` |
| R25 (E-5) 忘記密碼 rate limit | 暴力破解防護 | 2026-04-20 | commit `3849f68` |
| R25 DB statement_timeout | DoS 防護 | 2026-04-20 | commit `cc6e536` — statement timeout 阻斷長查詢 |
| R18-1 每日分段 Code Review | 持續資安審視 | 2026-04 | Claude `/schedule` 10 天一輪迴模組 review |
| R18-2 每日健康檢查 | 持續資安審視 | 2026-04 | cargo test / clippy / npm audit / cargo deny / build / E2E |
| R19-14 邀請制安全測試 | 授權鏈路 | — | Token 暴力破解 / 過期 / 重複使用防護測試 |
| P4-1 基礎映像 CVE 週期檢查 | 供應鏈安全 | 2026-02-28 | nginx-brotli 釘選 `1.29.5-alpine`，Q2 下次檢查 |
| P1-M4 憑證輪換文件 | 營運合規 | 2026-03-01 | `docs/security-compliance/CREDENTIAL_ROTATION.md` |
| P1-7 電子簽章合規審查 | 21 CFR Part 11 | 2026-02-25 | `ELECTRONIC_SIGNATURE_COMPLIANCE.md` |
| P2-M5 SOC2 Readiness | 合規 | 2026-03-01 | Trust Services Criteria 對照文件 |
| P1-M2 GDPR 資料主體權利 | 隱私合規 | 2026-03-01 | `GET /me/export` + `DELETE /me/account` |
| R17-3 CSP unsafe-inline/eval | 已接受風險 | 2026-03-29 | Cloudflare + Vite 需求，以 DOMPurify 為補償控制 |

---

## 七大面向分布

| 面向 | 項目數 |
|------|--------|
| 認證授權（2FA、二級認證、IDOR、Session） | 5 |
| 密碼與憑證（複雜度、外洩防護、輪換） | 6 |
| SQL / XSS / CSRF / HSTS / WAF | 9 |
| Rate Limit 與攻擊偵測告警（R22 / R24） | 9 |
| 稽核日誌（匯出、篩選、PDF 防 XSS、UI） | 4 |
| CI / Secrets / 供應鏈（Trivy、gitleaks、GH Secrets） | 6 |
| 合規（21 CFR Part 11、SOC2、GDPR） | 4 |
| 持續維運 / 已接受風險 | 3 |
