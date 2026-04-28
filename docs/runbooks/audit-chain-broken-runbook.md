# Audit Chain Broken — 處理 Runbook

> **用途**：當 HMAC audit chain 驗證偵測到斷鏈時，提供值班人員的標準處理流程。
> **適用範圍**：`backend/src/services/audit_chain_verify.rs` daily 02:00 cron 觸發之 `audit_chain_broken` 告警。
> **維護者**：on-call SRE + QAU。每次實際告警後須回填 §6 處理紀錄表。
> **重要**：本 runbook 為 GLP 合規關鍵流程（21 CFR §11.10(e) 完整性保證），任何偏離須留書面記錄。

---

## 1. 觸發來源

| 項目 | 內容 |
|---|---|
| **排程** | `daily 02:00 (Asia/Taipei, GMT+8)` |
| **執行者** | scheduler.rs `register_audit_chain_verify_job` |
| **驗證範圍** | 上一次成功驗證後的所有新增 audit row（增量 verify） |
| **告警寫入** | `security_alerts` 表，`alert_type='audit_chain_broken'` |
| **通知通道** | `SecurityNotifier` → email + Slack `#sec-oncall`（**TODO[使用者]**：實際通道名稱待確認） |

---

## 2. 通知對象與 SLA

| 角色 | 收到通知 | 行動 SLA |
|---|---|---|
| on-call SRE | 立即（cron 結束後 ≤5 分鐘內） | 4 小時內初步分析 |
| 系統管理員（admin role） | 立即 | 4 小時內加入處理 channel |
| QAU lead | 同步 | 24 小時內收到初步報告 |
| PI / Study Director | 視範圍而定（影響其計畫紀錄時） | 24 小時內被告知 |

**結案 SLA**：發現後 24 小時內結案（含 root cause + 後續行動）。若需 ≥48 小時，需 QAU 書面 extend。

---

## 3. 處理步驟（依序執行，禁止跳步）

### Step 1 — 隔離寫入（5 分鐘內）

> **目標**：避免後續寫入「在已斷裂的 chain 上又疊加」造成 forensic 困難。

- [ ] 通知 admin 暫停受影響服務的寫入：
  - 簽章 endpoint（`POST /signatures`）
  - Audit-heavy 業務 endpoint（amendment decide / animal observation submit / sacrifice）
- [ ] 設定 maintenance mode flag（**TODO[使用者]**：本系統 maintenance mode 機制待確認；若無，臨時方案為下降 backend pod replicas 至 0）
- [ ] 通知所有獸醫使用者「系統暫停寫入，唯讀模式」

### Step 2 — 識別斷裂點（30 分鐘內）

- [ ] 跑診斷 SQL（透過 admin dashboard 或直接 DB）：
  ```sql
  SELECT id, created_at, prev_hash, hash, actor_id, entity_type, entity_id
  FROM audit_log
  WHERE id IN (
      SELECT broken_link_id FROM audit_chain_verify_reports
      WHERE created_at = (SELECT MAX(created_at) FROM audit_chain_verify_reports)
  )
  ORDER BY created_at;
  ```
- [ ] 記錄斷裂點的：
  - row id
  - `created_at` 時間戳
  - 前後 ±10 筆 row 的 actor / entity（context）
- [ ] 確認斷裂是「單點」還是「多點」：
  - 單點 → 多半為單次寫入異常（bug / 競態）
  - 多點 / 連續區段 → 高度懷疑篡改 or 大規模 migration 副作用

### Step 3 — 比對 backup（1 小時內）

- [ ] 取最近一次 daily backup（前一日 03:00 後產出）
- [ ] 對斷裂點所在時段 query 同 row id：
  - 若 backup 中該 row hash 與當前一致 → 斷裂在 backup 之前已發生，需追溯更早
  - 若 backup 中該 row hash 不一致 → 篡改發生於 backup 之後 24 小時內
- [ ] 將 backup 比對結果寫入處理工單（incident ticket）

### Step 4 — 三種可能性判定（4 小時內）

| 情境 | 判定依據 | 處理路徑 |
|---|---|---|
| **(a) 真正篡改（malicious）** | row hash 與 backup 不符 + 找不到合理 migration / bug 解釋 | 觸發 [安全事件流程](../security/THREAT_MODEL.md)；通知資安長 + 法務；保留 forensic 證據 |
| **(b) Bug / migration 副作用** | 可定位到具體 PR / migration 改動 audit row 結構 | 修補 bug；對受影響區段重算 hash chain（需 QAU 簽核）；新增 regression test |
| **(c) 已知容忍情境** | 例如 timezone 不一致導致排序變動、罕見 race window 已修但留下歷史斷點 | 文件化於本 runbook §5「已知容忍清單」；不重算，但 verify report 標記為「accepted exception」 |

### Step 5 — 報告 + post-mortem（24 小時內）

- [ ] 撰寫 incident report，內容含：
  - Timeline（從 cron 觸發到結案）
  - 根因（root cause）
  - 影響範圍（哪些 entity / 哪段時間）
  - 處理動作
  - 後續預防措施（test / monitor / process）
- [ ] post-mortem 會議：on-call SRE + admin + QAU + 影響到的 PI
- [ ] 報告歸檔至 `docs/audit/incidents/YYYY-MM-DD-audit-chain-broken.md`（**TODO[使用者]**：incidents 子目錄結構待確認）

---

## 4. 嚴格禁止的操作

> 以下動作會破壞 GLP 完整性保證、造成稽核時無法解釋，**任何角色（含 superadmin）都不得執行**：

- ❌ **直接 DELETE audit_log row**：無論是「斷裂的那筆」或「為了讓 chain 重新對齊」都不行
- ❌ **直接 UPDATE `hash` 或 `prev_hash` 欄位蓋過去**：會造成「斷鏈被掩蓋」更嚴重的合規問題
- ❌ **跳過 verify report 直接清空 `security_alerts`**：alert 必須走 acknowledged → resolved 流程，留處理紀錄
- ❌ **重跑 `audit_chain_verify` 期望它消失**：cron 是冪等（idempotent）的，若再跑顯示 OK，代表中間有人改了資料 → 反而更可疑
- ❌ **私下處理不開 incident ticket**：QAU 必須有完整審計線索

唯一允許的「修正」路徑：經 QAU 書面簽核的 chain rehash（情境 b），且須在 audit_log 同時寫入一筆 `event_type='CHAIN_REHASH'` 的元事件，記錄誰、何時、為何重算。

---

## 5. 已知容忍清單（accepted exceptions）

> 此區記錄已調查、確認非惡意、可接受的歷史斷點。每筆需 QAU 簽核。

| 日期 | 斷裂 row id 範圍 | 原因 | QAU 簽核 |
|---|---|---|---|
| _(暫無)_ | — | — | — |

---

## 6. 處理紀錄（每次告警必填）

| 日期 | on-call | 斷裂範圍 | 判定（a/b/c） | 結案 SLA 達成？ | Incident report 連結 |
|---|---|---|---|---|---|
| _(暫無)_ | — | — | — | — | — |

---

## 7. 反向引用

- 程式入口：`backend/src/services/audit_chain_verify.rs`、`backend/src/services/scheduler.rs::register_audit_chain_verify_job`
- 告警寫入：`migrations/037_*` (`security_alerts`)
- Traceability：[`../glp/traceability-matrix.md`](../glp/traceability-matrix.md) §11.10(e) / §11.200(a)(2)
- 上游缺口：[`../audit/system-review-2026-04-25.md`](../audit/system-review-2026-04-25.md) §2 [H1]
