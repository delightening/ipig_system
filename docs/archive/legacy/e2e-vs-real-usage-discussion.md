# E2E 與真實操作模式對照與修改計畫

## 1. 目前 E2E 的運作方式

| 階段 | 行為 |
|------|------|
| **auth-setup** | 先後以「user」與「admin」帳號各登入一次，把 cookie 存成 `user.json`、`admin.json` |
| **多數 spec** | 使用 **admin** 的 storageState（從檔案載入同一份 cookie），每個 test 是**新的 browser context**，但 cookie 相同 |
| **login.spec** | 使用**空** storage（無 cookie），在測試裡再執行一次真實登入 |
| **執行順序** | 依專案/檔案順序：admin-users → animals → dashboard → login → profile → protocols |

也就是說：  
- 除了「登入流程」那組是「真的登入一次」以外，其他都是「載入同一份已登入的 cookie」再操作。  
- 同一份 admin cookie 會被多個 test 重複使用（每個 test 新 context，但 cookie 一樣）。

---

## 2. 真實使用者操作模式

- 使用者**一次登入**，之後在同一個 session 裡：點動物、Dashboard、個人資料、計畫書等。
- Session 一直用到登出或逾時，中間不會「從檔案重新載入 cookie」。
- 不會在「同一個使用者」下，同時存在「已存檔的 session」與「剛剛才登入的 session」並行使用。

所以：  
- **真實**：1 次登入 → 很多次請求，同一個活 session。  
- **E2E**：1 次登入存成檔案 → 很多個 test 各自用這份 cookie 開新 context，且中間還有一個 test 會再登入一次（login.spec）。

---

## 3. 差異與可能問題

### 3.1 Session 失效（目前 3 個失敗）

- **現象**：約第 12、13、17 個 test（動物 Tab、語言切換）時，請求被導向登入頁。  
- **可能原因**（需後端確認）：  
  - Session TTL 較短（例如 1 分鐘），跑完前面 11 個 test 已逾時。  
  - 或「同一 session_id 的請求次數/頻率」有上限或特殊處理。  
- **與真實的關係**：  
  - 真實使用通常是「登入後連續操作」，不會在幾十秒內重複建立那麼多「新 context、同一 cookie」的請求。  
  - 所以 E2E 對 session 的「使用方式」比真實更苛刻，若後端對 TTL 或請求數很敏感，就會在 E2E 先爆。

結論：**E2E 有發現「session 在連續多請求下會失效」的風險，但觸發方式與真實不完全相同**。要讓 E2E 穩定通過，通常需要後端配合（例如在 E2E 環境放長 TTL 或放寬限制）。

### 3.2 登入測試與其他測試的 session 關係

- **E2E**：  
  - 前面很多 test 用「auth-setup 存的 admin」。  
  - 中間有一個 test（成功登入應導向 dashboard）是「空 cookie → 再登入同一個 admin」。  
- 若後端是 **單一 session  per user**（同一帳號只允許一個有效 session）：  
  - 登入 test 的那次登入，可能會讓「auth-setup 存下來的那個 admin session」失效。  
  - 之後再用 admin.json 的 test 就會被導向登入頁。  
- **真實**：同一使用者不會在「已登入」的同一瀏覽器裡再登入一次，所以不會觸發「第二個登入踢掉第一個」的情況。

結論：**若後端是單 session per user，目前 E2E 的「登入 test」與「使用 admin 的 test」混跑，可能與真實使用不一致，且容易造成後續 test 失敗。** 這部分可以靠「測試設計」調整，不一定改產品程式碼。

### 3.3 多 worker 並行

- **E2E**：多 worker 時，多個 context 同時用同一份 admin.json，等於多個「同一帳號」的 session 並行。  
- **真實**：通常一個使用者一個瀏覽器，不會同帳號多個 tab 同時打很多並行請求。  
- 若後端對「同帳號多 session」或並行有限制，E2E 會比真實更容易觸發 429/401。

結論：**用 `--workers=1` 跑 E2E，較接近「單一使用者連續操作」，與真實較一致。**

---

## 4. E2E 是否與真實操作吻合？

