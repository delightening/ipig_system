# Dependabot PR 分流清單

> 草稿日期：2026-10-28。搭配 [`DEPENDABOT_AUTOMERGE.md`](./DEPENDABOT_AUTOMERGE.md) 閱讀。
> 範圍：本 repo 的 Dependabot PR（cargo / npm / pip / docker / github-actions）。

## 0. 這份文件解決什麼問題

`DEPENDABOT_AUTOMERGE.md` 定義的是「**哪些 PR 可以自動合併**」。但實務上，
docker、GitHub Actions、major、0.x minor、playwright 都被刻意排除在自動合併之外
（2026-10-28 當天的 4 個 Dependabot PR 全部落在這個範圍），所以**人工分流才是主要路徑**。

人工分流過去沒有固定步驟，品質取決於當下有沒有想到。已發生的後果：

- #177（playwright base image 1.64.0→1.64.1）：requirements.txt 沒同步，靠 CI 紅燈才發現。
  `dependabot.yml` 寫了「人工同步」，但沒有任何機制提醒。
- #179（password-hash 0.5→0.6）：編譯就失敗，且沒有「何時可升」的紀錄，
  Dependabot 之後可能再開同類 PR。

CI 全綠是**必要條件，不是充分條件**：CI 回答「現在有沒有壞」，
回答不了「這次升級值不值得、上游準備好了沒、我們有沒有暴露在漏洞下」。

## 1. 五個步驟

### 步驟 1：辨識

- [ ] 確認 `gh` 以 **delightening** 登入（`gh api user --jq .login`）。
- [ ] 這是**安全更新**還是**一般版本更新**？本 repo 沒有 `security` 標籤，group PR 的描述也不一定
      提到漏洞，所以**不要靠標籤或描述判斷**。改查 Dependabot alerts，用套件名與修補版本比對：
      `gh api "repos/delightening/ipig_system/dependabot/alerts?state=open" --jq '.[] | {pkg:.dependency.package.name, sev:.security_advisory.severity, ghsa:.security_advisory.ghsa_id, patched:.security_vulnerability.first_patched_version.identifier}'`
      PR 的目標版本 ≥ `patched` 即為修補該 alert。安全更新不受 7 天冷卻期限制，優先序較高。
- [ ] 生態系與更新級別（patch / minor / major；舊版是否 0.x）。
- [ ] 是否落在 `dependabot-automerge.yml` 的自動合併資格內？
      是 → 等 auto-merge（機制上線後）；否 → 繼續步驟 2。

### 步驟 2：現況分析——我們真的需要這個更新嗎？

- [ ] 若是安全更新：advisory 編號、嚴重度（CVSS）、**修補版本**是否就是這個 PR 的版本。
- [ ] **我們有沒有暴露？** 漏洞路徑是否會被我們的程式碼走到（grep 實際用法、輸入是否來自外部）。
- [ ] 若是一般版本更新且沒有漏洞：沒有急迫性，可以等下一個週期或上游穩定後再升。
- [ ] 結論**寫下來**。有 advisory 的依賴，在 `requirements.txt` / `Cargo.toml` 旁寫註解
      （範本：`services/print-pdf/requirements.txt` 的 pypdf 段落——advisory 編號、CVSS、
      是否受影響、為何仍升版）。

### 步驟 3：評估代價與風險

**3a. 版本風險**

- [ ] major、0.x minor：依 semver 可能含破壞性變更，視為高風險，必須讀 changelog。
- [ ] 讀 release notes / compare，列出**對我們有影響的變更**。

**3b. 上游是否準備好**

- [ ] 相依的其他套件是否已支援（例：升 `password-hash` 前確認 `argon2` 是否支援同一版；
      升 `typescript` 前確認 `typescript-eslint` 是否支援）。
- [ ] 沒準備好 → 不升，進步驟 5 並寫入 `ignore`。

**3c. 本 repo 的特殊規則**（違反過就出過事）

