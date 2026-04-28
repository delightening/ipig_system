# Training Records SOP — §11.10(i) 對照

> **用途**：將本系統 `training` 模組的功能對應到 21 CFR §11.10(i) 與 OECD GLP §1 對人員教育訓練的要求，作為稽核員查詢「人員是否具備足夠訓練」時的標準回應依據。
> **適用範圍**：`backend/src/services/training.rs` + `backend/src/models/training.rs` + `training_records` 資料表（migration 016）。
> **維護者**：HR + QAU。每次新增 / 修改訓練類型須同步本文件。
> **語言備註**：條款引用保留英文原文。

---

## 1. 法規依據

> **21 CFR §11.10(i)**:
> "Determination that persons who develop, maintain, or use electronic record/electronic signature systems have the education, training, and experience to perform their assigned tasks."

> **OECD GLP §1.1.2(g)**:
> "Test facility management ensure that personnel clearly understand the functions they are to perform and, where necessary, provide training for these functions."

要求：
1. 系統開發者、維護者、使用者皆需具備訓練紀錄
2. 紀錄需可被 FDA / OECD 稽核員查驗
3. 訓練應與實際 assigned task 對應（不是泛用訓練可矇混）

---

## 2. 角色 × 訓練類型對照表

| 角色 | 訓練類型 | 課程 / 內容 | 認證 / Certificate |
|---|---|---|---|
| **系統管理員（admin）** | 系統管理訓練 | ipig_system 後台操作、user 管理、權限管理、備份還原 | 內訓證書（QAU 簽發） |
|  | 資安基礎 | 帳號管理、2FA、釣魚演練 | TODO[使用者]：是否要求外部資安認證 |
|  | DR 演練 | 參與年度 DR drill（[`../runbooks/dr-drill-records.md`](../runbooks/dr-drill-records.md)） | DR drill 紀錄 |
| **獸醫（veterinarian）** | 動物福祉訓練 | AAALAC / 機構動物福祉政策 | TODO[使用者]：機構認可證書 |
|  | IACUC 訓練 | IACUC 角色、protocol / amendment 流程 | IACUC 訓練證書 |
|  | 系統使用訓練 | observation / surgery / sacrifice 紀錄填寫、簽章操作 | 內訓證書 |
|  | 異種器官移植專項 | xenotransplantation SOP（**本機構特化**） | 機構內訓 |
| **PI（Principal Investigator）** | IACUC 訓練 | 計畫主持人 IACUC 訓練 | IACUC 訓練證書 |
|  | 簽章意義訓練 | 21 CFR §11.50 — 電子簽章法律效力理解 | 內訓證書 + 簽署 acknowledgement |
|  | GLP 原則 | OECD GLP 10 章節 overview | 外部 / 內訓證書 |
| **QAU（Quality Assurance Unit）** | 稽核訓練 | GLP / 21 CFR Part 11 稽核技能 | 外部認證（建議：SQA 等） |
|  | 系統稽核訓練 | ipig_system audit log / chain verify / record lock 機制理解 | 內訓證書 |
|  | Incident 處理 | 斷鏈、篡改、外洩等事件處理流程 | 內訓 + 演練紀錄 |
| **Lab Technician** | GLP 基礎 | GLP 基本原則 | 內訓 |
|  | 系統使用訓練 | 與獸醫同（依職責子集） | 內訓 |
| **System Developer / Maintainer** | 21 CFR Part 11 awareness | 開發者 awareness：哪些改動會影響合規 | 內訓 + 簽署 acknowledgement |
|  | Secure coding | OWASP Top 10、Rust 安全慣例 | TODO[使用者]：是否要求外部認證 |

---

## 3. 紀錄方式（系統實作）

### 3.1 資料表

`training_records`（migration 016）：

| 欄位（建議） | 用途 |
|---|---|
| `id` | PK |
| `user_id` | FK → `users.id` |
| `training_type` | 訓練類型 enum（對應 §2 表中「訓練類型」欄） |
| `course_name` | 課程名稱 |
| `completed_at` | 完成時間 |
| `expires_at` | 失效時間（若該訓練有效期限） |
| `certificate_url` | 上傳之證書附件 URL（FK → `attachments`） |
| `issuer` | 發證單位（內訓 / 外部機構名） |
| `notes` | 備註 |

### 3.2 上傳流程

1. HR 或受訓者本人上傳證書 PDF / image 到 `attachments`
2. 建立 `training_records` row 連結附件
3. QAU 抽查驗證

### 3.3 查詢介面

- Admin dashboard 提供「by user」「by training type」「即將到期」三個視角
- Audit export 含 training records（協助 FDA 稽核員一次取得）

---

## 4. 保留期

與其他 GLP record 一致：

- **最少保留 5 年**（OECD GLP §10）
- **建議保留 10 年**（與 study report 同期）
- 即使員工離職，training record 仍須保留至上述期限滿

實作：`training_records` 不允許 hard delete；soft delete 需 admin + 理由，且 audit log 記錄。

---

## 5. 缺口與後續

- 🟡 **目前缺口**：本系統有 training 模組，但**未有正式對照文件說明哪些角色需要哪些訓練類型**。本文件為首次補完。
- 🟡 **TODO[使用者]**：§2 表中標記 `TODO[使用者]` 的外部認證要求需 HR + QAU 共同決議。
- 🟡 **未來**：訓練到期前 30 天自動 email 通知（已規劃，未排程）。

---

## 6. 反向引用

- 程式入口：`backend/src/services/training.rs`、`backend/src/models/training.rs`
- Migration：`backend/migrations/016_glp_compliance.sql`（含 `training_records`）
- Traceability：[`traceability-matrix.md`](traceability-matrix.md) §11.10(i)
- 法規對照：[`../R26_compliance_requirements.md`](../R26_compliance_requirements.md)
