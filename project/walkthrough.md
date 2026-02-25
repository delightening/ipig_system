# 程式碼拆分完成報告

---

## 根目錄整理（2026-02-25）

為減少根目錄雜亂，進行以下調整並已更新所有引用：

| 變更 | 說明 |
|------|------|
| **docs/** | 新增 `docs/`，將 `DATA_RETENTION_POLICY.md`、`DEPLOYMENT.md`、`DR_RUNBOOK.md`、`GLP_VALIDATION.md`、`PROGRESS.md`、`QUICK_START.md`、`TODO.md`、`USER_GUIDE.md` 移入。`README.md`、`CLAUDE.md` 保留於根目錄。 |
| **deploy/** | 新增 `deploy/`，將 `cloudflared-config.yml`、`grafana_dashboard.json` 移入，供隧道與監控使用。 |
| **scripts/** | 將根目錄的 `start.ps1`、`start_tunnel.ps1`、`start_named_tunnel.ps1` 移入 `scripts/`。`start_named_tunnel.ps1` 改為以專案根目錄為工作目錄，設定檔路徑為 `deploy/cloudflared-config.yml`。 |
| **連結** | `README.md` 內文件導覽改為 `docs/QUICK_START.md` 等；`docs/` 內互相引用維持檔名，對 `README` 改為 `../README.md`。 |
| **.gitignore** | 新增 `last_error.json`，避免執行期產物被提交。 |

啟動隧道請從專案根目錄執行：`.\scripts\start_named_tunnel.ps1`（或指定 `-ConfigPath deploy/cloudflared-config.yml`）。

---

## 概述（程式碼拆分）

將三個超大檔案拆分為模組化結構，總計減少了約 **5,850 行**的單檔案負擔。`cargo check` 通過，0 errors。

---

## 任務二：拆分 main.rs

| 項目 | 變更前 | 變更後 |
|------|--------|--------|
| `main.rs` | 893 行 | **165 行** |

### 新增檔案

| 檔案 | 說明 |
|------|------|
| [db.rs](file:///d:/Coding/ipig_system/backend/src/db.rs) | 資料庫連線池建立與重試邏輯 |
| [startup/mod.rs](file:///d:/Coding/ipig_system/backend/src/startup/mod.rs) | 啟動模組入口 |
| [startup/admin.rs](file:///d:/Coding/ipig_system/backend/src/startup/admin.rs) | 管理員帳號初始化 |
| [startup/schema.rs](file:///d:/Coding/ipig_system/backend/src/startup/schema.rs) | 資料庫 schema 確認 |
| [startup/permissions.rs](file:///d:/Coding/ipig_system/backend/src/startup/permissions.rs) | 權限與角色初始化 |
| [startup/dev_users.rs](file:///d:/Coding/ipig_system/backend/src/startup/dev_users.rs) | 開發環境預設帳號 |

---

## 任務三：拆分 animal.rs

| 項目 | 變更前 | 變更後 |
|------|--------|--------|
| `animal.rs` | 3,217 行 | **65 行**（mod.rs） |

### 新增子模組

| 檔案 | 說明 | 約行數 |
|------|------|--------|
| [source.rs](file:///d:/Coding/ipig_system/backend/src/services/animal/source.rs) | 動物來源管理 | ~75 |
| [core.rs](file:///d:/Coding/ipig_system/backend/src/services/animal/core.rs) | 動物 CRUD 與分配 | ~470 |
| [observation.rs](file:///d:/Coding/ipig_system/backend/src/services/animal/observation.rs) | 觀察試驗紀錄 | ~250 |
| [surgery.rs](file:///d:/Coding/ipig_system/backend/src/services/animal/surgery.rs) | 手術紀錄 | ~260 |
| [weight.rs](file:///d:/Coding/ipig_system/backend/src/services/animal/weight.rs) | 體重紀錄 | ~125 |
| [medical.rs](file:///d:/Coding/ipig_system/backend/src/services/animal/medical.rs) | 疫苗/犧牲/獸醫/版本歷史/匯出/病理 | ~550 |
| [import_export.rs](file:///d:/Coding/ipig_system/backend/src/services/animal/import_export.rs) | 模板生成與檔案匯入 | ~890 |
| [blood_test.rs](file:///d:/Coding/ipig_system/backend/src/services/animal/blood_test.rs) | 血液檢查管理 | ~590 |

---

## 任務三：拆分 protocol.rs

| 項目 | 變更前 | 變更後 |
|------|--------|--------|
| `protocol.rs` | 1,906 行 | **21 行**（mod.rs） |

### 新增子模組

| 檔案 | 說明 | 約行數 |
|------|------|--------|
| [core.rs](file:///d:/Coding/ipig_system/backend/src/services/protocol/core.rs) | 計畫 CRUD | ~310 |
| [status.rs](file:///d:/Coding/ipig_system/backend/src/services/protocol/status.rs) | 提交與狀態管理 | ~520 |
| [numbering.rs](file:///d:/Coding/ipig_system/backend/src/services/protocol/numbering.rs) | APIG/IACUC 編號生成 | ~140 |
| [history.rs](file:///d:/Coding/ipig_system/backend/src/services/protocol/history.rs) | 版本與活動歷程 | ~190 |
| [review.rs](file:///d:/Coding/ipig_system/backend/src/services/protocol/review.rs) | 審查人員管理 | ~350 |
| [comment.rs](file:///d:/Coding/ipig_system/backend/src/services/protocol/comment.rs) | 審查意見與回覆 | ~290 |
| [my_protocols.rs](file:///d:/Coding/ipig_system/backend/src/services/protocol/my_protocols.rs) | 我的計畫/獸醫審查 | ~160 |

---

## 驗證結果

```
cargo check: Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.27s
Errors: 0
Warnings: 10 (均為 unused imports，可用 cargo fix 自動修復)
```

## 待辦

- [ ] 任務一：Token → HttpOnly Cookie（尚未開始）
- [ ] 整合測試驗證
