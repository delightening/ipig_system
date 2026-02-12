# SQLx 離線查詢索引

> 此檔案是 `.sqlx/` 目錄中 50 個查詢快取檔的可讀索引。
> 自動產生於 2026-02-12。檔名為查詢內容的 SHA-256 hash 前 12 碼。

---

## 📋 修正案 (Amendments) — 22 筆

### 基本 CRUD

| Hash | 操作 | SQL 摘要 | 參數 | 回傳欄位 |
|------|------|---------|------|---------|
| `eb2a8d5976b4` | INSERT | 建立新修正案（status=DRAFT） | 9 | 18 |
| `e8129b0bbc7c` | SELECT | 依 ID 查詢單一修正案 | 1 | 18 |
| `3e90acf3639f` | SELECT | 查詢修正案清單（支援 protocol_id / status / type 篩選） | 3 | 17 |
| `b47168104711` | UPDATE | 更新修正案內容（title / description / change_items / changes_content） | 5 | 18 |

### 狀態流轉

| Hash | 操作 | SQL 摘要 | 參數 | 回傳欄位 |
|------|------|---------|------|---------|
| `115ee43b69cd` | UPDATE | 提交修正案（更新 status / submitted_by） | 3 | 18 |
| `49b0c6d883cc` | UPDATE | 變更修正案狀態（通用） | 2 | 18 |
| `939388851854` | UPDATE | 分類修正案（設定 amendment_type / classified_by） | 5 | 18 |
| `26181675611e` | UPDATE | 設為 UNDER_REVIEW | 1 | 18 |
| `ce711b303632` | UPDATE | 設為 REVISION_REQUIRED | 1 | 0 |
| `7861d5defbc0` | UPDATE | 設為 APPROVED | 1 | 0 |
| `49226af9bd9d` | UPDATE | 設為 REJECTED | 1 | 0 |
| `a7a15589e942` | INSERT | 記錄修正案狀態異動歷史 | 6 | 0 |
| `ebaba471fec8` | SELECT | 查詢修正案狀態歷史 | 1 | 7 |

### 版本管理

| Hash | 操作 | SQL 摘要 | 參數 | 回傳欄位 |
|------|------|---------|------|---------|
| `714efd67dbf4` | SELECT | 取得 protocol 下最大 revision_number | 1 | 1 |
| `1ec399582128` | SELECT | 取得 amendment 下最大 version_no | 1 | 1 |
| `535731bf32e5` | INSERT | 新增修正案版本快照 | 5 | 6 |
| `4f08a19449e6` | SELECT | 查詢修正案版本清單（降序） | 1 | 6 |

### 審查指派

| Hash | 操作 | SQL 摘要 | 參數 | 回傳欄位 |
|------|------|---------|------|---------|
| `34c7a95e349b` | INSERT | 指派審查委員（ON CONFLICT DO NOTHING） | 3 | 8 |
| `7051e8ec87e5` | SELECT | 查詢修正案的審查委員清單 | 1 | 10 |
| `ae1e6578595e` | SELECT | 檢查某人是否已被指派審查此修正案 | 2 | 1 |
| `0f1b8d3e20f7` | UPDATE | 審查委員提交決定 | 4 | 8 |
| `3270426c57a0` | SELECT | 統計修正案審查結果（核准/駁回/需修改） | 1 | 5 |

---

## 🐷 安樂死流程 (Euthanasia) — 16 筆

### 安樂死命令

| Hash | 操作 | SQL 摘要 | 參數 | 回傳欄位 |
|------|------|---------|------|---------|
| `4d26e22469b6` | INSERT | 建立安樂死命令 | 5 | 12 |
| `4f4b94c7bfa5` | SELECT | 依 ID 查詢安樂死命令詳細資訊（含豬隻/獸醫/PI 名稱） | 1 | 16 |
| `b3942b9ea5ca` | SELECT | 查詢 PI 待回應的安樂死命令清單 | 1 | 16 |
| `7744819a550d` | SELECT | 依 ID + PI 查詢 pending_pi 狀態的命令 | 2 | 12 |
| `1297b634574d` | SELECT | 確認命令屬於特定 PI 且為 pending_pi 狀態 | 2 | 1 |
| `3d5c73178ea8` | SELECT | 查詢 approved 狀態的命令 | 1 | 12 |

