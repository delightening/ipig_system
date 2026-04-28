# 系統審查報告 — 併發 / GLP / ISO 27001

- **日期**：2026-04-25
- **審查對象**：ipig_system（實驗動物管理系統，Rust/Axum + React）
- **審查範圍**：併發安全、21 CFR Part 11 / OECD GLP、ISO 27001 Annex A
- **審查方式**：3 個並行 sub-agent 對 backend / frontend / migrations / 設定檔做靜態掃描，本檔為彙整與校準後成果
- **狀態**：靜態審查（無動態測試、無滲透測試）

---

## 0. 摘要與校準

| 面向 | Critical | High | Medium | Low |
|------|---------|------|--------|-----|
| 併發 | 0 | 2 | 3 | 2 |
| GLP / 21 CFR Part 11 | 3 | 4 | 2 | 3 |
| ISO 27001 Annex A | **0**（原 1，已校準） | **3**（原 2，併入 1 項） | 2 | 3 |
| **彙整去重後** | **3** | **8** | **6** | **6** |

### 已校準的誤判
- **ISO-C1（.env 進版控）誤判**：經 `git ls-files .env` / `git log --all -- .env` 驗證，`.env` 從未進入 git history，`.gitignore:18` 已忽略。**降為 Low（本機明文密碼建議改用 secrets manager）**。

### 跨面向交叉引用（同一根因，多面向影響）
| 議題 | 併發 | GLP | ISO |
|------|------|-----|-----|
| Audit chain HMAC verify cron 未排程 | H2（多 instance 重複） | H1（無告警） | A.8.15/A.8.16 |
| Audit log 與業務寫入 transaction 邊界 | （良好） | C1/C2（鎖定缺口） | A.8.15 |
| Permission cache stampede | H1 | — | A.5.18 |
| Impersonation 簽章追蹤 | — | M2 | A.8.2 |

---

## 1. Critical（合規/安全阻斷，須立即修補）

### [C1] 已簽章動物觀察記錄無 UPDATE 鎖
- **規範**：21 CFR §11.10(e)(1)
- **位置**：`backend/src/handlers/animal/observation.rs:181-195`、`migrations/006`
- **缺口**：`animal_observations` schema 缺 `is_locked / locked_at / locked_by`。`SignatureService::lock_record()` 寫入後，update handler 沒檢查 lock 旗標 → 簽章後仍可改記錄
- **建議**：
  1. 新增 migration 加 `is_locked / locked_at / locked_by` 三欄（observation / surgery / medication / blood_test）
  2. 所有受規範記錄的 update handler 開頭 `require !is_locked`，否則回 409
  3. 簽章成功後同 transaction 自動鎖定

### [C2] Amendment 核准後仍可竄改
- **規範**：21 CFR §11.10(e)(1)
- **位置**：`backend/src/handlers/amendment.rs`、`backend/src/services/amendment/workflow.rs`
- **缺口**：`amendments` 表無簽章 FK；`decide()` 將狀態改為 APPROVED 後，UPDATE handler 仍允許編輯
- **建議**：
  1. Migration：`amendments` 加 `approved_signature_id UUID REFERENCES electronic_signatures(id)`
  2. `decide()` 在 APPROVED/REJECTED 時於同一 tx 建立 electronic_signature
  3. UPDATE handler 檢查 `approved_signature_id IS NOT NULL` → 拒絕

### [C3] 密碼變更 / 2FA 切換無雙因素確認
- **規範**：21 CFR §11.200(a)、ISO A.5.16
- **位置**：`backend/src/services/auth/password.rs`、`backend/src/handlers/auth/two_factor.rs`
- **缺口**：
  - 密碼變更只驗舊密碼，無短期 reauth token；無 `password_confirmation`
  - 2FA 啟用/停用只用當前 JWT，不需重新輸入密碼或 OTP
- **風險**：XSS / session hijack 後，攻擊者可無痛改密碼並關閉 2FA，無立即可見痕跡
- **建議**：
  1. 密碼變更要求 `password_confirmation` 欄位 + 後端 match 驗證
  2. 2FA 切換要求重新輸入密碼（並走密碼簽章 flow）
  3. 所有 credential 變更寫 `SecurityEvent::CREDENTIAL_CHANGED`，含 IP / UA

