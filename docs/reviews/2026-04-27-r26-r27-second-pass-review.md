# R26 + R27 第二輪 Code Review — Findings 管理表

- **審查日期**：2026-04-27
- **審查範圍**：R26 Service-driven Audit Refactor（PR #199 epic）+ R27 PRs #217 / #218 / #220 / #221 / #222
- **審查模式**：Second-pass — 排除已知 Medium+ 5 項；僅錄驗證後成立的新發現
- **審查方法**：6 個 parallel sub-agent 分頭跑 → 主審 verify 每筆 critical/high → 拒絕 2 筆 over-stated（SQL JOIN 偽錯、JSON field-order 偽錯）
- **檔案職責**：tracking-only。修復進度更新此檔；若涉及計畫變更同步至 `docs/TODO.md`

---

## 已知排除清單（Pre-filed，本表不再追蹤）

| # | 來源 | Severity | 摘要 |
|---|------|----------|------|
| K1 | R26 | Medium | concurrent audit test 並行度 3 < 規劃 10（pool 限制） |
| K2 | R26 | Medium | Anonymous actor HMAC fallback 文檔不足 |
| K3 | R27 #217 | Medium | docker-entrypoint.sh 無自動化測試 |
| K4 | R27 #218 | Medium | admin always-load 4-table JOIN cost 未量化 |
| K5 | R27 #221 | Medium | silent skip 導致通知遺漏（emergency + abnormal 均失敗） |

---

## 本輪新發現（按 Severity 排序）

> **欄位定義**
> - **驗證**：✅ 已親自核對源碼確認；⚠️ 合理但需 owner 二次確認 awk/CI 細節
> - **狀態**：⏳ 待修 / 🚧 進行中 / ✅ 已修 / 🔴 拒絕（false positive） / ⏸ 已轉 backlog
> - **修復 PR**：填上對應 PR 編號或 commit hash

### 🟧 Medium

| # | 編號 | 摘要 | 位置 | 驗證 | 狀態 | 修復 PR |
|---|------|------|------|------|------|---------|
| 1 | M1 | Migration 037 註解 vs verifier `try-both` 行為矛盾，誤導 backfill 腳本撰寫者 | `backend/migrations/037_audit_hmac_version.sql:26` ↔ `backend/src/services/audit.rs:735–759` | ✅ | ⏳ | — |
| 2 | M2 | Anonymous→SYSTEM HMAC 替代讓 actor 類別在鏈中無法區分，理論上可被竄改 | `backend/src/services/audit.rs:722–724`（verifier）+ `460–462`（writer） | ✅ | ⏳ | — |
| 3 | M3 | Advisory lock key 無中央註冊；i64 常數 vs `hashtext()` 派生兩種命名空間共存 | `backend/src/services/audit_chain_verify.rs:64` ↔ `audit.rs:429` ↔ `protocol/numbering.rs:20` ↔ `auth/login.rs:31` | ⚠️ | ⏳ | — |
| 4 | M4 | middleware 把 repository 的 `AppError::Database` 包成 `AppError::Internal`，error variant 流失（與 `load_permissions` 路徑行為不一致） | `backend/src/middleware/auth.rs:220–225` | ✅ | ⏳ | — |
| 5 | M5 | Prometheus init 失敗時 metrics 靜默掉（NoopRecorder），無 ops 可觀測 | `backend/src/main.rs:141–151` + `backend/src/lib.rs:54–77` + `auth.rs:166–191` | ✅ | ⏳ | — |
| 6 | M6 | `create_animal_observation` 缺 IDOR `require_animal_access`（pre-existing，非 PR #221 引入；handler + service 兩層皆未檢查） | `backend/src/handlers/animal/observation.rs:88–181` + `backend/src/services/animal/observation.rs:88+` | ✅ | ⏳ | — |

### 🟨 Low