### 安樂死狀態流轉

| Hash | 操作 | SQL 摘要 | 參數 | 回傳欄位 |
|------|------|---------|------|---------|
| `0a7dedee1d63` | UPDATE | PI 申訴 → status=appealed | 1 | 0 |
| `f52946823615` | UPDATE | PI 同意 → status=approved（含 pi_responded_at） | 1 | 12 |
| `caa571a7100f` | UPDATE | 設為 approved | 1 | 0 |
| `b6519ed6203b` | UPDATE | 設為 chair_arbitration | 1 | 0 |
| `bd6fe95048b6` | UPDATE | 通用狀態更新 | 2 | 0 |
| `3e37899614a8` | UPDATE | 執行安樂死 → status=executed | 2 | 12 |
| `e2d10ed6caf0` | UPDATE | 自動核准逾期未回應的命令 | 1 | 2 |

### 安樂死申訴

| Hash | 操作 | SQL 摘要 | 參數 | 回傳欄位 |
|------|------|---------|------|---------|
| `a65f52dbf336` | INSERT | 建立申訴 | 6 | 10 |
| `136e196bb7de` | SELECT | 查詢待主席裁決的申訴 | 2 | 10 |
| `a47afd39f714` | UPDATE | 主席裁決（approve/reject） | 2 | 10 |
| `128d6f21cd0b` | UPDATE | 申訴逾期自動駁回 | 1 | 0 |
| `9d9367c7c258` | SELECT | 查詢逾期未裁決的申訴 | 1 | 3 |

---

## 🧬 計畫書 / 使用者 / 動物 (Protocols / Users / Pigs) — 12 筆

### 計畫書 (Protocols)

| Hash | 操作 | SQL 摘要 | 參數 | 回傳欄位 |
|------|------|---------|------|---------|
| `faa7f164d1ee` | SELECT | 查詢計畫書狀態 | 1 | 1 |
| `c43ca869ca75` | SELECT | 查詢計畫書的 IACUC 編號 | 1 | 1 |
| `e7de3a7592ad` | SELECT | 查詢計畫書的審查委員 ID 清單 | 1 | 1 |

### 使用者與計畫書關聯

| Hash | 操作 | SQL 摘要 | 參數 | 回傳欄位 |
|------|------|---------|------|---------|
| `0c89c61d8be1` | SELECT | 檢查使用者是否關聯於某計畫書 | 2 | 1 |
| `a0a00775c59e` | SELECT | 檢查使用者是否為某計畫書的 PI | 2 | 1 |
| `59415812fdb6` | SELECT | 查詢使用者關聯的所有 protocol_id | 1 | 1 |
| `dedbfecae0c1` | SELECT | 查詢 IACUC 主席（活躍使用者） | 0 | 1 |

### 豬隻 (Pigs)

| Hash | 操作 | SQL 摘要 | 參數 | 回傳欄位 |
|------|------|---------|------|---------|
| `c3667d2acd67` | SELECT | 依 ID 查詢豬隻資訊（含 PI） | 1 | 4 |
| `01b42f232544` | SELECT | 查詢豬隻、使用者、獸醫的資訊（for 通知） | 3 | 5 |
| `b5281838e196` | UPDATE | 更新豬隻狀態 | 2 | 0 |

---

## 📊 統計

| 分類 | 查詢數 | SELECT | INSERT | UPDATE |
|------|--------|--------|--------|--------|
| 修正案 (Amendments) | 22 | 10 | 4 | 8 |
| 安樂死 (Euthanasia) | 16 | 5 | 2 | 9 |
| 計畫書 / 使用者 / 豬隻 | 12 | 9 | 0 | 1*(pig status)* |
| **合計** | **50** | **24** | **6** | **18+2** |
