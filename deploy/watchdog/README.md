# ipig-watchdog — 外部看門狗

## 1. 這東西為什麼存在

Prometheus、Alertmanager、Grafana、Loki 全部跑在 prod 筆電的 Docker 裡（`monitoring/`）。
它們把「系統內部發生什麼事」看得很清楚，但有一件事永遠看不到：**筆電本身掛掉**。
筆電掛的那一刻，Alertmanager 跟著掛，`backup_last_success_timestamp_seconds` 這條 25h 告警
永遠不會送出。在此之前，**能偵測「系統掛了」的唯一機制，跟系統掛在同一台機器上**。

這支 Worker 是唯一跑在筆電外面的那一層。它只有一個設計紅線：

> **通知路徑不得經過那台筆電。**
> 不可複用 Alertmanager 的 SMTP、不可複用 R22 `SecurityNotifier`，
> 否則就退化成自己監控自己，等於沒做。

因此告警走 Cloudflare Email Routing 的 `send_email` binding——同一個 Cloudflare 帳號、
不引入新廠商、與筆電零耦合。

## 2. 兩個職責

| # | 機制 | 抓什麼 |
|---|---|---|
| a | **主動探測**：cron 每 5 分鐘打 `/api/health`，連續 3 次失敗（＝15 分鐘）告警 | 筆電斷電、斷網、tunnel 掛、API 掛 |
| b | **被動心跳（dead man's switch）**：收 `POST /ping/<job>`，逾期未收到即告警 | **靜默失敗**——東西沒在跑，但外面看起來一切正常 |

(b) 才是真正補上缺口的那一半。2026-05-09 那次事故的形狀正是如此：`pg_backup.sh` 的
`DB_NAME` 預設值打錯（`erp_db` vs `ipig_db`），cron **數週靜默失敗**、`/backups/` 空無一物，
而網站好好的、主動探測全綠（見 `docs/runbooks/dr-drill-records.md` §5 問題 #1）。
心跳的邏輯是反過來的：**不是「壞了才通知」，是「好了才回報，該回報而沒回報就通知」。**

目前註冊的 job：

| job | 逾期門檻 | 誰送 ping |
|---|---|---|
| `backup` | 26 小時（cron 排 02:00 daily，給 2 小時寬限） | `scripts/backup/pg_backup.sh` 末端，全鏈成功才送 |

### 刻意不做的事

- **degraded 不告警**（HTTP 200 但 `status != "healthy"`）。那代表筆電還活著，內部
  Alertmanager 本來就看得到、管得比這裡細。在這裡重複一份只會變成兩邊都要維護的告警規則。
  **本 Worker 只管「還在不在」，不管「好不好」。**
- **不設第二收件人**（2026-08-03 使用者裁定）。要加時改 `ALERT_TO` 與 `send_email`
  binding 的 `destination_address`，兩個地方都要改，且新地址必須先在 Email Routing 驗證。

## 3. 首次部署

前置條件：`ipigsystem.asia` zone 的 Email Routing 已啟用。Cloudflare 帳號登入是
**下面流程的第一步**，不是事前準備——⚠️ 它必須排在 `npm ci` **之後**，理由見該步註解。

