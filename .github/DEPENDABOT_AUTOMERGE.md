# Dependabot 自動合併：設計、上線步驟與交接

> 建立日期：2026-10-27。適用範圍：本 repo（Rust 後端維護期）。
> 後端改寫為 Go 後，請照〈§7 Go 遷移時要重設的項目〉重新設定一次。

## 1. 它做什麼、不做什麼

Dependabot 開出依賴更新 PR 後，若更新「夠小、夠常規」，就替它啟用 GitHub 的 auto-merge；
**真正的合併由 GitHub 在 required status checks 全綠後執行**。其餘（major、0.x minor、
Docker base image、GitHub Actions、人工清單內的套件）一律照舊，等人看。

```
Dependabot 開 PR
   │
   ├─ CI 照常跑（ci.yml，不變）
   │
   └─ dependabot-automerge.yml（pull_request_target，不 checkout PR 程式碼）
        ├─ fetch-metadata 讀出「這個 PR 更新了哪些套件、哪一級」
        ├─ 全部符合政策？ ── 否 ─→ 不排隊，PR 留給人（Actions 摘要寫明原因）
        └─ 是 ─→ gh pr merge --auto --squash
                      │
                      └─ required checks 全綠 → GitHub 自動 squash 合併
                         任一紅燈 → 不合併，PR 停著
```

**安全性的上限 = required checks 的可信度。** 本機制不做任何額外測試，只是把「人按合併鍵」
換成「CI 全綠就自動按」。CI 沒抓到的壞更新，會直接進 main。

> ℹ️ **Dependabot PR 的 E2E（2026-10-27 補上，尚未實測）。** 原本 `ci.yml` 的
> `🧪 E2E: Playwright` 在 `github.actor == 'dependabot[bot]'` 時整個 job 被 skip，而 skip
> 視為通過，等於 required check 對 Dependabot PR 沒有保護。原先的理由是「拿不到 secrets，
> Docker 環境起不來」，但 `docker-compose.test.yml` 與 Playwright 步驟的密碼都有 fallback
> （secret 為空時兩端落到同一個值），所以已移除該排除條件。
> **上線前必須確認：** 第一個 Dependabot PR 上 `🧪 E2E: Playwright` 要真的執行並通過
> （不是 skipped）。若失敗，先看是否為 secrets 以外的原因；無法修就把排除條件加回，
> 並把此 check 從 required checks 移除（避免誤以為受保護）。

## 2. 檔案清單

| 檔案 | 作用 |
|---|---|
| `.github/workflows/dependabot-automerge.yml` | 資格判定 + 啟用 auto-merge。政策旋鈕（生態系白名單、人工清單）在 `decide` 步驟的 `env`。 |
| `.github/dependabot.yml` | 既有；本次只在 cargo / npm / pip 加 `cooldown: default-days: 7`。 |
| `.github/scripts/setup-branch-protection.sh` | 一鍵開啟 repo 的 auto-merge、建立 main 的 required-checks ruleset、（可選）打開 kill switch。預設 dry-run。 |
| `.github/DEPENDABOT_AUTOMERGE.md` | 本文件。 |

## 3. 自動合併的政策（誰能排隊、誰不行）

PR 內**每一個**被更新的套件都要同時滿足，整個 PR 才會排隊（group PR 中有一個不合格，整包留給人）：

| 條件 | 規則 | 理由 |
|---|---|---|
| 生態系 | 只有 `cargo`、`npm_and_yarn`、`pip` | `docker`：base image 變動 = 正式環境 runtime 變動，CI 驗不了行為。`github_actions`：Actions 在 CD 持有寫入權限，SHA pin 的本意就是版本更新要經人眼；月更一次，量小。 |
| 更新級別 | `patch`；或 `minor` 且舊版**不是 0.x** | 0.x 的 minor 依 semver 慣例可含破壞性變更（Cargo 生態很常見）。 |
| 無法判讀 | `updateType` 空、舊版未知 → 不排隊 | allowlist 思維：不確定就交給人。 |
| 人工清單 | `playwright`、`mcr.microsoft.com/playwright*` | requirements.txt 與 Dockerfile base image tag 必須同版，見 `dependabot.yml` 的 pip `ignore` 註解（2026-08-06 prod 事故）。 |
| 冷卻期 | 新版本滿 7 天才開 PR（security updates 不受限） | 給供應鏈攻擊（惡意版本、帳號被盜）被發現、下架的時間。 |

判定邏輯已用 12 個案例驗證（patch、0.x minor、1.x minor、group 含 major、docker、actions、
playwright、空 updateType、舊版未知、`v` 前綴 0.x、空陣列、全 patch 的 group），結果都符合預期。

