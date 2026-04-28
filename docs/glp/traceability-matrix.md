# GLP / 21 CFR Part 11 Traceability Matrix

> **用途**：供稽核員（QAU、外部 GLP / FDA pre-audit）一覽各條款的實作位置與測試覆蓋。
> **適用範圍**：ipig_system 全模組（backend + migrations + tests）。
> **維護者**：QA / 系統管理員。每次條款相關 PR 合併後同步更新「狀態」與引用路徑。
> **語言備註**：條款引用保留英文原文以便對照 FDA / OECD 原文；說明採繁體中文。

---

## 1. 圖示說明

| 圖示 | 含義 |
|------|------|
| ✅ | 已實作 + 測試覆蓋 |
| 🟡 | 部分實作（程式或測試其一缺口） |
| 🔴 | 未實作 / 已知缺口（追蹤中） |
| — | 不適用 / 流程性條款（非程式可實作） |

---

## 2. 雙向追溯表（21 CFR Part 11）

| 條款 | 需求摘要 | Migration | Service / Module | Test | 狀態 |
|---|---|---|---|---|---|
| §11.10(a) Validated systems | 系統需經 IQ / OQ / PQ 驗證 | — | — | — | 🟡 IQ/OQ/PQ docs 缺（R30 backlog） |
| §11.10(b) Accurate copies | 提供準確完整副本，可供 FDA 稽核還原 | — | `services/data_export.rs` | `tests/data_export` | 🟡 漏 19 表（R30-18 追蹤中） |
| §11.10(c) Record retention | 紀錄保留期間可被還原 | `016_glp_compliance` | — | — | 🔴 跨表 retention policy 未實作（R30-17） |
| §11.10(d) System access limited | 僅授權人員可存取 | — | `services/auth/*`、`middleware/auth.rs` | `tests/auth` | ✅ |
| §11.10(e) Audit trail | 安全、電腦自動產生、時序的稽核軌跡 | `012`, `035`–`037` | `services/audit.rs`、`services/audit_chain_verify.rs` | `tests/audit` | ✅ |
| §11.10(e)(1) Record protection | 簽章後紀錄不得被竄改 | `006` (`animal_sacrifices`)、`038_glp_record_locks` | `services/signature/mod.rs::lock_record` | `tests/signature_lock` | ✅（5 表覆蓋，見 record-lock-rationale.md） |
| §11.10(f) Operational checks | 步驟順序強制（如 workflow 必經狀態） | `amendment_status` enum | `services/amendment/workflow.rs` | `tests/amendment_workflow` | ✅ |
| §11.10(g) Authority checks | 僅授權者可執行特定操作 | — | `services/access.rs`、`middleware/auth.rs` | `tests/auth` | ✅ |
| §11.10(h) Device checks | 終端 / 來源裝置驗證 | — | — | — | — 不適用（無硬體裝置整合） |
| §11.10(i) Education / training | 操作者具備教育訓練紀錄 | `016` (`training_records`) | `services/training.rs` | — | 🟡 SOP 對照缺（見 training-records-sop.md，R30-39） |
| §11.10(j) Written policies | 簽署者責任之書面政策 | — | — | — | 🟡 SOP 文件補強中（R30-I） |
| §11.10(k) Documentation controls | 系統文件版本控管 | — | — | — | 🟡 文件版本以 git 控管，正式 SOP 缺 |
| §11.30 Open systems | 開放系統額外加密 | — | — | — | — 不適用（內網 / VPN 部署） |
| §11.50 Manifestation of meaning | 簽章意義（approve / review / authorship）需可呈現 | `003` (electronic_signatures) | `services/signature/mod.rs` | — | 🔴 `meaning` 欄缺（R30-10） |
| §11.70 Signature linking | 簽章與紀錄連結，不可拆 | `003`、`038` | `services/signature/mod.rs::lock_record` | `tests/signature_lock` | ✅ |
| §11.100(a) Unique signing | 每位簽署者唯一身份 | `003` | `services/signature` | `tests/signature` | ✅ |
| §11.100(b) Identity verification | 簽章前驗證身份 | — | `services/auth/password.rs` | `tests/auth` | ✅ |
| §11.100(c) Certification to FDA | 公司聲明電子簽章合法等同手寫 | — | — | — | — 客戶交付時提供聲明書 |
| §11.200(a)(1)(i) Two distinct components | 雙因素（如密碼 + token） | — | `services/signature` | — | 🔴 目前單因素（密碼）（R30-8） |
| §11.200(a)(1)(ii) Continuous session | 同一連續 session 內可只用一因素，但首次需雙因素 | — | `services/auth/two_factor.rs` | — | 🟡 R26-1 強制 admin 2FA 已上線；簽章流程未串 |
| §11.200(a)(2) Genuine claim | 簽章後不可否認 | `003` HMAC | `services/audit_chain_verify.rs` | `tests/audit_chain` | ✅ |
| §11.200(a)(3) Other than owner can't use | 防他人冒用 | — | `services/auth/password.rs` (Argon2)、`services/signature` (二次密碼) | `tests/auth` | ✅ |
| §11.300(a) Unique combination | ID + 密碼組合唯一 | — | `services/auth` | `tests/auth` | ✅ |
| §11.300(b) Periodic checks / revisions | 密碼定期更換 | — | `services/auth/password.rs` | `tests/auth_password_age` | 🟡 政策已就緒，強制週期待確認 |
| §11.300(c) Loss management | 遺失 / 失竊裝置處理 | — | `services/auth` | — | 🟡 帳號鎖定有，遺失通報 SOP 缺 |
| §11.300(d) Unauthorized use | 偵測未授權使用並通報 | `037` (security_alerts) | `services/security_notifier` | `tests/security_alerts` | ✅ |
| §11.300(e) Initial / periodic testing of devices | 認證裝置（如 token）測試 | — | — | — | — 不適用（無硬體 token） |

