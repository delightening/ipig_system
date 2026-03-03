# CI 測試通過改進計畫

> 依據 `gh run view` 最新失敗紀錄 (Run #22612994678) 分析，並整合既有修復措施。

---

## 一、最新 CI 狀態摘要

| Job | 狀態 | 說明 |
|-----|------|------|
| Backend: cargo test | ❌ 失敗 (exit 101) | 測試執行階段錯誤 |
| Frontend: tsc check | ✅ 通過 | Vitest 改為 `npx vitest run` 後已通過 |
| E2E: Playwright | ⏭ 未執行 | 依賴 backend + frontend，backend 失敗時被跳過 |
| 其他 (cargo audit, cargo-deny, clippy 等) | ✅ 通過 | 無需變更 |

---

## 二、已知失敗原因與修復對照

### 2.1 Backend: cargo test (exit 101)

| 問題 | 原因 | 修復 | 狀態 |
|------|------|------|------|
| **E0063: missing field `disable_csrf_for_tests`** | `config.rs` 測試模組的 `minimal_config()` 未包含新欄位 | 在 `minimal_config()` 中加入 `disable_csrf_for_tests: false` | ✅ 已套用 |
| **api_animals 403 Forbidden** | CSRF 中介層阻擋 POST，整合測試未送 X-CSRF-Token | 1. 新增 `Config.disable_csrf_for_tests` 2. TestApp 設定 `DISABLE_CSRF_FOR_TESTS=true` 3. csrf middleware 檢查此旗標並略過 | ✅ 已套用 |

**相關檔案：**

- `backend/src/config.rs`：`disable_csrf_for_tests` 欄位、`from_env()`、`minimal_config()`
- `backend/src/middleware/csrf.rs`：`skip_csrf` 檢查
- `backend/tests/common/mod.rs`：`DISABLE_CSRF_FOR_TESTS=true`

### 2.2 Frontend: tsc check (歷史失敗)

| 問題 | 原因 | 修復 | 狀態 |
|------|------|------|------|
| **No projects matched filter "unit"** | Vitest `--project=unit` 與 `vitest.config.ts` project 結構不符 | CI 改為 `npx vitest run`（不指定 project） | ✅ 已套用 |

**相關檔案：**

- `.github/workflows/ci.yml`：`單元測試 (Vitest)` 步驟

---

## 三、驗證清單

推送前請確認：

1. [ ] `backend/src/config.rs` 包含 `disable_csrf_for_tests`（含 `from_env` 與 `minimal_config`）
2. [ ] `backend/tests/common/mod.rs` 設有 `DISABLE_CSRF_FOR_TESTS=true`
3. [ ] `backend/src/middleware/csrf.rs` 有 `skip_csrf` 邏輯
4. [ ] `.github/workflows/ci.yml` 單元測試為 `npx vitest run`（無 `--project=unit`）
5. [ ] 本機 `cargo test` 可通過（需 PostgreSQL + `DATABASE_URL`）
6. [ ] 本機 `npx vitest run` 可通過（前端目錄）

---

## 四、建議執行步驟

```bash
# 1. 確認所有修改已提交
git status

# 2. 推送到遠端
git add backend/src/config.rs backend/src/middleware/csrf.rs backend/tests/common/mod.rs .github/workflows/ci.yml
git commit -m "fix(ci): backend config + CSRF test exemption + Vitest command"
git push origin main

# 3. 觸發 CI 驗證
gh workflow run CI
# 或透過 PR 觸發
```

---

## 五、若仍有失敗時的排查方向

| 症狀 | 可能原因 | 對應動作 |
|------|----------|----------|
| 仍出現 `missing field disable_csrf_for_tests` | 修改未推送到遠端或分支錯誤 | 確認 `git push` 且 PR/分支正確 |
| `api_animals` 仍 403 | TestApp 未讀到 `DISABLE_CSRF_FOR_TESTS` | 確認 `common/mod.rs` 在 `spawn()` 時設定 env |
| Vitest 仍報 `No projects matched` | CI 仍使用舊 workflow | 確認 `ci.yml` 已更新並推送 |
| `cargo test` 編譯成功但測試失敗 | 其他整合測試失敗 | 執行 `gh run view <run_id> --log-failed` 檢視具體測試與堆疊 |

---

## 六、參考

- Run ID：`22612994678`
- 檢視失敗日誌：`gh run view 22612994678 --log-failed`
- 專案規則：`docs/CLAUDE.md`、`docs/TODO.md`
