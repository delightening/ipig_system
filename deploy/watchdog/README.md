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

需要 Cloudflare 帳號登入（`npx wrangler login`），以及 `ipigsystem.asia` zone 的 Email Routing 已啟用。

```bash
cd deploy/watchdog
npm install

# 1) 建 KV namespace，把印出來的 id 填進 wrangler.toml 的 <...>
npx wrangler kv namespace create WATCHDOG_KV

# 2) 產一組心跳 token（給備份腳本用），存進 Worker secret
#    同一組值要另外存到 secrets/watchdog_ping_token（見 §4）
npx wrangler secret put PING_TOKEN

# 3) 填 wrangler.toml 剩下的 <佔位符>：ALERT_TO、send_email 的 destination_address
#    ⚠️ 收件地址必須先在 Cloudflare Dashboard → Email Routing → Destination addresses 驗證過，
#       否則 send() 會被拒。

# 4) 部署
npx wrangler deploy
```

部署後 Worker 會有一個 `https://ipig-watchdog.<你的子網域>.workers.dev` 位址。

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