---

## 3. OECD GLP Principles（補充對應）

| 章節 | 需求摘要 | 對應實作 | 狀態 |
|---|---|---|---|
| §1 Test Facility Organization | 試驗機構組織 | `services/access.rs` 角色定義 | ✅ |
| §2 Quality Assurance Programme | QAU 稽核 | `services/audit.rs` + admin dashboard | ✅ |
| §3 Facilities | 設施 | `services/facility.rs` | ✅ |
| §4 Apparatus, Material, and Reagents | 設備校驗 | `services/equipment.rs` | ✅ |
| §5 Test Systems | 試驗系統（動物） | `services/animal/*` | ✅ |
| §6 Test and Reference Items | 試驗物品管理 | `services/inventory` | ✅ |
| §7 Standard Operating Procedures (SOPs) | SOP 書面程序 | `docs/glp/*`、`docs/runbooks/*` | 🟡 R30-I 補完中 |
| §8 Performance of the Study | 研究執行 | `services/protocol/*`、`services/amendment/*` | ✅ |
| §9 Reporting of Study Results | 結果報告 | `services/data_export.rs` | 🟡 漏 19 表（R30-18） |
| §10 Storage and Retention | 紀錄保留 | `migrations/*` 分區表 + backup | 🟡 跨表 retention policy（R30-17） |

---

## 4. R30 修補對照（已知缺口的 task 編號）

| 條款 | 缺口 | 對應 R30 task |
|---|---|---|
| §11.10(b) | 資料匯出漏 19 表 | R30-18 |
| §11.10(c) | 跨表 retention policy 未實作 | R30-17 |
| §11.10(i) | training SOP 對照文件 | R30-39（本批次） |
| §11.50 | `meaning` 欄缺 | R30-10 |
| §11.200(a)(1)(i) | 簽章雙因素 | R30-8 |
| §11.10(j) / SOP 全套 | 文件補完 | R30-I（本批次：R30-34 ~ R30-39） |
| Amendment 終態缺 | EFFECTIVE 終態 | R30-25 |

---

## 5. 反向引用

- 本表的缺口分析來源：[`docs/audit/system-review-2026-04-25.md`](../audit/system-review-2026-04-25.md) §1–§4
- Amendment 狀態機 SOP：[`amendment-sop.md`](amendment-sop.md)
- Record lock 表選擇理由：[`record-lock-rationale.md`](record-lock-rationale.md)
- Audit chain 斷鏈處理：[`../runbooks/audit-chain-broken-runbook.md`](../runbooks/audit-chain-broken-runbook.md)
- DR drill 演練紀錄：[`../runbooks/dr-drill-records.md`](../runbooks/dr-drill-records.md)
- Training records SOP：[`training-records-sop.md`](training-records-sop.md)
