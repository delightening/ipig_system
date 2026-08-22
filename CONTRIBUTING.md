# 貢獻指南 (Contributing Guide)

> 適用對象：所有開發者、維護者、AI agent（Claude Code）。
> 本檔說明「怎麼做」；「做什麼」詳見 docs/TODO.md嚗?冽?隞塚?；「系統架構」詳見 docs/README.md嚗?冽?隞塚?。

---

## 目錄

1. [環境設定](#1-環境設定)
2. [分支策略](#2-分支策略)
3. [Commit 規範](#3-commit-規範)
4. [Pull Request 流程](#4-pull-request-流程)
5. [測試要求](#5-測試要求)
6. [程式碼規範](#6-程式碼規範)
7. [文件更新政策](#7-文件更新政策)
8. [審查流程](#8-審查流程)
9. [合規開發要求](#9-合規開發要求)
10. [AI Agent 協作規則](#10-ai-agent-協作規則)
11. [緊急 Hotfix 流程](#11-緊急-hotfix-流程)

---

## 1. 環境設定

### 1.1 工具需求

| 工具 | 最低版本 | 確認指令 |
|------|----------|----------|
| Rust | 1.82+ | `rustc --version` |
| Node.js | 22.x LTS | `node --version` |
| pnpm | 10.33.0 | `pnpm --version` |
| Docker | 27+ | `docker --version` |
| PostgreSQL | 16（透過 Docker） | — |

### 1.2 首次設定

```bash
# 1. 複製環境變數範本
cp .env.example .env
# 依 .env 內的說明填入必要金鑰

# 2. 啟動開發環境（含 Postgres、API、前端）
docker compose up -d

# 3. 後端：確認編譯與測試通過
cd backend
cargo check
cargo test --lib

# 4. 前端：安裝依賴並啟動開發伺服器
cd frontend
pnpm install
pnpm dev
```

詳見 docs/user/QUICK_START.md嚗?冽?隞塚?。

### 1.3 本地 CI 模擬

```bash
# 後端完整 CI（需本地 Postgres）
docker compose -f docker-compose.test.yml up -d postgres
cd backend && cargo clippy --all-targets -- -D warnings -A deprecated
cd backend && cargo test --all-targets

# 前端 CI
cd frontend && pnpm lint && pnpm test

# 詳細步驟
# docs/dev/ci-local.md
```

---

## 2. 分支策略

### 2.1 命名規則

```text
main                          # 唯一保護分支，禁止直接 push
feature/<ticket-id>-<slug>    # 新功能（例：feature/R32-pdf-export）
fix/<ticket-id>-<slug>        # Bug 修復
hotfix/<slug>                 # 緊急修復，直接自 main 開分支
refactor/<slug>               # 重構
docs/<slug>                   # 純文件更新
chore/<slug>                  # 依賴更新、工具配置
claude/<slug>                 # AI agent 工作分支
```

### 2.2 生命週期規則

- **main** 只接受 PR merge，禁止 force push
- 每個 PR 只做一件事；多個獨立功能請開多個 PR
- 分支存活期不應超過 2 週；長期分支定期 `git rebase main`
- PR merge 後分支自動刪除（GitHub 設定已啟用）

---

## 3. Commit 規範

### 3.1 格式（Conventional Commits）

```text
<type>(<scope>): <subject>

<body>（選填，說明「為什麼」而非「做了什麼」）

Closes #<issue-number>（選填）
```

### 3.2 Type 清單

| Type | 用途 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修復 |
| `refactor` | 重構（無功能變更） |
| `test` | 測試新增或修改 |
| `docs` | 純文件更新 |
| `chore` | 建置、依賴、工具 |
| `perf` | 效能優化 |
| `security` | 資安修補 |
| `compliance` | GLP / 21 CFR Part 11 相關 |
| `migration` | 資料庫 migration |

### 3.3 Scope 建議值

`auth` · `audit` · `animal` · `protocol` · `erp` · `hr` · `notification` · `admin` · `ci` · `docker` · `monitoring` · `deps` · `pdf` · `ems`

### 3.4 Subject 規則

- 使用動詞現在式（英文）或動詞進行式（中文）
- 不超過 72 字元
- 首字母不大寫，句尾不加句點

### 3.5 範例

```text
feat(animal): add bulk transfer API with audit logging

Enables transferring multiple animals across studies in a single
transaction. Each transfer is audited via the HMAC chain (R26).

Closes #142
```

```text
fix(auth): prevent login bypass when TOTP secret is null

Null TOTP secret caused the TOTP check to be skipped entirely.
Now returns 403 with TOTP_NOT_CONFIGURED error code.
```

### 3.6 Commit 粒度

- 每個 PR 應有 **3–15 個 commit**
- 單一 commit **不超過 500 行**
- 每個 commit 應可獨立理解，`git log --oneline` 能看懂脈絡

---

## 4. Pull Request 流程

### 4.1 PR 大小原則

| 類型 | 建議變更行數 |
|------|-------------|
| Bug fix | ≤ 200 行 |
| 新功能（單一模組） | ≤ 500 行 |
| 重構 | ≤ 1000 行（建議拆 PR） |
| Migration | 視情況，附詳細影響說明 |

### 4.2 開啟 PR 前自查

- [ ] 已填寫 PR template 所有必填欄位
- [ ] 標題格式：`<type>(<scope>): <subject>`
- [ ] `cargo clippy` + `cargo test` 本地全綠
- [ ] 若有 DB migration，已附 `migrations/down/` 對稱 migration
- [ ] 若影響合規功能，已更新 `docs/glp/traceability-matrix.md`嚗?冽?隞塚?
- [ ] 已自我 review 一次（`git diff main...HEAD`）

### 4.3 Reviewer 指派

- **CI / Docker / Security / Middleware 相關** → 自動指派 @jasonwang（CODEOWNERS）
- **一般功能 PR** → 至少 1 位 reviewer 批准後方可 merge
- **Breaking change** → 至少 2 位 reviewer 批准

### 4.4 Merge 策略

| 情境 | Merge 方式 |
|------|-----------|
| 一般 feature / fix | Squash and Merge |
| Hotfix（需保留 commit 脈絡） | Merge Commit |
| 禁止 | Rebase Merge |

---

## 5. 測試要求

### 5.1 依 PR 類型

| PR 類型 | 最低測試要求 |
|---------|-------------|
| 純 models / services / repositories 層 | `cargo test --lib` 綠燈 |
| 動到 handlers / middleware / routes | `cargo test --all-targets` 全綠（需本地 Postgres） |
| 只動文件 / CLAUDE.md / migration SQL | `cargo check` 綠燈 |
| 前端功能變更 | `pnpm test` + 關鍵路徑 E2E |

### 5.2 啟動測試用資料庫

```bash
docker compose -f docker-compose.test.yml up -d postgres
```

### 5.3 E2E 測試

```bash
cd frontend
pnpm test:e2e                      # 全部執行
pnpm test:e2e --grep "動物管理"    # 篩選指定流程
```

詳見 docs/dev/e2e/README.md嚗?冽?隞塚?。

---

## 6. 程式碼規範

完整規範詳見 [CLAUDE.md](CLAUDE.md) §代碼規範，以下為重點摘要。

### 6.1 架構分層（嚴格遵守，禁止跨層直接依賴）

```text
Handler → Service → Repository → Model
              ↑
          Middleware
```

### 6.2 後端 (Rust / Axum)

- Handler 只做 HTTP 解析 + 回應組裝，**禁止在 handler 寫 SQL 或業務邏輯**
- 錯誤統一使用 `AppError`，禁止 `unwrap()`（測試碼除外）
- 函式長度 ≤ 50 行，圈複雜度 ≤ 10，參數數量 ≤ 5 個
- 魔術字串定義為 `const` 或 `enum`（見 `constants.rs`）

### 6.3 前端 (TypeScript / React)

- API 呼叫透過 TanStack Query，**禁止裸 `fetch` 或 `axios`**
- **禁止引入 Zod**（strict CSP `no-unsafe-eval` 限制，見 CLAUDE.md §5）
- 元件檔 ≤ 300 行，JSX return ≤ 80 行
- 禁止將 custom hook 回傳的整個物件放入 `useEffect` / `useCallback` deps

### 6.4 Lint & Format 指令

```bash
# Backend
cargo fmt
cargo clippy --all-targets -- -D warnings -A deprecated

# Frontend
cd frontend && pnpm lint
```

---

## 7. 文件更新政策

### 7.1 各檔案職責（完整版見 CLAUDE.md §文件記錄規則）

| 檔案 | 更新時機 | 禁止事項 |
|------|----------|----------|
| `docs/TODO.md`嚗?冽?隞塚? | 完成任務 → 標 `[x]`；新任務 → 對應 section 補充 | 變更日誌（放 PROGRESS.md） |
| `docs/PROGRESS.md §9`嚗?冽?隞塚? | 每次完成有意義的工作後新增一筆 | 任務狀態追蹤 |
| `DESIGN.md Decisions Log` | 設計 / 架構決策 | 任務、進度 |
| `docs/glp/traceability-matrix.md`嚗?冽?隞塚? | 新增或修改合規功能 | — |
| `docs/spec/`嚗?冽?隞塚? | 新增模組或修改 API contract | — |

### 7.2 文件撰寫原則

- **受眾優先**：先確認「這是給誰看的」（見 CLAUDE.md §交付物受眾定義）
- **反向時間序**：時間性條目新的放最上面
- **單一職責**：每個檔案只做一件事
- **語言**：UI / 技術文件用繁體中文；程式碼識別字 / commit message 保持英文

---

## 8. 審查流程

### 8.1 Reviewer 職責

- 確認 PR 描述清楚、變更範圍符合 issue
- 確認關鍵路徑有測試覆蓋（非追求 100% coverage）
- 合規相關變更確認 traceability-matrix.md 已更新
- 意見說明**為什麼**，而非只說「請改成 X」

### 8.2 Author 回應原則

- 48 小時內回覆 reviewer 意見
- Resolve conversation 前確保已處理，或說明不處理的原因
- 不強行 dismiss review（除非 reviewer 已明確同意）

### 8.3 Stale PR 處理

- 超過 14 天未更新的 PR 加上 `stale` 標籤
- 超過 30 天無回應則由 maintainer 關閉（可重開）

---

## 9. 合規開發要求

本系統須符合 **GLP**（良好實驗室規範）與 **21 CFR Part 11**（電子記錄與簽章），所有涉及資料 mutation 的功能開發必須遵守：

### 9.1 Audit Trail（稽核追蹤）

所有 mutation（新增 / 修改 / 刪除）必須透過 Service-driven Audit Pattern（R26）：

```rust
// 正確：在 service 層呼叫，傳入 transaction
AuditService::log_activity_tx(&mut tx, actor, event_type, resource_type, resource_id, diff).await?;
```

Actor 類型：

| 情境 | Actor |
|------|-------|
| HTTP request（已登入） | `ActorContext::User(CurrentUser)` |
| Scheduler / CLI 工具 | `ActorContext::System { reason }` |
| 登入前匿名事件 | `ActorContext::Anonymous` |

### 9.2 Electronic Signature（電子簽章）

涉及「作者確認」「審查」「核准」「責任」的操作須要求密碼 + TOTP 2FA 重新驗證。
簽章寫入後，觸發 `glp_record_locks` 紀錄，DB trigger 拒絕後續 UPDATE。

### 9.3 Traceability（追溯性）

每次新增或修改合規功能後，更新 `docs/glp/traceability-matrix.md`嚗?冽?隞塚?：

```text
| 條款 | 功能描述 | Migration | Service | Handler | Test |
```

完整合規需求詳見：
- docs/glp/traceability-matrix.md嚗?冽?隞塚?
- docs/glp/R26_compliance_requirements.md嚗?冽?隞塚?
- docs/spec/guides/AUDIT_LOGGING.md嚗?冽?隞塚?

---

## 10. AI Agent 協作規則

本專案使用 [Claude Code](https://claude.ai/code) 進行 AI 輔助開發。

### 10.1 AI 可自決的範圍

- 低風險、可逆操作（檔名選擇、變數命名、helper 抽取）
- 文件撰寫與格式調整

### 10.2 AI 必須停下確認的操作

- DB Schema migration
- API contract 變更
- 新增 / 移除 dependency（Cargo.toml / package.json）
- CI/CD 設定修改
- `git push`、`git reset --hard`、PR merge
- 任何操作到 staging / production DB

### 10.3 審查 AI 產出的 PR 要點

- Commit message 是否有意義且符合本規範？
- 變更範圍是否超出 issue 描述？（警惕 drive-by improvements）
- 合規 checklist 是否已確認？
- 有無未被要求的順帶修改？

### 10.4 AI Branch 命名

AI agent 使用 `claude/<slug>` 分支，PR 標題加 `[AI]` 前綴以便識別與追蹤。

---

## 11. 緊急 Hotfix 流程

1. 自 `main` 開 `hotfix/<slug>` 分支
2. 最小化修復範圍，附 reproducing test
3. 通知所有相關 reviewer（即時訊息 + @mention）
4. 至少 1 位 reviewer 批准後 merge（可事後補完整審查）
5. 若有進行中的長期分支，立即 cherry-pick 回去

---

## 問題 / 建議

- 開 [GitHub Issue](https://github.com/delightening/ipig_system/issues/new/choose)（選擇對應 template）
- 緊急問題聯繫 CODEOWNERS（見 [.github/CODEOWNERS](.github/CODEOWNERS)）
- 資安漏洞：[GitHub Security Advisories](https://github.com/delightening/ipig_system/security/advisories/new)
