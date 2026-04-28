# Code Review Findings — 2026-04-28

> 三軸平行審查：**多人協作併發協調 / 操作日誌記錄與顯示 / GLP 合規性**。
> 來源：Claude 三個 sub-agent 平行掃描全 codebase（backend Rust/Axum + frontend React + migrations + docs）。
> 後續行動清單見本文末「R30 計畫」與 `docs/TODO.md` R30 章節。

---

## 摘要

| 軸 | 等級 | 最關鍵發現 |
|----|------|-----------|
| 軸一 併發協調 | 🟡 PASS w/ 重大缺口 | `euthanasia` 全模組無 tx / 無 FOR UPDATE；protocol body 無 version 防 lost update |
| 軸二 操作日誌 | 🟢 整體成熟 | R26 pattern 已普及；但 `euthanasia.create_order` / `protocol CRUD` / `accounting` / `sudden_death` / `import_export` 漏寫 audit |
| 軸三 GLP 合規 | 🔴 不可上線稽核 | 簽章單因素 + 非 HMAC + 不在 chain；IDXF 漏 19 表；soft-delete / retention 未強制 |

**三軸交叉的最薄弱點**：`euthanasia` 模組（不可逆動物處置決策路徑）三軸全敗 → R30 第一順位。

---

## 軸一：多人協作併發協調

### 評等分布

| 模組 | 評等 | 證據 |
|------|------|------|
| animals | ★★★★★ 範本 | `services/animal/core/update.rs:42-165` FOR UPDATE + `version` 樂觀鎖 + audit-in-tx；衝突回 409「此記錄已被其他人修改」 |
| amendment | ★★★★ | `services/amendment/workflow.rs:514-577` 終態守衛 + `apply_terminal_decision_tx` 簽章同 tx |
| stock / warehouse | ★★★★ | `services/storage_location.rs:632-650` atomic decrement `WHERE on_hand_qty >= $1` + `rows_affected == 0` 偵測 |
| accounting | ★★★★ | `services/accounting.rs::post_document` 接收 `Transaction`，由 `document/workflow.rs:44` 鎖定 document 後同 tx 過帳 |
| hr (leave/overtime/attendance) | ★★★★ | `leave.rs` 9 處 FOR UPDATE、`balance.rs:264` 餘額 FOR UPDATE |
| protocol | ★★★ | 狀態機 `change_status_tx` FOR UPDATE OK，但 body 欄位 PUT **無 version 防 lost update** |
| notification | ★★ | 業務 commit 後通知失敗只 `tracing::warn!`（如 `euthanasia.rs:88-99`），不一致 |
| **euthanasia** | ★ HIGH | `pi_approve` / `pi_appeal` / `chair_decide` **無 tx + 無 FOR UPDATE**；`pi_appeal:275-298` 連續兩次 pool-level UPDATE 跨呼叫 |

### HIGH（立即可造成 lost update / 不一致）
1. **`backend/src/services/euthanasia.rs:171, 217, 312`** — 三條核心審批路徑無併發保護；雙端同時操作可導致狀態錯亂、execute 兩次。
2. **`backend/src/services/euthanasia.rs:275-298`** — `pi_appeal` 連續兩段 UPDATE 中間狀態可被讀到。
3. **protocol body PUT** — 兩 reviewer 同時編輯 IACUC 內容會靜默覆寫；待逐 mutation 驗證 `update_protocol` SQL 是否有 optimistic lock。

### MEDIUM
- `services/amendment/crud.rs:232` `max_version + 1` 取號，需確認 DB 端 UNIQUE 約束防 race。
- 通知失敗於 commit 後吞訊（`tracing::warn!`）→ 建議 outbox pattern。
- 前端 mutations **未統一帶 `version` 欄位**送回 server；需逐 mutation 驗證。
- **無 presence / soft-lock UI**：併發編輯只能事後 409 toast，UX 不佳。

### 範本（推廣藍本）
- `animal/core/update.rs`：FOR UPDATE + optimistic version + audit-in-tx 三件套。
- `amendment/workflow.rs`：終態守衛 + 簽章同 tx，GLP 合規範本。
- `storage_location.rs:632`：atomic decrement + rows_affected 偵測。

---

## 軸二：操作日誌記錄與顯示