```bash
set -euo pipefail   # ← 與下面三段繞法同一道守衛。npm ci 失敗卻繼續往下跑，
                    #   等於帶著一個殘缺的 node_modules 去登入與部署。

cd deploy/watchdog
npm ci --include=dev   # 本目錄是獨立的 npm 專案，有自己的 package-lock.json。
                       # 🔴 --include=dev 不能省：wrangler 在 devDependencies，
                       #    而 NODE_ENV=production 時 npm ci 會跳過 devDependencies，
                       #    於是 node_modules/.bin/wrangler 根本不存在。
                       # 用 ci 不是 install 的理由見本節末尾。

# 0) 登入 Cloudflare。
#    🔴 **必須在 npm ci 之後**——乾淨 checkout 上先跑 `npx wrangler login` 的話，
#       本地還沒有 wrangler，npx 會從快取或 registry 抓一個**不受本 lockfile 約束**
#       的版本來跑。用本地執行檔就沒有這個問題。
#
#    ⚠️ 本檔的 `npx wrangler` 與 `./node_modules/.bin/wrangler` 是**刻意混用**的，
#       判準只有一條：**npx 只在「cwd 底下有裝好的 node_modules」時才安全**
#       （那時它優先用本地那支）。所以
#         - 本節後面的 kv／secret／dev／tail 用 npx —— 它們都在 deploy/watchdog
#           底下、且都在 npm ci 之後，本地一定裝好了
#         - 「已知問題」那節的三種繞法用明確路徑 —— 它們的 cwd 是**沒有
#           node_modules 的乾淨目錄**，npx 在那裡會轉去 registry
#         - 這一步（登入）在 npm ci 之後，**npx 其實也安全**；仍然寫死路徑是因為
#           「登入排在 npm ci 前面」正是前一版的錯。寫死之後這個順序要求就不必靠
#           讀者記得——真的搬到前面去跑，路徑不存在會當場失敗，而不是默默抓一個
#           別的版本裝作沒事
#    ⚠️ 開不了瀏覽器時（SSH、無桌面環境）改用 `login --device`。
#
#    🔒 **憑證儲存位置是個已知的未處理項**（CodeRabbit 於 MR !19 以 CWE-522 提出）：
#       `wrangler login` 完成後，OAuth 憑證預設寫在本機的 wrangler 設定檔裡，
#       同機的其他程序或使用者讀得到，而那組憑證可以部署 Worker。
#       ⚠️ 它建議加 `--use-keyring` 並設 `CLOUDFLARE_AUTH_USE_KEYRING=true` 改存
#       作業系統金鑰圈。**本文件刻意不寫那兩個旗標——因為沒有人在這裡驗證過它們存在。**
#       這一節的其餘內容已經因為「寫了沒跑過的步驟」被連續訂正過三輪，不再多添一筆。
#       要處理的人請先 `wrangler login --help` 確認旗標名稱與該版本的行為，
#       實測通過再寫進來，並把這段警語換掉。
./node_modules/.bin/wrangler login
./node_modules/.bin/wrangler whoami   # 確認登入的是預期的帳號

# 1) wrangler.toml 含真實收件信箱與 KV namespace id，不進版控（R104 最小揭露原則）。
#    從樣板複製一份本機專用的（.gitignore 已排除 wrangler.toml 本體，只追蹤 .example）：
cp wrangler.toml.example wrangler.toml

# 1) 建 KV namespace，把印出來的 id 填進 wrangler.toml 的 <...>
npx wrangler kv namespace create WATCHDOG_KV

# 2) 產一組心跳 token（給備份腳本用），存進 Worker secret
#    同一組值要另外存到 secrets/watchdog_ping_token（見 §4）
npx wrangler secret put PING_TOKEN

# 3) 填 wrangler.toml 剩下的 <佔位符>：ALERT_TO、send_email 的 destination_address
#    ⚠️ 收件地址必須先在 Cloudflare Dashboard → Email Routing → Destination addresses 驗證過，
#       否則 send() 會被拒。

# 4) 部署
#    🔴 **Windows 使用者不要跑下面這行**——已知會卡死（wrangler 4.128.0／4.129.0 實測）。
#       直接跳到下一節「已知問題」，用那裡的 PowerShell 或 Git Bash 繞法。
#       這一行只適用 macOS／Linux。
#    （同樣用本地執行檔而非 npx，理由見第 0 步。）
./node_modules/.bin/wrangler deploy
```

部署後 Worker 會有一個 `https://ipig-watchdog.<你的子網域>.workers.dev` 位址。

### ⚠️ 已知問題：在本目錄直接 `wrangler deploy` 常會卡死