| 面向 | 吻合度 | 說明 |
|------|--------|------|
| **單次操作** | ✅ 高 | 每個 test 的「點什麼、填什麼、預期什麼」與真實操作一致。 |
| **登入後連續逛多頁** | ⚠️ 部分 | 邏輯上是一登入後連續操作，但 E2E 是「多個 context 共用同一 cookie」，且中間有「再登入一次」的 test，與真實「一次登入用到底」不盡相同。 |
| **Session 生命週期** | ❌ 低 | 真實是「一個活 session 用很久」；E2E 是「同一份 cookie 被多個 context 重複使用」，若後端 TTL 或單 session 限制較嚴，E2E 會先失效。 |
| **並行** | ❌ 低 | 多 worker 時同帳號並行，真實少見；改 1 worker 較貼近真實。 |

整體：  
- **單一操作與流程**：E2E 有對齊真實（點擊、表單、導向、權限）。  
- **Session 與並行**：E2E 目前較「嚴苛」或「不同於真實」，需要透過設定或測試設計彌補，而不是改產品邏輯去配合 E2E。

---

## 5. 修改計畫（若需要）

### A. 不需改產品程式碼、只調測試/環境（建議先做）

1. **執行方式**  
   - 本機/CI 一律用 `--workers=1` 跑 E2E，讓「同一帳號、同一 session」的用法更接近真實。

2. **登入 test 與 session 隔離**  
   - 選項一：把「成功登入應導向 dashboard」移到**最後**跑（例如獨立 project，dependency 在其它 E2E 之後），避免同一 run 內「再次登入」踢掉 auth-setup 的 admin session。  
   - 選項二：登入 test 改用與 admin **不同**的測試帳號（若後端允許多帳號），就不會影響 admin session。  
   - 這樣可避免「E2E 因第二次登入而失效」與真實不符的問題。

3. **後端 session 設定（若可調）**  
   - 在 E2E 或 dev 環境：適度拉長 session TTL（例如 ≥ 5 分鐘），讓 34 個 test 在 1 worker 下能一次跑完。  
   - 若後端有「同帳號單一 session」：確認 E2E 的「登入 test」不會在跑其它 test 前就執行，或改用獨立帳號。

### B. 可選的程式碼/設定變更（依你們後端架構決定）

| 項目 | 改動 | 目的 |
|------|------|------|
| **Session TTL** | 後端設定（config/env） | E2E 環境拉長 TTL，讓 1 worker 跑完全部 test 不逾時。 |
| **E2E 專用 config** | 後端或前端：E2E 時放寬 rate limit / 單 session 限制（若有） | 減少 429 或「多 context 同 cookie」造成的失效。 |
| **語言選擇器** | 已做 | 在 MainLayout 加 `data-testid="language-selector"`，E2E 穩定找到元素。 |

### C. 不建議的改動

- **為通過 E2E 而放寬「正式環境」的 session 或安全設定**：E2E 應盡量用環境/參數區隔，而不是改產品預設行為。  
- **在產品程式碼裡加「若是 E2E 就怎樣」的邏輯**：盡量用 config/env 區分環境即可。

---

## 6. 建議的下一步（討論用）

1. **確認後端 session 行為**  
   - Session 逾時時間（TTL）？  
   - 是否「一帳號一 session」？若是，登入 test 跑完後，其它用 admin 的 test 是否預期會失效？  

2. **決定 E2E 執行策略**  
   - 是否接受「本機/CI 一律 `--workers=1`」？  
   - 是否要調整「登入 test」順序或改用獨立帳號，避免踢掉 admin session？  

3. **是否調整後端設定**  
   - 僅在 E2E/dev 環境拉長 TTL 或放寬限制，讓目前 3 個失敗的 test 穩定通過，而不改產品邏輯。  

若你提供後端 session 的實作方式（例如 TTL、單 session 與否），可以再對應寫出更具體的「要改哪個檔、哪個參數」的修改計畫。

---

## 7. 已實作與遺漏檢查（2026-02 更新）

### 7.1 已實作