---

## 2. High（高風險，1–2 sprint 內修補）

### [H1] Audit chain HMAC 驗證 cron 未排程或無告警（合規 + 併發）
- **規範**：21 CFR §11.10(e)、ISO A.8.15
- **位置**：`backend/src/services/audit.rs`（`ChainVerificationReport` 已實作）；`backend/src/services/scheduler.rs:467-497`（`register_audit_chain_verify_job`）
- **缺口**：
  - GLP 角度：cron 排程細節未對接 `AuditService::verify_chain_range()`，斷鏈無 alert
  - 併發角度：多 instance 部署時無 distributed lock（pg_advisory_lock 缺）
- **建議**：
  1. cron 進場先 `SELECT pg_try_advisory_lock(<job_id>)`，搶不到就跳過
  2. 失敗（broken_links 非空）觸發 `SecurityNotifier` 發 CRITICAL 告警
  3. Admin dashboard 暴露最新驗證報告

### [H2] Permission cache stampede（auth 中介層）
- **位置**：`backend/src/middleware/auth.rs:135-212`
- **缺口**：cache miss 時多執行緒同時跑 4-table JOIN；DashMap insert 不會錯，但浪費 DB 資源
- **建議**：以 `tokio::sync::OnceCell` per-key 或 `moka` async cache 做 single-flight；或 5 min TTL 改 jittered 避免同時過期

### [H3] File upload 與 metadata 寫入不在同一 transaction
- **位置**：`backend/src/services/file.rs:264-352`
- **缺口**：磁碟寫入完成 → 回傳 metadata → handler 才寫 DB。DB 失敗即孤立檔案
- **建議**：
  1. 短期：handler 在同一 service call 內接收 `&mut Transaction`；DB 失敗時 unlink
  2. 長期：背景 cron 掃孤立檔案（無 metadata 對應）

### [H4] Login 流程：token 已發出但 session 建立失敗
- **位置**：`backend/src/handlers/auth/login.rs:92-105`
- **缺口**：`issue_login_tokens()` 已返簽好的 JWT，之後才 `create_session()` / `end_excess_sessions()`。後者失敗，client 拿到有效 JWT 卻無 session 記錄
- **建議**：先 create_session + 檢查上限通過，再 issue token；或 token 中埋 `session_id`，驗證時 JOIN session

### [H5] Audit entity_display_name 不規範化，事後追蹤困難
- **規範**：21 CFR §11.10(e)(1)(i)
- **位置**：`backend/src/services/animal/observation.rs`
- **缺口**：observation audit 記 entity_id=UUID 但無 ear_tag / IACUC_NO context。FDA 查詢「誰改了動物 X 的記錄」需手 JOIN
- **建議**：
  1. 統一 entity_display_name 格式：`"{ear_tag} - {event_type} on {date}"`
  2. 提供 view：`audit_entity_timeline` JOIN animal/protocol metadata
  3. Audit export 加 `expand_entity=true` 自動展開

### [H6] Amendment decide() 無權限驗證
- **規範**：21 CFR §11.70、ISO A.5.18
- **位置**：`backend/src/services/amendment/workflow.rs`
- **缺口**：`decide()` 接收 `decided_by: Uuid` 但不驗該 user 是否有 amendment.approve 權限
- **建議**：
  1. Migration `016` 加 `amendment.approve` permission，綁 TEST_FACILITY_MANAGEMENT 角色
  2. handler 加 `require_permission!(user, "amendment.approve")`

### [H7] JWT EC 私鑰檔權限未強制
- **規範**：ISO A.8.24
- **位置**：`./secrets/jwt_ec_private_key.pem`
- **建議**：
  1. Startup 檢查檔案權限 ≤ 600，否則拒啟（startup-time validation）
  2. 中期遷移到 KMS / HSM
  3. Container 內 mount as read-only secret