2026-09-09 實測（wrangler 4.128.0／4.129.0，Windows）：在 `deploy/watchdog/`
這個目錄下直接跑 `wrangler deploy`，不管加 `--no-autoconfig`／`--dry-run`／
`--no-bundle`／換版本，都會印完版本橫幅後卡住，行程最終以不正常的方式結束
（曾在錯誤路徑上看到 `Assertion failed: !(handle->flags & UV_HANDLE_CLOSING),
file src\win\async.c`，是 wrangler 在 Windows 上一個底層 libuv 當機的 bug，
不是本專案的設定或程式碼問題）。`Glob(".git/hooks/*")` 在本 repo 也會逾時，
懷疑跟 wrangler 4.129+ 預設開啟的 `autoconfig`（會掃描 git 資訊）卡在這個
repo 的 `.git` 目錄有關，未查到根因（`wrangler deploy --autoconfig`
的說明：`Enables framework detection and automatic configuration when
deploying`，預設 `true`）。

**繞法**（PowerShell 與 Git Bash 兩版實測有效；**WSL 那版未實測**，見該節警語）：
把 `src/worker.js` 與 `wrangler.toml`（含真實值那份）
複製到本 repo 之外的乾淨目錄（維持 `src/worker.js` 的相對路徑結構，因為
`wrangler.toml` 的 `main = "src/worker.js"` 是相對路徑），從那裡跑
`wrangler deploy`。Cloudflare 認的是 `wrangler.toml` 裡的 `name = "ipig-watchdog"`，
不是本機執行目錄，所以這樣部署完全等效：

下面有**三種**寫法，依你用的 shell 挑一種。⚠️ **前兩種與第三種的前提不同，別混用**：

| shell | 工作目錄 | 來源路徑 | 用哪份 `node_modules` |
|---|---|---|---|
| **PowerShell** | `deploy/watchdog/` | 相對 | §3 在 Windows 裝的那份 |
| **Git Bash** | `deploy/watchdog/` | 相對 | 同上（Windows 那份） |
| **WSL** | 任意 | **絕對**（`/mnt/c/…`） | 🔴 **要在 WSL 內自己重裝一份** |

前兩種承接上面 §3 的 `cd deploy/watchdog`，來源路徑一律相對於該目錄。
WSL 不能沿用 Windows 的 `node_modules`（原因見該節），所以它連來源路徑都得寫絕對的。
三者的目的地都只要不在 `ipig_system` repo 底下即可，路徑本身不重要。

🔴 **前兩種一定要用本專案安裝的那支 wrangler，不能在乾淨目錄裡直接打 `npx wrangler`。**
乾淨目錄底下沒有 `node_modules`，`npx` 會轉去外面解析——可能跑到全域快取裡的某個版本、
也可能當場下載最新版，總之**不是 §3 裝的那一支**。所以下面先把本地執行檔的絕對路徑
記起來，再切換目錄。（WSL 那一種不適用：它在目的地自己跑 `npm ci`，`npx` 取到的
就是剛裝好的本地版本。）

**PowerShell**（本問題只發生在 Windows，故列為預設）：

```powershell
$ErrorActionPreference = 'Stop'   # ← 任何一步失敗就停，不要帶著舊檔繼續部署

# 仍在 deploy/watchdog 底下：先把本地 wrangler 的絕對路徑記住
$wrangler = (Resolve-Path '.\node_modules\.bin\wrangler.cmd').Path
$dst = Join-Path $env:TEMP 'watchdog-deploy'

# 🔴 先整個刪掉再建。New-Item -Force 只保證目錄存在，**不會清掉裡面的舊檔**——
#    重用目的地時，上一次部署留下的 worker.js 會原地不動。
#    ⚠️ 只在「目錄不存在」時忽略錯誤。若是權限不足或檔案被佔用而刪不掉，
#    必須停下來——吞掉那個錯就等於帶著一個沒清乾淨的目錄繼續部署。
if (Test-Path $dst) { Remove-Item -Recurse -Force $dst }   # 失敗會因 ErrorActionPreference 中止
New-Item -ItemType Directory -Force -Path (Join-Path $dst 'src') | Out-Null

Copy-Item 'src\worker.js' (Join-Path $dst 'src\worker.js') -Force
Copy-Item 'wrangler.toml' (Join-Path $dst 'wrangler.toml')  -Force
try {
  Set-Location $dst
  & $wrangler deploy
  if ($LASTEXITCODE -ne 0) { throw "wrangler deploy 失敗（exit $LASTEXITCODE）" }
} finally {
  # 🔒 wrangler.toml 含真實收件信箱與 KV namespace id（R104 最小揭露原則），
  #    用完就刪，不留在 TEMP。worker.js 不刪——它本來就在公開 repo 裡。
  #    ⚠️ 這裡的 SilentlyContinue 是對的：部署失敗時可能根本還沒複製過去，
  #    而 finally 不該用「清理失敗」去蓋掉原本那個錯。
  Remove-Item (Join-Path $dst 'wrangler.toml') -Force -ErrorAction SilentlyContinue
}
```