| 項目 | 實作內容 |
|------|----------|
| **workers=1 固定** | `playwright.config.ts`：`workers: 1`、`fullyParallel: false`，本機與 CI 皆單 worker。 |
| **登入 test 最後跑** | Chromium / Firefox / WebKit 皆拆成「主 project + -login」：主 project 的 `testIgnore` 含 `login.spec.ts`，`*-login` 僅跑 `login.spec.ts` 且依賴主 project。完整單一瀏覽器：`--project=chromium-login`、`--project=firefox-login`、`--project=webkit-login`。 |
| **後端 JWT TTL 說明** | `backend/env.sample` 改為 `JWT_EXPIRATION_MINUTES=15` 並註解「E2E/本機開發建議 ≥5」；`docs/QUICK_START.md` 補充 JWT 與 E2E 說明。 |
| **程式碼** | 語言選擇器已加 `data-testid="language-selector"`，無額外產品程式變更。 |

### 7.2 後端 Session 結論（依調閱）

- **TTL**：由 `JWT_EXPIRATION_MINUTES` 控制（config 預設 15 分鐘），JWT access token 逾時即失效。E2E 全量約 1.5–2 分鐘，預設 15 分鐘足夠；若曾改短，建議 E2E/dev 設為 ≥5。
- **一帳號一 session**：**否**（已確認），故登入 test 不會因「第二個登入」踢掉其他 test 的 session；挪到最後跑主要為貼近真實（先逛再測登入）並避免邊界情況。

### 7.3 遺漏檢查

| 檢查項 | 狀態 |
|--------|------|
| CI 是否固定 workers=1 | ✅ 已由 config 統一為 `workers: 1`，CI 無需再傳參。 |
| CI 是否需改指令 | ✅ 不需。CI 執行 `npm run test:e2e`（即 `playwright test`）會跑所有 project：auth-setup → chromium → chromium-login → firefox → firefox-login → webkit → webkit-login，各瀏覽器皆為登入最後跑。 |
| **實際跑一次結果** | 執行 `npx playwright test --project=chromium-login`：**24 通過、4 失敗、6 未執行**。失敗的 4 個皆為「被導向登入頁」：動物列表 切換/欄舍、Dashboard 語言切換、個人資料 應顯示個人資料頁面。與文件 §3.1 的 session 失效一致；若後端 `JWT_EXPIRATION_MINUTES` 已 ≥5 仍失敗，可再查 cookie/domain 或前端重導邏輯。 |
| 跑完整 Chromium 時是否含登入 test | ✅ 使用 `--project=chromium-login` 會先跑 chromium（28 個）再跑 chromium-login（6 個），共 34 個。 |
| firefox / webkit 是否也要登入最後跑 | ✅ 已做。firefox、webkit 皆拆成主 project + -login（testIgnore login.spec.ts；firefox-login、webkit-login 僅跑 login.spec.ts），與 Chromium 一致。 |
| .env 已有 JWT_EXPIRATION_MINUTES 時是否需改 | 不必。預設 15 分鐘已足夠；僅在曾改短或 E2E 仍失敗時檢查 ≥5。 |
| 文件與指令一致性 | ✅ QUICK_START 已改為建議 `npx playwright test --project=chromium-login` 與 JWT 說明。 |

### 7.4 E2E 四項設定一致性檢查

針對「同源、JWT、Cookie、6 未執行」四項的逐項對照已寫入 **[e2e-four-items-check.md](./e2e-four-items-check.md)**，結論摘要如下：

| 項目 | 結論 |
|------|------|
| **1. 後端與前端同源** | ✅ 一致。E2E baseURL 為 `http://localhost:8080`，前端 API 用相對路徑 `/api`，docker / nginx 將 `/api` 代理到後端，瀏覽器僅與 8080 同源。 |
| **2. JWT_EXPIRATION_MINUTES** | ✅ 正確。根目錄 .env 為 15、CI 為 60，E2E 全量約 1.5–2 分鐘，TTL 足夠。 |
| **3. Cookie（COOKIE_SECURE / COOKIE_DOMAIN）** | ✅ 正確。.env 為 `COOKIE_SECURE=false`、未設 COOKIE_DOMAIN，與 `http://localhost` 搭配無誤。 |
| **4. 6 個登入 test 未執行** | ⚠️ 設定正確，但實際跑 `--project=chromium-login` 曾出現「6 did not run」。可單獨補跑：`npx playwright test --project=chromium-login e2e/login.spec.ts`，或拆成兩步（先 chromium、再 chromium-login）確保 34 個皆執行。 |

四項**未發現設定不一致或錯誤**；4 個失敗仍屬同一 session 在連續多 test 後被導向登入頁的既有現象。