### [H8] 帳號鎖定事件 audit 寫入用 `tokio::spawn` 異步
- **規範**：21 CFR §11.10(e)、ISO A.8.15
- **位置**：`backend/src/services/auth/login.rs:57-78`
- **缺口**：`SEC_EVENT_ACCOUNT_LOCKOUT` 用 `tokio::spawn` fire-and-forget，crash 即遺失
- **建議**：改用 `insert_failure_event_in_tx` 同 transaction（已具備該函式）

---

## 3. Medium（中期改善）

### [M1] JWT blacklist DB-fallback 寫快取存在 race window
- **位置**：`backend/src/middleware/jwt_blacklist.rs:110-139`
- **建議**：以 `pg_advisory_lock` 序列化、或加版本號比對

### [M2] 簽章密碼參數未 zeroize
- **規範**：21 CFR §11.200、ISO A.8.24
- **位置**：`backend/src/services/signature/mod.rs:484-526`
- **建議**：引入 `zeroize` crate，密碼包成 `SecretString`

### [M3] Impersonation 期間簽章無雙因素 + 紀錄缺 context
- **規範**：21 CFR §11.200、ISO A.8.2
- **位置**：`backend/src/services/signature/mod.rs`、`backend/src/middleware/auth.rs:25-26`
- **缺口**：JWT claim 有 `impersonated_by`，但 signature record 無此欄位；admin 模擬期間簽署無額外驗證
- **建議**：
  1. `electronic_signatures` 加 `impersonation_context JSONB`
  2. 模擬期間簽章要求原 admin 二次認證（OTP）

### [M4] 軟刪除欄位不一致
- **位置**：`migrations/006`、各 service
- **缺口**：`animal_observations` / `care_medication_records` 有 deleted_at；`record_annotations` 無
- **建議**：統一三欄模式（`deleted_at / deleted_by / deletion_reason`）+ `all_deleted_records` view

### [M5] Admin 操作無特殊 audit 標記
- **規範**：ISO A.8.2
- **位置**：`backend/src/services/audit.rs`
- **建議**：`event_category="ADMIN"` 自動由 `actor.is_admin()` 標記

### [M6] CORS / SEED_DEV_USERS 缺 CI guard
- **規範**：ISO A.8.32 / A.8.34
- **建議**：CI 對 main branch 檢查 `CORS_ALLOWED_ORIGINS` 不含 localhost、`SEED_DEV_USERS=false`

---

## 4. Low（觀察 / 持續改善）

### [L1] 本機 `.env` 含明文密碼（**修正自原 ISO-C1**）
- **校準**：`.env` 不在 git 歷史中，已被 `.gitignore:18` 忽略。Git 洩漏風險不存在
- **保留問題**：本機開發者磁碟上仍是明文，多人協作場景不佳
- **建議**：
  1. 短期：`.env` 改用 `direnv` + `pass` / `1Password CLI` 動態注入
  2. 長期：dev/staging/prod 全部走 secrets manager（Doppler / AWS SM / Vault）

### [L2] 簽章手寫 SVG 無真實性驗證
- **位置**：`backend/src/services/signature/mod.rs:121-150`
- **建議**：手寫僅作視覺輔助，法律效力以密碼簽章為準

### [L3] Audit log export 無 max_rows
- **位置**：`backend/src/handlers/audit.rs:50-67`
- **建議**：export 限 100k rows + rate limit

### [L4] 時區依 Rust `Utc::now()`，非 DB `NOW()`
- **規範**：21 CFR §11.10(k)
- **建議**：audit 時間戳改由 DB server 產生

### [L5] CSRF `constant_time_eq` 函式實作未驗證
- **位置**：`backend/src/middleware/csrf.rs:94`
- **建議**：替換為 `subtle::ConstantTimeEq` 或 `hmac::Mac::verify` 確保 constant-time

### [L6] Rate limiter cleanup 同步遍歷
- **位置**：`backend/src/middleware/rate_limiter.rs:30-55`
- **建議**：清理移背景 task / TTL-based

---

## 5. 已實作良好的部分

### 併發
- `AuditService::log_activity_tx` 用 `&mut Transaction`，audit 與業務同 tx
- HMAC chain advisory lock 序列化（audit.rs:429-432）
- INFRA-4 `CancellationToken` 已接所有 cron job，graceful shutdown 完整
- 檔案 magic number + ZIP path traversal 防禦完整