### 已做好
- **R26 Service-driven pattern 普及**：129 處 `log_activity_tx` 跨 34 個 service；舊 `log_activity` 已退場。audit 與業務同 transaction。
- **HMAC chain v2**：migration 037 length-prefix canonical 編碼避免字串串接碰撞；`services/audit_chain_verify.rs` 每日驗證 + `audit_chain_broken` 告警。
- **Diff & redact**：`models/audit_diff::DataDiff::compute<T: Redactable>` 編譯期強制 redact；`changed_fields` 保留欄位名供稽核但值為 `[REDACTED]`。
- **Impersonation (SEC-11)**：migration 034 `impersonated_by_user_id` 納入 HMAC，前端 Dialog 突顯。
- **i18n label**：`constants/auditLogs` 提供 zh-TW 對照表，非 raw enum string。
- **時區**：DB `TIMESTAMPTZ` UTC、前端列表 `formatDateTimeDisplay` 顯式 `Asia/Taipei`。

### GAP（未寫 audit 的關鍵路徑）
| File | 缺失 |
|------|------|
| `services/euthanasia.rs:43-117` `create_order` | 整檔 0 處 audit |
| `services/protocol/core.rs::create / update` | protocol CRUD 無 audit（status/history 有） |
| `handlers/animal/sudden_death.rs` | 突發死亡通報 0 處 audit |
| `handlers/accounting.rs` | 金額路徑 0 處 audit |
| `services/animal/import_export.rs` | 動物批次 import/export 無 audit |
| `services/animal/vet_patrol.rs` | 用 redact 但無 `log_activity` 呼叫 |

### UX
- before/after 用 `JSON.stringify(_, null, 2)` 整塊紅/綠 `<pre>` dump → **無 key-by-key 對比、無 highlight `changed_fields`**。
- **缺自由文字搜尋**（無法搜 entity name / IP / display name）。
- 表格未套 container-queries / 卡片化（`AuditLogTable.tsx`），窄螢幕需橫向 scroll；但**沒有 truncate**（符合偏好）。
- Dialog 時區是否為 GMT+8 取決於 `lib/utils.ts` `formatDate/formatTime` 內部設定，需驗證。

### SECURITY
- **CSV 匯出時間欄為 UTC**（`handlers/audit.rs:169`）— 與使用者偏好不符，稽核解讀風險。
- **DB 層未 REVOKE UPDATE/DELETE**：完整性僅靠 HMAC 事後偵測；GLP §11.10 期待 prevention，建議 `REVOKE UPDATE, DELETE ON user_activity_logs FROM <app_role>` 或 `BEFORE UPDATE/DELETE` trigger。
- `AuditService::log_activity_oneshot`（`handlers/audit.rs:293`）用 `tokio::spawn` fire-and-forget，無 retry / dead letter。
- redact trait 是 opt-in：新增 entity 開發者忘記實作 `Redactable` → 預設不 redact，仰賴 code review。
- **CSV 匯出缺 before/after 與 changed_fields 欄位**：稽核員外帶資料看不到變更內容。

### Append-only & 保留期
- 表 `PARTITION BY RANGE (partition_date)`，分區建到 2028 Q4。
- **未加 7 年保留期 partition drop 政策**。
- 匯出端點 `/admin/audit-logs/export?format=csv|json` 已實作（CSV BOM 支援中文 Excel）。

---

## 軸三：GLP 合規性

### A. 資料完整性（ALCOA+）

**現況**：`log_activity_tx` + `ActorContext`（uid/role/IP/UA）+ tx 內時戳；`migrations/038_glp_record_locks.sql` 為 observation/surgery/blood_test/care_medication 加 `is_locked/locked_at/locked_by`，配 `services/signature/mod.rs::ensure_not_locked_uuid_tx`（FOR UPDATE）。DB-side `NOW()` 不信任 client clock。

**缺口**
- `signature/mod.rs::sign_internal` 的 `signature_data` = SHA-256（uid:hash:ts:pwd_hash）**無 HMAC key** → 拿到 DB read 者可重算，非 cryptographic non-repudiation。
- `electronic_signatures` **未進 HMAC chain** → DBA 改 `is_valid=false` 不會被偵測。
- `lock_record` 用 `format!()` 拼 table name（白名單擋住注入，但 lint 議題）。

### B. 設計確認（DQ）

**現況**：`docs/R26_compliance_requirements.md` 把 21 CFR §11 / SOC 2 / ISO 27001 條款映射到 R26-1~4。

**缺口**
- **無 traceability matrix**：條款 → 需求 → file → test 雙向追溯表不存在。
- amendment 8 狀態 enum 沒有對應 SOP 流程圖。