**Git Bash**（沿用 §3 在 Windows 上裝好的那份 `node_modules`）：

```bash
set -euo pipefail   # ← 任何一步失敗就停，不要帶著舊檔繼續部署

# 仍在 deploy/watchdog 底下
wrangler="$PWD/node_modules/.bin/wrangler"
dst=/tmp/watchdog-deploy
rm -rf "$dst"       # 🔴 先刪。mkdir -p 只保證目錄存在，不會清掉上次留下的舊檔
mkdir -p "$dst/src"
# 🔒 wrangler.toml 含真實收件信箱與 KV namespace id（R104 最小揭露原則），
#    不論成功失敗都在離開時刪掉，不留在 /tmp。worker.js 不刪——它本來就公開。
trap 'rm -f "$dst/wrangler.toml"' EXIT
cp src/worker.js "$dst/src/worker.js"
cp wrangler.toml "$dst/wrangler.toml"
cd "$dst"
"$wrangler" deploy
```

**WSL**——🔴 **不能沿用 Windows 裝的 `node_modules`**（2026-09-11 訂正，原文把 WSL 與
Git Bash 並列成同一種做法，那是錯的）：

wrangler 依賴 `workerd`，那是**平台原生的二進位檔**。在 Windows 上裝到的是 Windows 版，
而 WSL 跑的是 Linux Node——直接指向那支執行檔會失敗。WSL 要在自己的環境裡重裝一份：

```bash
set -euo pipefail   # ← 任何一步失敗就停。npm ci 失敗卻照樣 deploy，等於部署上一次的舊檔

# 在 WSL 內，用 Linux 的 Node 裝到 repo 之外的乾淨目錄
dst=~/watchdog-deploy
rm -rf "$dst"       # 🔴 先刪。mkdir -p 不會清掉舊檔，連 node_modules 也會殘留
mkdir -p "$dst/src"
# 🔒 同上：wrangler.toml 含真實值，離開時刪掉。
#    只刪它、不刪整個 $dst——那裡的 node_modules 重裝一次很慢。
trap 'rm -f "$dst/wrangler.toml"' EXIT
src="/mnt/c/<repo 路徑>/deploy/watchdog"   # ← 引號不能省。路徑含空格時 bash 會在
                                          #   賦值處就斷開，後面每個 cp 都拿不到值。
                                          #   （上面的 <repo 路徑> 這個佔位符本身就含
                                          #   空格，照抄不加引號一定斷。）
cp "$src/src/worker.js"      "$dst/src/worker.js"
cp "$src/wrangler.toml"      "$dst/wrangler.toml"
cp "$src/package.json"       "$dst/package.json"
cp "$src/package-lock.json"  "$dst/package-lock.json"
cd "$dst"
npm ci --include=dev   # ← Linux 版的 workerd 在這一步才裝進來。
                       #   --include=dev 同上：wrangler 在 devDependencies

# 🔴 WSL 要自己登入一次。wrangler 的憑證存在 Linux 這側的設定目錄／keyring，
#    **Windows shell 裡的登入狀態不會共用**。跳過這步會在 deploy 時才失敗。
#    開不了瀏覽器（純終端機的 WSL）就用 --device。
./node_modules/.bin/wrangler login          # 或 login --device
./node_modules/.bin/wrangler whoami         # 確認登入的是預期的帳號
./node_modules/.bin/wrangler deploy         # 用本地執行檔，不走 npx
```

