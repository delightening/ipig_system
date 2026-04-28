# GLP Record Lock 5 表選擇理由

> **用途**：解釋為何選定這 5 張表納入 GLP record lock 機制（21 CFR §11.10(e)(1)），而其他類似性質表暫不納入。
> **適用範圍**：`backend/migrations/006`（`animal_sacrifices`）+ `backend/migrations/038_glp_record_locks.sql`（其餘 4 表）+ `services/signature/mod.rs::lock_record`。
> **維護者**：QAU + 後端維護者。新增受規範表（如未來 transplant_records）時須評估是否納入並更新本文。
> **語言備註**：條款引用保留英文原文。

---

## 1. 5 張被鎖定的表

| 表 | Migration | 對應 Service |
|---|---|---|
| `animal_observations` | `038` | `services/animal/observation.rs` |
| `animal_surgeries` | `038` | `services/animal/surgery.rs` |
| `animal_sacrifices` | `006`（最早即內建） | `services/animal/sacrifice.rs` |
| `animal_blood_tests` | `038` | `services/animal/blood_test.rs` |
| `care_medication_records` | `038` | `services/care_medication.rs` |

每張表皆有 `is_locked BOOLEAN NOT NULL DEFAULT false`、`locked_at TIMESTAMPTZ`、`locked_by UUID REFERENCES users(id)` 三欄。

---

## 2. 各表 lock 設計理由

### 2.1 `animal_observations`（動物觀察記錄）

- **為什麼 lock**：日常臨床觀察是 GLP §8 Performance of the Study 的原始研究數據（raw data）。一旦獸醫簽章，代表「我目擊並對此記錄負責」，不可事後修改。
- **觸發點**：獸醫於觀察 form 上完成電子簽章後，同 transaction 內 `lock_record()`。
- **鎖後仍允許**：
  - `softdelete with reason`（`deleted_at` + `deletion_reason` 必填，需 admin 權限）
  - `append child record`（如「補充觀察 #2」連結至原 record，但不修改原 row）
- **不允許**：UPDATE 任何欄位（`ensure_not_locked` guard 拒 409）

### 2.2 `animal_surgeries`（手術記錄）

- **為什麼 lock**：手術為計畫關鍵節點（protocol milestone），時間、術者、麻醉方案、術中所見等資訊不得事後改寫，否則無法支持後續組織病理 / 數據分析的可信度。
- **觸發點**：手術完成後 PI 或主刀獸醫簽章。
- **鎖後仍允許**：附件追加（術中影像、後續病理報告 PDF）— 透過 `attachments` 表關聯，不動原 row。
- **特殊規則**：手術死亡需另開 `animal_sacrifices` record，不在此處改 `outcome`。

### 2.3 `animal_sacrifices`（犧牲記錄）

- **為什麼 lock**：動物終點為不可逆事件，是 IACUC 核心追蹤指標。安樂死方式、執行人、時間若可改 → IACUC 稽核失去意義。
- **觸發點**：獸醫填寫並簽章後立刻鎖（migration 006 即內建）。
- **鎖後仍允許**：附件（屍解報告 PDF、組織取樣 manifest）。
- **歷史背景**：本表是最早納入 lock 機制的表，後續其他表仿照此模式擴展。

### 2.4 `animal_blood_tests`（血液檢驗記錄）

- **為什麼 lock**：實驗室檢驗結果為定量數據（quantitative raw data），是後續統計分析、報告 figure 的基礎。GLP §9 Reporting 要求所有報告數據可追回到原始紀錄；若可改，等同於可造假。
- **觸發點**：QAU 或 lab manager 確認結果後簽章。
- **鎖後仍允許**：
  - 附件追加（儀器原始輸出 PDF、外送報告掃描）
  - `record_annotations` 表寫入「補註說明」（不改原數值，但說明異常 / 重檢理由）

### 2.5 `care_medication_records`（給藥 / 照護記錄）

- **為什麼 lock**：給藥紀錄涉及管制藥（controlled substances，如麻醉劑）追蹤，DEA / 衛福部食藥署稽核要求精確帳目。同時影響動物福祉判定（IACUC 關注）。
- **觸發點**：給藥執行人完成 form 並簽章。
- **鎖後仍允許**：
  - `softdelete with reason`（用於記錄筆誤；需 admin + reason）
  - 後續給藥另開新 row

---

## 3. 為什麼是「這 5 張」而非其他

對照與本系統其他類似 / 相關表的判斷：

| 表 | 是否 lock | 理由 |
|---|---|---|
| `animal_transfers` | ❌ 不鎖 | 轉籠 / 移動屬「行政事件」非原始研究數據；IACUC 通常不要求簽章。改用 audit log 追溯即可。 |
| `animal_weights` | ❌ 不鎖 | 體重為高頻測量（每日 / 每週），常有筆誤需更正。改用 `record_versions` 快照保留歷史版本，搭配 audit log 即達合規。 |
| `animal_euthanasia_plans` | ❌ 不鎖 | 計畫（plan）非執行紀錄；實際執行落在 `animal_sacrifices`，由後者 lock。 |
| `animal_births` | ❌ 不鎖（暫定） | 出生紀錄為靜態事實，目前以 audit log 追溯。若客戶 IACUC 要求簽章可未來納入。 |
| `animal_pen_assignments` | ❌ 不鎖 | 設施配置為行政性，無 GLP 簽章要求。 |
| `equipment_maintenance` | ❌ 不鎖（但有 audit） | GLP §4 要求設備校驗紀錄保存，但未要求 immutable lock；以 audit + 簽章日期欄位達成。 |
| `protocols`（IACUC 計畫主表） | ❌ 不鎖 | 改用 `amendments` 機制管理變更，原 protocol row 由 amendment workflow 守護。 |
| `amendments` | 🟡 by FK | 透過 `approved_signature_id IS NOT NULL` 達成等效鎖（C2 修補後）；不直接用 `is_locked` 欄。 |

---

## 4. 鎖定後操作的統一規則

所有 service 層在 UPDATE / DELETE 前必須呼叫：

```rust
ensure_not_locked(tx, table, record_id).await?;
```

該 helper 查詢 `is_locked` 欄；若 `true` 即回 `AppError::Conflict("record is locked after signature")`，handler 統一映射為 HTTP 409。

例外：admin role 的 softdelete 路徑透過獨立 endpoint，必須附 `deletion_reason`，並另寫 `audit_log` 標記 `event_category="ADMIN_OVERRIDE"`。

---

## 5. 反向引用

- Migration 來源：`backend/migrations/006_*.sql`（sacrifices）、`backend/migrations/038_glp_record_locks.sql`（其餘 4 表）
- 簽章鎖定實作：`backend/src/services/signature/mod.rs::lock_record` / `lock_record_uuid`
- Traceability：[`traceability-matrix.md`](traceability-matrix.md) §11.10(e)(1) / §11.70
- 缺口背景：[`../audit/system-review-2026-04-25.md`](../audit/system-review-2026-04-25.md) §1 [C1]