### C. IQ / OQ / PQ

**現況**：`startup/config_check.rs` 啟動印 ✅/⚠️ 摘要；`startup/migration.rs` `sqlx::migrate!` checksum；migration 修復工具齊全。

**缺口**
- `handlers/health.rs` **刻意移除 `version`**（M14）— 但稽核員需要知道 production 跑哪個 commit；無 `/version` endpoint。
- `config_check.rs` 只警告 print，不會 fail-fast；ADMIN 弱密碼仍可啟動。
- 無啟動自動 self-test（DB schema 完整性、必要 enum / role / permission 存在）。

### D. 變更控制

**現況**：amendment 完整狀態機（DRAFT → ... → APPROVED/REJECTED）；`migrations/039_amendment_decision_signature.sql` FK 簽章；reviewer 指派 + 狀態改同 tx。

**缺口**
- amendment 無 `EFFECTIVE` 終態與 `effective_from`：approved 後 protocol 何時用新版不明。
- migration **無 down/rollback** SQL。
- `role/permission` 變更走一般 audit，**未強制簽章**核准（不像 amendment 有 signature FK）。

### E. 電子簽章（21 CFR Part 11）

**現況**：`signature/mod.rs::sign_record` 強制密碼驗證（即使手寫簽章 — SEC-BIZ-3）；綁 `content_hash` (SHA-256) 可 verify；`is_valid + invalidated_reason/by/at` 軟失效；簽章後 `lock_record_uuid`。

**缺口**
- **單因素**（密碼）違 §11.200(a)(1)(ii)；`auth/two_factor.rs` 存在但 `sign_record` 未呼叫。
- `signature_data` 非 HMAC（同 A 節）。
- `invalidate()` 是 UPDATE，**未進 audit chain** → 簽章作廢操作無 tamper-evident 軌跡。
- 缺 `meaning / reason` 欄位（§11.50(a)(3) 要求簽章意義：「approved by」「reviewed by」…）。

### F. 資料保留與封存

**現況**：`services/data_export.rs` 全庫 IDXF（JSON/Zip/NDJSON）含 `include_audit` 開關；DR runbook 在 `docs/runbooks/`。

**缺口**
- **保留期未編碼**：grep `retention|7年` 命中 `glp_compliance.rs` 但無 protocol/animal/euthanasia 的 X 年保留 enforcement。
- `services/animal/core/delete.rs` 無 `soft_delete` 字串 → 疑似 hard delete；違 ALCOA Original。
- **IDXF 漏 19 表**（記憶體 `project_backup_missing_tables`）→ §11.10(c)「準確完整紀錄副本」未滿足。
- `data_export.rs::include_audit: false` 預設 → 重建後 chain 斷。

### CRITICAL（阻擋 GLP 上線稽核）
1. 電子簽章單因素 + `signature_data` 非 HMAC — 違 §11.200。
2. `electronic_signatures` 不在 tamper-evident chain。
3. IDXF 匯出漏 19 表 — §11.10(c) 未滿足。
4. 資料保留與 soft-delete 未強制；`animal/core/delete.rs` 疑似 hard delete。

### HIGH（下個 milestone）
- amendment 無 `EFFECTIVE` 狀態 + `effective_from`；migration 無 down。
- `/version` endpoint 缺；無啟動 self-test；config_check 弱密碼仍能啟動。
- 簽章 `meaning/reason` 欄位缺（§11.50）。
- role/permission 變更未強制 signature 流程。
- `audit_chain_verify` 排程驗證需確認 nightly + alert 已落地。

### DOCUMENTATION（程式有做但缺 SOP）
- Traceability matrix 缺。
- AmendmentStatus 8-state 流程圖未對應 IACUC SOP。
- GLP record-lock 5 表選擇理由未文件化。
- HMAC chain 斷鏈處理 runbook 缺。
- DR drill 演練紀錄表缺。
- training 模組與 §11.10(i) 訓練紀錄 SOP 未交叉引用。

---

## R30 計畫（摘要，已驗證）

詳細任務分項見 `docs/TODO.md` R30 章節。R30 以「**先補三軸交叉的 euthanasia 模組 → 簽章系統升級到 21 CFR §11 標準 → IDXF / soft-delete / retention → SOP 文件補完**」為四大階段。