### GLP
- 電子簽章核心：含 signer / time / type，密碼驗證
- HMAC chain v2 canonical 編碼（防字串串接衝突）
- 分區表設計（quarterly partitions 2026–2029）
- record_versions 快照保留

### ISO 27001
- JWT ES256 非對稱簽章（避免 HS256 對稱密鑰風險）
- Argon2 密碼雜湊（m=19456, t=2, p=1）+ 強度驗證
- 帳號鎖定 TOCTOU-safe（`pg_advisory_xact_lock`）
- CSRF signed double submit（HMAC-SHA256）
- Security headers 完整（CSP / HSTS / X-Frame / X-Content-Type）
- sqlx 全 parameterized query，無字串拼接 SQL
- Timing side-channel 防禦（不存在帳號做虛擬 Argon2）
- 多層 rate limit（auth / API / upload / forgot-pw）

---

## 6. 建議修補順序（roadmap）

| 優先 | 期程 | 項目 | 對應發現 |
|------|------|------|---------|
| P0 | 本週 | 受規範記錄加 `is_locked` 欄位與 update guard | C1 |
| P0 | 本週 | Amendment 核准簽章 FK + UPDATE 阻擋 | C2 |
| P0 | 本週 | 密碼/2FA 變更要求二次認證 | C3 |
| P1 | 本月 | Audit chain verify cron + advisory lock + alert | H1 |
| P1 | 本月 | Login 順序修正（session 先於 token） | H4 |
| P1 | 本月 | File upload 同 tx | H3 |
| P1 | 本月 | Amendment decide 權限驗證 | H6 |
| P2 | 1–2 月 | Permission cache single-flight | H2 |
| P2 | 1–2 月 | Audit entity_display_name 規範化 + view | H5 |
| P2 | 1–2 月 | JWT key 權限檢查 / 遷 KMS | H7 |
| P2 | 1–2 月 | 帳號鎖定 audit 改同步 | H8 |
| P3 | 持續 | M1–M6（zeroize / impersonation context / 軟刪統一 / admin tag / CI guard） | M1–M6 |
| P4 | 觀察 | L1–L6（secrets manager / 手寫簽 / export limit / DB time / constant-time / rate limit cleanup） | L1–L6 |

---

## 7. 審查方法限制

- **靜態審查**：未跑動態測試、未做滲透、未驗 race 是否實際觸發
- **未涵蓋**：infra 安全（K8s RBAC / 網路分段 / WAF）、人員與流程控制（A.5/A.6/A.7）、災難復原演練實況（A.8.13）、第三方依賴 CVE 掃描（建議跑 `cargo audit` / `npm audit`）
- **建議補充**：
  1. 跑 `cargo audit` + `npm audit` 出依賴 CVE 報告
  2. PR #199（R26 Epic Closure）做專項滲透測試
  3. 安排第三方 GLP / ISO 27001 pre-audit

---

## 8. 對應文件（R30-I 補完）

本審查報告所列缺口的 SOP / runbook / traceability 補完文件（2026-04-28 R30-I 新增）：

- [`../glp/traceability-matrix.md`](../glp/traceability-matrix.md) — GLP / 21 CFR Part 11 雙向追溯表（§1 各 Critical / High 缺口的條款對應）
- [`../glp/amendment-sop.md`](../glp/amendment-sop.md) — Amendment 狀態機 SOP（對應 §1 [C2]）
- [`../glp/record-lock-rationale.md`](../glp/record-lock-rationale.md) — GLP record lock 5 表選擇理由（對應 §1 [C1]）
- [`../runbooks/audit-chain-broken-runbook.md`](../runbooks/audit-chain-broken-runbook.md) — Audit chain 斷鏈處理 runbook（對應 §2 [H1]）
- [`../runbooks/dr-drill-records.md`](../runbooks/dr-drill-records.md) — DR drill 年度演練紀錄（對應 R26-3 / SOC 2 A1.2）
- [`../glp/training-records-sop.md`](../glp/training-records-sop.md) — Training records SOP 對照（對應 §11.10(i)）