| # | 編號 | 摘要 | 位置 | 驗證 | 狀態 | 修復 PR |
|---|------|------|------|------|------|---------|
| 7 | L1 | Migration 037 `hmac_version` 缺 `CHECK (IS NULL OR IN (1, 2))` 約束 | `backend/migrations/037_audit_hmac_version.sql:9` | ✅ | ⏳ | — |
| 8 | L2 | CI `audit-redaction-guard` 用 awk 行掃，多行 struct fixture 風險，無防回歸測試 | `.github/workflows/ci.yml::audit-redaction-guard` | ⚠️ | ⏳ | — |
| 9 | L3 | `NGINX_ENVSUBST_OUTPUT_DIR` compose 設值但 entrypoint 不讀取，意圖不明 | `docker-compose.test.yml:84` ↔ `frontend/docker-entrypoint.sh:33` | ⚠️ | ⏳ | — |
| 10 | L4 | `API_BACKEND_URL` 只驗非空，缺 `https?://` scheme 檢查（nginx 失敗訊息隱晦） | `frontend/docker-entrypoint.sh:25–26` | ✅ | ⏳ | — |

### ⚪ Nit

| # | 編號 | 摘要 | 位置 | 驗證 | 狀態 | 修復 PR |
|---|------|------|------|------|------|---------|
| 11 | N1 | PR #218 拆分後三個 helper（`validate_jwt`/`load_permissions`/`check_user_active_status`）缺獨立 unit test | `backend/src/middleware/auth.rs` + `backend/tests/api_auth.rs` | ✅ | ⏳ | — |
| 12 | N2 | metric 命名 commit message vs 實作精神不一致（口語 hits/misses vs label-based requests_total{result}） | `backend/src/middleware/auth.rs:167–190` | ✅ | ⏳ | — |
| 13 | N3 | `classify_minor_with_signature_tx` 57 行 / `classify_major_with_reviewers_tx` 51 行，未達 PR description 「~45 行」目標 | `backend/src/services/amendment/workflow.rs:292–402` | ✅ | ⏳ | — |

### 🔴 拒絕 — Sub-agent 過度發現（驗證後判定 false positive）

| # | 摘要 | 拒絕理由 |
|---|------|----------|
| R1 | PR #218 「SQL JOIN Critical 邏輯錯誤」 | `INNER JOIN user_roles ON rp.role_id = ur.role_id` 是標準橋接（`role_permissions.role_id` 與 `user_roles.role_id` 都是 `roles.id`），加 `DISTINCT` 處理多角色重複，**正確** |
| R2 | R26 「JSON field-order 非確定 → HMAC false positive」 | `backend/Cargo.toml:29` 沒啟用 `serde_json` 的 `preserve_order` feature，預設 `BTreeMap` 鍵自動排序，**確定性 OK** |
| R3 | PR #220 record_decision TOCTOU | `SELECT … FOR UPDATE` 鎖到 `tx.commit()`，agent 自己 verify 後也判定 safe |
| R4 | PR #221 「animal fetch silent skip」 | 與已知 K5 同機制，**重複** |

---

## 修復建議優先序

1. **M6 IDOR**（pre-existing，但屬 R26-11 IDOR 統一原則的漏網之魚，安全分量重）
2. **M1 + M2** 一起處理（同屬 R26 audit chain 完整性 / 可審計性語意）
3. **M5 Prometheus 失盲**（生環境風險）
4. **M3 / M4 / L1–L4 / N1–N3** 進 R28 backlog，依時程逐步處理

---

## 修復檢核清單（每筆完成時打勾）

- [ ] M1 — Migration 037 註解修正 + 補 idempotent backfill 腳本
- [ ] M2 — HMAC 計算納入 actor 類別 / 或補 design doc 說明 accepted residual risk
- [ ] M3 — `backend/src/constants/lock_keys.rs` 中央註冊 + 規範文檔
- [ ] M4 — `check_user_active_status` 改為直接 `?` 透傳或 wrap 時保留 source
- [ ] M5 — Prometheus init 失敗 fail-fast 或 healthcheck degraded 標記
- [ ] M6 — handler 加 `require_animal_access`，或下沉到 service `create()` 開頭
- [ ] L1 — Migration 037 補 CHECK 約束 + 測試
- [ ] L2 — CI audit-redaction-guard 補 multi-line struct fixture test
- [ ] L3 — Compose 變數加註解 / 或修 entrypoint 讀取該值
- [ ] L4 — entrypoint 補 `https?://` scheme 驗證
- [ ] N1 — 三個 helper 各補 unit test
- [ ] N2 — PR description / runbook 對齊 metric 命名
- [ ] N3 — classify helper 續抽 audit log 寫入塊（可選）

---

## 變更記錄

| Date | Change | By |
|------|--------|-----|
| 2026-04-27 | Initial — 13 findings + 4 拒絕項 + 5 已知排除 | 第二輪 review |