🔴 **WSL 這一版從頭到尾未實測**（手上沒有 WSL 環境），請把它當**起點而不是步驟表**。

PowerShell 與 Git Bash 兩版是實際跑過的。WSL 這版的每一步都是推導出來的：
`workerd` 是平台原生二進位 → 要重裝；wrangler 憑證在 Linux 側 → 要重新登入。
推導不等於驗證——2026-09-11 的審查在這一段連續補了「絕對路徑要引號」「`npm ci`
要 `--include=dev`」「WSL 要自己登入」三項，每一項都是原本漏掉的。**很可能還有第四項。**

第一個真的在 WSL 上跑成功的人，請把實際步驟覆蓋掉這一段，並把這個警語刪掉。
在那之前，能用 PowerShell 或 Git Bash 就別走這條。

⚠️ **「用哪一支 wrangler」與「卡死的成因」是兩件事**（本段 2026-09-11 訂正，原文把兩者
寫成同一件，結論因此只對了一半）：

- **在實測過的兩個版本上，換版本沒有用。** 4.128.0 與 4.129.0 都會卡，而換目錄兩者都好。
  所以本節**不釘**特定版本號——在已知的證據下，釘版本解決不了問題，只會把注意力
  導向一個沒被證實的變因。
  ⚠️ **但這不等於「卡死與版本無關」。** 只測了兩個相鄰版本，推不出通則；
  上段也明說根因未查到。若哪天在某個版本上「同一個目錄卻不卡」，那就是反例，
  這條要跟著改。
- **但執行哪一支 wrangler 必須是確定的。** 這跟卡死無關，是可重現性的問題：不確定就等於
  每次部署都在賭一個沒測過的版本。原文只講了前者，於是留下一行會去外面抓版本的指令。

`deploy/watchdog/package.json` 是**獨立的 npm 專案根目錄**——repo 根目錄的 lockfile 鎖不到
它的依賴，所以本目錄有自己的 `package-lock.json`。

§3 用 `npm ci` 而不是 `npm install`，理由是**失敗方式**不同，不是「install 不看 lockfile」
（⚠️ 本段 2026-09-11 訂正——原文是這樣寫的，而那是錯的。CodeRabbit 於 MR !19 指出）：

| | package.json 與 lockfile 相容時 | 不相容時 |
|---|---|---|
| `npm install` | 照 lockfile 裝 | **自行重新解析並改寫 lockfile**，不報錯 |
| `npm ci` | 照 lockfile 裝 | **直接失敗** |

兩者在正常情況下裝出來的東西是一樣的。差別在出問題的時候：`install` 會**靜默地**把
lockfile 改成它自己算出來的版本——於是你以為部署用的是版控裡那一份，實際上不是，
而且工作區還多出一個沒人注意到的未提交變更。`ci` 會當場停下來告訴你兩者對不上。

部署流程要的是後者：**寧可明確失敗，也不要靜默換版本。**

改完 `worker.js` 要部署時，重新做一次上面的複製再部署即可；不用嘗試在
`deploy/watchdog/` 本機目錄修這個問題。

> **刻意用 workers.dev、不綁 `ipigsystem.asia` 的路由。**
> 心跳端點若掛在同一個 zone，DNS 或 tunnel 出問題時備份腳本就 ping 不到，
> 反而製造假告警——看門狗的入口必須跟被監控對象沒有共用的故障點。

## 4. 讓備份腳本送心跳

`db-backup` 服務已經 `env_file: - .env`，所以**不需要改 `docker-compose.yml`**，
在 `.env` 加兩行即可：

```bash
# .env
WATCHDOG_PING_URL=https://ipig-watchdog.<你的子網域>.workers.dev
WATCHDOG_PING_TOKEN=<與 wrangler secret put PING_TOKEN 相同的值>
```