## 4. 上線步驟（尚未執行；依序做，不要跳）

前提：`gh` 要以 repo 擁有者 **delightening** 登入。本機 `gh` 預設可能是別的帳號，腳本會先驗身分，
不符就拒絕執行。

1. **確認 Dependabot secrets**：Settings → Secrets and variables → **Dependabot** 必須有
   `CI_ADMIN_PASSWORD`（`ci.yml` 的 cargo test 與 coverage 會讀）。Dependabot PR 讀不到 Actions
   那一側的 secrets；缺的話 Dependabot PR 的後端測試會在沒有管理員密碼的情況下跑，結果不可信。
2. **合併本 PR**（人工）。此時機制仍是**關閉**的（variable 未設）。
3. **dry-run 看腳本會做什麼**：
   ```bash
   bash .github/scripts/setup-branch-protection.sh
   ```
4. **套用設定（kill switch 仍關）**：
   ```bash
   gh auth switch -u delightening
   bash .github/scripts/setup-branch-protection.sh --apply
   ```
5. **驗證 ruleset 沒有擋掉正常流程**：開一個無害的 PR（例如改一行註解），確認 required checks
   會出現且能轉綠、能合併。接著**看 main 上第一次 push 的 CI**，確認 coverage ratchet 步驟沒有被拒
   （見〈已知風險 2〉）。
   ⚠️ 無害 PR 可能讓覆蓋率不變，此時 ratchet 步驟不會 push、也會是綠燈，**這不能證明 bypass 有效**。
   必須用一個「會讓覆蓋率上升（觸發 baseline 更新）」的受控 PR 合併後，確認 main 上出現
   ratchet 的 `[skip ci]` commit，才算驗證通過。
6. **正式啟用**：
   ```bash
   bash .github/scripts/setup-branch-protection.sh --apply --enable
   ```
7. **觀察第一個符合資格的 patch PR**：Actions 的 `Dependabot auto-merge` 摘要應寫「已排入
   auto-merge」；PR 頁面出現 "Auto-merge enabled"；CI 全綠後自動合併。
8. **合併後檢查 main**：CI 與 CD 有沒有被觸發（見〈已知風險 1〉）。

## 5. 已知風險與待實測項目

1. **以 `GITHUB_TOKEN` 啟用的 auto-merge，合併後的 push 事件可能不會觸發 main 上的 workflow。**
   GitHub 的設計是「GITHUB_TOKEN 造成的事件不會觸發新的 workflow run」。若成立，Dependabot PR
   合併後 main 不會跑 CI、CD 也不會建映像，且 coverage ratchet 不會更新。
   - 目前正式環境是手動部署，影響偏小；但 main 上沒有完整 CI 紀錄，要意識到這件事。
   - 若實測成立且不能接受：改用 GitHub App token 或 fine-grained PAT 來啟用 auto-merge
     （存成 Dependabot secret，workflow 的 `GH_TOKEN` 改讀它）。
2. **coverage ratchet 會直接 push 回 main**（`ci.yml`：後端與前端各一處，`[skip ci]` commit）。
   一旦 main 有 required checks，這種直接 push 預設會被擋。腳本把 `github-actions`
   （Integration 15368）加入 ruleset 的 bypass 名單來處理，**但這個 bypass 對 `GITHUB_TOKEN`
   是否生效尚未實測。**
   - 症狀：main 上 CI 的 ratchet 步驟出現 `GH013` / "protected branch" 之類的拒絕訊息。
   - 對策：改成 ratchet 開 PR 而非直接 push；或改用具 bypass 資格的 App token；
     或暫時把 `backend-coverage-ratchet` 的 push 步驟關掉，改為手動調整 baseline。
3. **required check 名稱綁死 `ci.yml` 的 job `name:`。** 改 job 名稱 = required check 永遠等不到
   = 所有 PR 卡死。改名時同步改腳本的 `CHECKS` 陣列並重跑 `--apply`。
4. **被 skip 的 job 視為通過。** 前端 only 的 PR 會 skip 後端 job，這是預期；但也代表
   `needs` 的上游失敗時，下游 job 變 skip——只要上游本身是 required，就仍會擋住。
5. **Dependabot rebase 會重跑整輪 CI**（cargo test 單項約 25–30 分鐘）。同一時間大量 PR 排隊時
   runner 會塞；`dependabot.yml` 的 groups 與 `open-pull-requests-limit` 就是用來壓這個量。
6. **CODEOWNERS 的擁有者字串要和實際帳號對得上。** 目前檔案寫的是 `@jasonwang`，而合併 PR 的
   帳號是 `delightening`。若日後在 ruleset 加上 "Require review from Code Owners"，
   帳號不符會讓 `.github/` 相關 PR 永遠等不到審查。本機制**沒有**啟用該選項。