| 類型 | 規則 | 依據 |
|---|---|---|
| `playwright`（docker base image） | `services/print-pdf/requirements.txt` 的 playwright 必須與 Dockerfile base image tag **同版**，同一個 PR 內一起改 | 2026-08-06 prod 事故；#177 |
| `postgres` major | 必須與 prod DB 主版本一致，或先跑 `scripts/backup/dr_drill.sh` 通過 | `dependabot.yml` 的 ignore 註解 |
| `typescript` major | 等 `typescript-eslint` 支援 | `dependabot.yml` 的 ignore 註解 |
| GitHub Actions（SHA pin） | 確認新 SHA 是上游某個 tag／分支的**目前 commit**；用 `compare` 看改了哪些檔案 | 供應鏈強化（SHA pin 的本意是版本更新要經人眼） |
| 密碼、加密、驗證相關 crate | 除了編譯與測試通過，確認**舊資料仍可驗證**（例如既有雜湊登入）；沒有此類測試就先補 | 密碼雜湊路徑，CI 不一定涵蓋 |
| docker base image | runtime 變更，CI 只驗得起來、驗不了行為；看 smoke test 與 E2E 是否真的執行（不是 skipped） | `DEPENDABOT_AUTOMERGE.md` §3 |

**3d. CI 結果**

- [ ] 全部檢查完成（沒有進行中）、沒有失敗；被跳過的項目要確認是設計如此，不是被意外排除。
- [ ] 失敗時**先讀日誌**再判斷原因，不要憑檢查名稱猜。
- [ ] 查 CI 時查一次就好，還在跑就等，不掛輪詢迴圈。

### 步驟 4：全部安全 → 升級並合併

滿足下列全部條件才合併：

- [ ] 步驟 2、3 的結論都已寫下，且沒有未解決的疑慮。
- [ ] CI 全綠。
- [ ] 需要同步修改其他檔案的（如 playwright → requirements.txt），已在同一個 PR 內完成。
- [ ] 以 squash 合併（與 auto-merge 一致，維持一個 PR 一個 commit）。
- [ ] 合併後確認 main 的 CI 有跑（見 `DEPENDABOT_AUTOMERGE.md` §5 已知風險 1）。

### 步驟 5：任何一項不安全 → 與 Jason 討論

不要自行決定，整理成一份簡短摘要交給 Jason：

- PR 編號與標題
- 為什麼不能直接合（失敗的檢查、上游未就緒、有風險的變更）
- 選項與各自代價（例如：關閉並等上游／自己適配新 API／先觀察）
- 建議哪一個，以及理由

**決定「不升」時，做兩件事（缺一不可）：**

1. 關閉 PR，留言說明原因。
2. 在 `dependabot.yml` 加 `ignore`，寫明**原因與何時可以升**（格式比照既有的 typescript、postgres 條目）。
   只關 PR 擋不住——typescript 7 的 PR 就是被關了兩次又復活（見 `dependabot.yml` 註解）。

## 2. 責任分工

| 動作 | 誰做 |
|---|---|
| 步驟 1–3 的調查與摘要 | AI 助手可代勞 |
| 合併、關閉 PR、推送到 Dependabot 分支、修改 `dependabot.yml` | **需 Jason 明確同意**（對外寫入） |
| 步驟 5 的決策 | Jason |

## 3. 案例（2026-10-28）

| PR | 內容 | 結果 | 該回頭補的環節 |
|---|---|---|---|
| #177 | playwright/python 1.64.0→1.64.1（docker） | CI 紅（版本一致性）。在 Dependabot 分支補 requirements.txt 後轉綠，已合併 | 步驟 3c：同版規則 |
| #178 | docker/login-action 4.6.0→4.7.0（Actions） | CI 全綠，已合併 | — |
| #179 | password-hash 0.5.0→0.6.1（cargo，0.x minor） | `cargo check` 即失敗，已關閉 | 步驟 3b：上游（argon2）未驗證就關，**尚未寫 ignore** |
| #181 | dtolnay/rust-toolchain SHA bump（Actions） | CI 全綠；新 SHA 即上游 `v1` 分支目前 commit，差 12 個 commit | 步驟 3c：SHA 來源 |

## 4. 待辦（本文件之外）

- `#179`：確認 `argon2` 是否已支援 password-hash 0.6；未就緒則補 `ignore` 條目，寫明解除條件。
- 根目錄 `package.json`／`uv.lock` 目前沒有納入 `dependabot.yml`（見 `DEPENDABOT_AUTOMERGE.md` §10）。
- 自動合併機制上線後，本文件的步驟 1 應補上「機制是否啟用」的判斷。