**未設 `WATCHDOG_PING_URL` 就整段跳過**（不報錯）——本機 / staging 不必配。
心跳失敗也**不會**讓備份失敗：監控不該反過來弄壞被監控的東西。

> **為什麼 token 放 `.env` 而不是 `secrets/` 檔案？**
> compose 的 `secrets:` 用 `file:` 宣告，**檔案不存在時整個 `docker compose up` 會失敗**
> （`backup_gpg_pubkey` 已經因此留下一段警語）。在一台同時是 prod 的機器上，
> 為了一個「只能用來偽造備份心跳、無法用來製造告警」的低價值 token
> 去增加 `up` 失敗的機會，划不來。
> 腳本仍支援 `WATCHDOG_PING_TOKEN_FILE`（優先於 `WATCHDOG_PING_TOKEN`），
> 未來要升級成檔案式時，補一個 compose secret 宣告即可，腳本不用動。

改完要讓 `db-backup` 吃到新變數：`docker compose up -d db-backup`
（**`restart` 不會 reload `.env`**）。本次還改了 `Dockerfile.backup`（加 `curl`），
所以第一次要 `docker compose build db-backup` 再 `up -d`。

## 5. 驗收與演練

> ⚠️ 本節與下面的演練區塊是**各自獨立、挑著跑**的指令，不是一串流程——所以
> **刻意沒有** §3 與繞法那幾段的 `set -euo pipefail`。那道守衛的用途是「前一步失敗
> 就別做下一步」，對一份清單加上去，等於把「這些你可以跑」變成「照順序跑完」。

```bash
# 語法檢查
npm run check

# 單元測試（判讀正確性）。不需要 wrangler、不連網、零額外依賴——
# 用 Node 內建的 node:test，並以 test/loader.mjs 把 cloudflare:email 導到 stub。
# ⚠️ 目前**不在 CI 裡跑**（加 workflow job 屬必問），改動 worker.js 後請手動跑一次。
npm test

# 本地跑（cron 不會自動觸發，用 __scheduled 端點手動打；純 `wrangler dev` 不會公開這個端點）
npx wrangler dev --test-scheduled
curl "http://localhost:8787/__scheduled"

# 看即時 log
npx wrangler tail
```

**失敗演練（不要停 prod 容器）**：把 `wrangler.toml` 的 `HEALTH_URL` 暫時改成同網域的
不存在路徑（例 `https://ipigsystem.asia/api/health-does-not-exist`）→ `deploy` →
等 15 分鐘應收到 🔴 告警 → 改回真實路徑 → `deploy` → 下一次 cron 應收到 ✅ 恢復通知。

**心跳演練**：手動用一個過期時間戳覆寫 KV，下一次 cron 應觸發 🟠 逾期告警：

```bash
npx wrangler kv key put --binding WATCHDOG_KV "ping:backup" '{"at":1}' --remote
# 驗完記得送一次真 ping 或等當晚 02:00 備份把它蓋回去
```

**查現況**（token 同 `PING_TOKEN`）：

```bash
curl -H "Authorization: Bearer <PING_TOKEN>" \
     https://ipig-watchdog.<你的子網域>.workers.dev/status
```

## 6. 維護注意

- **免費層 KV 每日 1000 writes**。Worker 已刻意只在狀態變動時回寫（成功時 `lastOkAt`
  最多每小時寫一次）。穩態下每天約 30 次寫入，加 job 時留意別把這個設計改掉。
- **告警寄不出去是最糟的失敗模式**——`sendAlert()` 失敗會 `console.error` 後 rethrow，
  讓該次 cron 在 Cloudflare Dashboard 標記為失敗。定期用 `wrangler tail` 或
  Dashboard → Workers → ipig-watchdog → Logs 確認沒有連續失敗。
- **這支 Worker 本身沒有人監控**（監控者的監控者問題到此為止）。實務上的兜底是
  §5 的季度演練：跟 DR drill 一起做一次假失敗，確認整條通知鏈還活著。
  未做演練的告警系統跟未做還原演練的備份是同一種東西。