7. **供應鏈風險無法歸零。** 冷卻期、major 手動、CI 掃描（cargo audit / deny、OSV、gitleaks、
   Trivy）降低風險，但新版本的惡意程式若不會被這些掃描抓到，仍會進 main。維護期的
   Rust 後端若不再需要新版本，更保守的做法見〈§6〉。

## 6. 維護期的建議（Rust 後端只維護、不新增功能）

- 後端凍結後，cargo 可只留**安全更新**：`dependabot.yml` 的 cargo 段設
  `open-pull-requests-limit: 0`（只停掉版本更新，security updates 仍會送）。
  少一條自動進 main 的路徑，比多一層自動化更安全。
- 前端與 print-pdf 的依賴仍在變動，自動合併的效益主要在這兩塊。

## 7. Go 遷移時要重設的項目

Go 版是另一個 repo（module `github.com/delightening/ipig-system-go`），不會繼承本 repo 的設定。
上線時逐項重做：

| 項目 | 動作 |
|---|---|
| `dependabot.yml` | 新增 `gomod` 生態系；移除 `cargo`。加 `cooldown`。 |
| `dependabot-automerge.yml` | `github.repository == '...'` 守門字串改成新 repo；`ALLOWED_ECOSYSTEMS` 把 `cargo` 換成 `gomod`。 |
| Go 的 major | Go modules 的 major 版本是 import path 的一部分（`/v2`），Dependabot 會怎麼標記 `updateType` 要用第一個實際 PR 驗證，不要假設。 |
| required checks | 新 repo 的 CI job 名稱不同，重寫腳本的 `CHECKS` 陣列；建議納入 `govulncheck`。 |
| coverage ratchet | 若新 CI 也有「直接 push 回 main」的步驟，重新評估〈已知風險 2〉。 |
| Dependabot secrets | 新 repo 的 Dependabot secrets 要重新建立。 |
| kill switch | 新 repo 自己的 `DEPENDABOT_AUTOMERGE` variable，預設關，驗證完再開。 |

## 8. 緊急停止

| 情境 | 做法 |
|---|---|
| 發現自動合併了不該合的東西 | 依序：① Settings → Secrets and variables → Actions → Variables，把 `DEPENDABOT_AUTOMERGE` 設為 `false`（或刪除）。② 取消已通過閘門、仍在跑的判定 run（變數只擋新 run，擋不住已進行中的）：`gh run list --workflow dependabot-automerge.yml --status in_progress --json databaseId --jq '.[].databaseId' \| xargs -n1 gh run cancel`（`queued` 狀態同理）。③ 再處理下一列已排隊的 PR。 |
| 已排隊、還沒合併的 PR | 在上一列的 ② 完成後執行 `gh pr merge --disable-auto <PR 編號>`；否則進行中的 run 可能在你取消排隊後又重新啟用。 |
| 已合併的壞更新 | 在 GitHub 上對該 squash commit 按 Revert；之後把該套件加進 workflow 的 `MANUAL_ONLY`，或在 `dependabot.yml` 加 `ignore`（寫明原因與解除條件，格式比照既有條目）。 |
| required checks 把正常流程卡死 | Settings → Rules → Rulesets → `main-required-checks` → 將 Enforcement 改為 Disabled。修好後再改回 Active。 |

## 9. 設計取捨紀錄（為什麼這樣做）

- **反方論點**：自動合併等於讓外部套件的任意版本不經人審進 main，供應鏈攻擊正是這樣擴散的。
- **回應**：只放行 patch / 非 0.x minor、擋掉 Docker 與 Actions、冷卻 7 天、CI 含依賴弱點掃描，
  並預設關閉、有 kill switch。這把風險壓低，**不是歸零**。
- **結論**：風險可接受的前提是 required checks 可信，所以上線順序是先建 ruleset、驗證，最後才開 kill switch。
- **沒有加「自動核准」步驟**：ruleset 沒有要求 approving review；若日後加上，注意 `github-actions[bot]`
  預設不能核准 PR（需另外開啟設定），且單人維護的 repo 通常不需要。
- **沒有改用 Dependabot 內建的 "auto-merge" 設定**：Dependabot 本身沒有這個旋鈕；官方做法就是
  workflow + `gh pr merge --auto`。

## 10. 目前未涵蓋的缺口（盤點時發現，未在本次處理）

- repo 根目錄有 `package.json` / `package-lock.json`（husky 等工具）與 `pyproject.toml` / `uv.lock`，
  `dependabot.yml` 沒有對應的 `npm`（directory `/`）與 `uv` 項目，這兩塊的依賴目前不會收到更新 PR。