> **2026-04-28 驗證後修訂**：原 42 項中 3 項誤報（animals 已 soft-delete / amendment_versions 已有 UNIQUE / audit_chain_verify 排程已註冊）已移除；6 項措辭精準化（19 表非 17、雙軌 hard delete、retention 部分有實作等）；新增 R30-29「audit_chain_verify_active flag 切 true」。實際 40 項。

| 階段 | 範圍 | 項目數 | 風險等級 | 預估 |
|------|------|-------|---------|------|
| **R30-A** Euthanasia 三軸補強 | 套 R26 pattern：FOR UPDATE + version + audit-in-tx；補 `create_order` audit；簽章保護；通知 outbox | 4 | CRITICAL | 24-32h |
| **R30-B** Protocol body lost update | PUT 用 schema 已有的 version 欄位 + optimistic lock；前端 UpdateProtocolRequest 補欄位 | 2 | HIGH | 6-10h |
| **R30-C** 簽章系統升級 §11.200 | sign_record 走 2FA；signature_data 改 HMAC；electronic_signatures 入 chain；加 `meaning` 欄 | 4 | CRITICAL | 24-40h |
| **R30-D** Audit log 顯示與匯出 | CSV 時區 GMT+8；CSV 補 before/after + changed_fields；前端 diff key-by-key highlight；自由文字搜尋；表格 RWD | 5 | MEDIUM | 12-16h |
| **R30-E** Soft-delete + retention + IDXF 補表 | 移除 legacy hard-delete fn；retention policy 表 + 排程；IDXF 補 19 表 + 覆蓋率測試；include_audit 預設 true | 4 | CRITICAL | 14-20h |
| **R30-F** Append-only DB 防護 | user_activity_logs 補 BEFORE DELETE trigger；electronic_signatures 加 immutability trigger | 2 | HIGH | 3-5h |
| **R30-G** IQ/PQ + 變更控制 | `/version` endpoint；config_check fail-fast；schema/role self-test；amendment `EFFECTIVE`；migration down.sql；role 變更簽章；audit_chain_verify_active 切 true | 7 | HIGH | 14-18h |
| **R30-H** 漏 audit 路徑補齊 | protocol CRUD / sudden_death / accounting / import_export 從 handler 搬 service / vet_patrol | 5 | MEDIUM | 12-16h |
| **R30-I** GLP 文件補完 | traceability matrix；amendment SOP；record-lock 5 表理由；HMAC 斷鏈 runbook；DR drill 紀錄表；training §11.10(i) 對照 | 6 | DOCUMENTATION | 16-20h |

**總計**：40 項；預估 **125-177 小時**（約 4-6 週全職）。

### 順位邏輯
1. **euthanasia (R30-A)** 是三軸交叉最弱點 → **必須最先**，且作為「R30 pattern 驗證 PR」（仿 R26 PR #3 模式）。
2. **簽章 (R30-C)** 是 GLP 上線阻擋根本，但工序大 → 拆 4 個子 PR：HMAC migration / 2FA hook / chain 整合 / meaning 欄。
3. **R30-E + R30-F** 同步進行（DB 層 GRANT/REVOKE + soft-delete schema 同一 PR）。
4. **R30-D** 是稽核員當天看得到的「外觀缺口」，安排在 R30-A、C 之間做為 quick win。
5. **R30-I** SOP 文件可平行於 code 階段，由獨立任務追蹤。

### 停機點（依 CLAUDE.md 執行紀律）
- R30-A 完成後**必停**：使用者驗證 euthanasia 三軸 pattern 可複製。
- R30-C 簽章每個子 PR 後**必停**：21 CFR §11 屬高風險，每段需明確同意。
- R30-E soft-delete schema 變更前**必停**：不可逆 schema 動作，列為使用者明確同意項。
- 其餘 PR 走「pattern 已驗證 → 同 session 內合併做」節奏。

---

## 附錄：審查方法

- 三個 sub-agent 平行執行（Anthropic Claude Code Agent tool）。
- Backend 涵蓋 `backend/src/{handlers,services,repositories,models,middleware,migrations}` 全掃。
- Frontend 涵蓋 `frontend/src/{pages,components,hooks,lib}` 重點掃描 audit 相關頁面與 mutation hooks。
- 關鍵搜尋：`UPDATE.*SET.*WHERE.*version`、`FOR UPDATE`、`pool.begin`、`log_activity_tx`、`Redactable`、`signature_data`、`include_audit`。
- 不執行任何修改；所有結論基於檔案 file:line 證據。
