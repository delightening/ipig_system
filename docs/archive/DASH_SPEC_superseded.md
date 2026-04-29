# DASH_SPEC v1.0 — ipig-dashboard 骨架（⛔ SUPERSEDED / 作廢）

> **⛔ 此方案於 2026-04-19 作廢，不動工**
> **替代方案**：`C:\System Coding\ipig_system\docs\OBSERVABILITY_PLAN.md`（R24）
> **作廢原因**：
> 1. 盤點後確認 ipig_system 已有 80% observability/audit 基礎（R22 完整攻擊偵測 + 4 種通知管道）
> 2. 「異常即封鎖」需 inline middleware 處理（延遲 < 1ms），獨立服務做不到
> 3. Grafana 已是業界標準 dashboard，自建無意義
> 4. 單人用場景不需要多服務架構
>
> 本檔保留作為決策紀錄，勿刪除。
>
> ---
>
> **原始狀態**：v1.0 骨架草案（2026-04-18 起草）
> **原設計**：獨立 Node+Hono+SQLite dashboard，監控 Claude Code agent + Safety Gate

---

## 1. 目標與範圍

### 1.1 目標
- Claude Code agent（針對 ipig_system 開發）的**視覺監控中心**
- **主動攔截**危險操作（Safety Gate via PreToolUse hook）
- 歷史 session log 查詢與重播

### 1.2 非目標（v1.0 不做）
- 多使用者 / 登入系統 → 不做（單機單人）
- AgentShield 完整功能 → v1.1（CLAUDE.md secret scan、MCP config audit、hook 權限稽核）
- 遠端 agent 監控 → v1.2+（目前只監控本機 Claude Code）
- 統計圖表、匯出報告 → v1.1+

### 1.3 技術棧（沿用任務排程系統）
| 層 | 技術 |
|----|------|
| Backend | Node.js 24 + TypeScript + Hono |
| DB | SQLite（better-sqlite3） |
| Frontend | React 18 + Vite + TanStack Query |
| Streaming | Server-Sent Events (SSE) |
| Dev | backend `:3000` / Vite `:5173` |
| Prod | VPS，子網域 `dash.ipigsystem.asia`，localhost-only 監聽 + 反向代理 |

---

## 2. 資料模型（SQLite）

### 2.1 `sessions`
| 欄位 | 型別 | 說明 |
|------|------|------|
| id | TEXT PK | uuid |
| started_at | TEXT | ISO8601 |
| ended_at | TEXT NULL | |
| cwd | TEXT | Claude Code 啟動目錄（應為 `C:/System Coding/ipig_system` 或子目錄） |
| status | TEXT | `running` / `completed` / `aborted` |

### 2.2 `tool_calls`
| 欄位 | 型別 | 說明 |
|------|------|------|
| id | INTEGER PK AUTOINCREMENT | |
| session_id | TEXT FK → sessions.id | |
| timestamp | TEXT | |
| tool_name | TEXT | e.g. `Bash` / `Write` / `Edit` |
| input_json | TEXT | tool input payload |
| output_snippet | TEXT NULL | 前 2KB，避免肥大 |
| duration_ms | INTEGER NULL | |
| status | TEXT | `allowed` / `blocked` / `completed` / `error` |

### 2.3 `safety_events`
| 欄位 | 型別 | 說明 |
|------|------|------|
| id | INTEGER PK | |
| session_id | TEXT FK | |
| timestamp | TEXT | |
| rule_id | TEXT | `R1_rm` / `R2_sysdir` / `R3_secrets` |
| tool_name | TEXT | |
| input_snippet | TEXT | 被攔截的指令或路徑 |
| action | TEXT | `blocked` / `warned` |
| override | INTEGER | 0/1，是否被使用者覆寫 |

### 2.4 `hooks_log`（debug / 稽核用）
| 欄位 | 型別 | 說明 |
|------|------|------|
| id | INTEGER PK | |
| session_id | TEXT FK NULL | session-start 之前可能為 null |
| timestamp | TEXT | |
| hook_event | TEXT | `PreToolUse` / `PostToolUse` / `SessionStart` / `Stop` / `UserPromptSubmit` |
| tool_name | TEXT NULL | |
| payload_json | TEXT | 原始 hook input |

---

## 3. 路由

### 3.1 Backend API（Hono）

#### Ingestion（Claude Code hooks 呼叫）
| Method | Path | 用途 | 回應 |
|--------|------|------|------|
| POST | `/api/hook/pre-tool-use` | 接收 PreToolUse event，判斷 allow/deny | `{ decision: "allow" }` 或 `{ decision: "deny", reason }` |
| POST | `/api/hook/post-tool-use` | 記錄 tool 執行結果 | 200 OK |
| POST | `/api/hook/session-start` | 開啟新 session（回傳 session_id） | `{ session_id }` |
| POST | `/api/hook/stop` | 關閉 session | 200 OK |

#### Query
| Method | Path | 用途 |
|--------|------|------|
| GET | `/api/sessions?limit=&offset=&status=` | session 列表 |
| GET | `/api/sessions/:id` | session 詳情 |
| GET | `/api/sessions/:id/tool-calls` | 該 session 的 tool call 列表 |
| GET | `/api/safety-events?from=&to=` | 攔截事件列表 |
| GET | `/api/stream/:session_id` | SSE 即時 log |

#### Config
| Method | Path | 用途 |
|--------|------|------|
| GET | `/api/safety/rules` | 列出 Safety Gate 規則狀態 |
| PATCH | `/api/safety/rules/:id` | 啟用/停用規則 |

### 3.2 Frontend（React Router）
| Path | 頁面 |
|------|------|
| `/` | Dashboard 總覽 |
| `/sessions` | Session 歷史列表 |
| `/sessions/:id` | Session 詳情 + 即時 log / 歷史重播 |
| `/safety` | Safety Gate 攔截事件時間線 |
| `/settings` | 規則開關 + Hooks 設定複製 |

---

## 4. UI 頁面

### 4.1 `/` Dashboard
- 當前 running session 卡片（點擊跳 session 詳情）
- 今日統計：tool 呼叫數、攔截次數、平均 session 時長
- 最近 5 筆攔截事件（摘要）

### 4.2 `/sessions`
- DataTable 欄位：時間、cwd、狀態、tool 數、持續時間、攔截數
- 篩選：日期範圍、狀態

### 4.3 `/sessions/:id`
- 上區：session metadata（cwd、起訖時間、總 tool 數）
- 中區：SSE 即時 log（running 狀態）或歷史 log（completed）
- 下區：tool call 表格，可展開 input/output JSON

### 4.4 `/safety`
- 時間線顯示所有攔截事件，依 rule_id 配色
- 每筆可展開看被攔截的完整指令、所屬 session

### 4.5 `/settings`
- 規則開關：R1 / R2 / R3（各附說明）
- Hooks endpoint 資訊（含「複製到 settings.json」按鈕，一鍵複製可貼入 Claude Code 設定）

---

## 5. Safety Gate 三條規則（v1.0）

### R1：rm / rmdir 攔截
- **觸發**：Bash 工具的 command 匹配 `\brm\s+(-[rRfi]+\s+)?` 或 `\brmdir\s+`
- **例外（白名單）**：目標路徑僅落在以下子目錄時放行
  - `node_modules/`
  - `dist/` / `build/`
  - `target/`（Rust）
  - `.next/` / `.vite/`
  - `/tmp/` 下
- **動作**：`blocked`，回覆 agent 說明原因
- **覆寫**：使用者可在 dashboard 手動標記為已核准（但 v1.0 不提供即時覆寫流程，需先停止 agent 再重跑）

### R2：系統目錄寫入攔截
- **觸發**：Write / Edit 工具的 file_path 落在下列任一前綴
  - Windows：`C:\Windows\`、`C:\Program Files\`、`C:\Program Files (x86)\`
  - Unix：`/etc/`、`/usr/`、`/bin/`、`/System/`、`/Library/`
- **動作**：`blocked`（**無覆寫**）

### R3：敏感檔案讀寫攔截
- **觸發**：file_path 或 command 包含以下 pattern
  - `.env`（不含 `.env.example` / `.env.sample` / `.env.template`）
  - `id_rsa` / `id_ed25519`
  - `credentials.json` / `credentials.yaml`
  - `*.pem` / `*.key`
- **動作**：`blocked`，明確告知 agent
- **適用工具**：Read / Write / Edit / Bash（cat/type 等）

### 規則引擎要求
- 規則定義集中於 `backend/src/services/safety-gate.ts`
- 每條規則一個 function：`(input: PreToolUseEvent) => RuleResult`
- 新增規則不改核心，只加 function 並註冊

---

## 6. Hooks 介面

### 6.1 Claude Code `settings.json` 片段

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          { "type": "command", "command": "curl -s -X POST http://localhost:3000/api/hook/session-start -H 'Content-Type: application/json' -d @-" }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash|Write|Edit|Read",
        "hooks": [
          { "type": "command", "command": "curl -s -X POST http://localhost:3000/api/hook/pre-tool-use -H 'Content-Type: application/json' -d @-" }
        ]
      }
    ],
    "PostToolUse": [
      {
        "hooks": [
          { "type": "command", "command": "curl -s -X POST http://localhost:3000/api/hook/post-tool-use -H 'Content-Type: application/json' -d @-" }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          { "type": "command", "command": "curl -s -X POST http://localhost:3000/api/hook/stop -H 'Content-Type: application/json' -d @-" }
        ]
      }
    ]
  }
}
```

### 6.2 PreToolUse 回應協定
- **允許**：HTTP 200，body `{"decision":"allow"}` → Claude Code 照常執行
- **阻擋**：HTTP 200，body `{"decision":"deny","reason":"..."}` → Claude Code 終止該 tool call，把 reason 回饋給 agent
- **超時**：dash 回應 >2s 視為允許（fail-open，避免擋到正常開發）
  - ⚠️ 待確認：fail-open 還是 fail-closed？v1.0 建議 fail-open（不阻塞開發），v1.1 視情況改

### 6.3 Session 識別
- SessionStart hook 回傳 `{ session_id }`
- 後續 hook payload 需帶 `session_id`
- ⚠️ **待補**：Claude Code 原生 hook payload 是否有內建 session 識別？若無，需靠啟動時注入環境變數 `DASH_SESSION_ID`，或在 PreToolUse hook 內自動建立 session

---

## 7. MVP 範圍界線

### 必做（v1.0）
- [ ] SQLite schema + migration（4 張表）
- [ ] Hono backend：4 個 hook endpoint + 5 個 query endpoint + 2 個 config endpoint
- [ ] Safety Gate 規則引擎（R1 / R2 / R3）
- [ ] SSE 即時 log stream
- [ ] React 5 個頁面（Dashboard / Sessions / Session Detail / Safety / Settings）
- [ ] Hooks 設定「一鍵複製到 settings.json」按鈕
- [ ] 基本錯誤處理 + console log

### 延後（v1.1）
- AgentShield：CLAUDE.md secret scan（定期掃描已知 pattern）
- AgentShield：MCP server 配置稽核
- AgentShield：Hook 腳本權限檢查
- 統計圖表（日/週 tool 使用趨勢）
- 攔截事件即時覆寫流程

### 延後（v1.2+）
- 多使用者 / 登入
- 遠端 agent 監控（跨機器）
- 匯出報告 / 稽核 log

### 明確不做
- 生產環境 multi-tenant
- 分散式收集

---

## 8. 專案結構（參考任務排程系統）

```
ipig-dashboard/
├── backend/
│   ├── src/
│   │   ├── index.ts              # Hono app entry
│   │   ├── routes/
│   │   │   ├── hooks.ts          # ingestion endpoints
│   │   │   ├── sessions.ts
│   │   │   └── safety.ts
│   │   ├── services/
│   │   │   ├── safety-gate.ts    # R1/R2/R3 規則引擎
│   │   │   └── sse.ts            # SSE broadcast
│   │   ├── db/
│   │   │   ├── schema.sql
│   │   │   └── client.ts
│   │   └── types/
│   ├── data/
│   │   └── dash.sqlite           # gitignored
│   ├── package.json
│   └── tsconfig.json
├── frontend/
│   ├── src/
│   │   ├── App.tsx
│   │   ├── pages/                # 5 頁
│   │   ├── components/
│   │   └── lib/api/
│   ├── package.json
│   └── vite.config.ts
├── DASH_SPEC.md                  # 本檔
├── README.md
└── .gitignore
```

---

## 9. 待補章節（骨架未涵蓋，審閱時請標註優先級）

| # | 項目 | 建議處理時機 |
|---|------|-------------|
| 1 | 部署：VPS 反向代理（Caddy / nginx）、SSL、process manager（pm2 / systemd） | 進入部署前補 |
| 2 | SQLite 備份策略（定期 copy、vacuum） | v1.0 可簡化為手動 |
| 3 | 結構化 logging（pino?）與 log rotation | 進建置時決定 |
| 4 | dash 自身安全：localhost-only 綁定 vs token 保護 | v1.0 決策：僅綁 `127.0.0.1`，無 token |
| 5 | SessionStart hook 的 session_id 注入機制（詳見 §6.3 待確認） | 建置前必須確認 |
| 6 | 錯誤處理策略（hook 回應失敗、DB lock、SSE 斷線） | 進建置時決定 |
| 7 | 前端設計 token / UI 套件選擇（沿用 ipig_system 的 shadcn？或更輕量） | UI 階段決定 |

---

## 10. 決策紀錄（Decisions Log）

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-04-18 | v1.0 僅監控 ipig_system 本機 | 範圍收斂，先做能跑的 MVP |
| 2026-04-18 | Safety Gate 採 PreToolUse 主動攔截 | 事後記錄擋不住破壞，須在 tool 執行前介入 |
| 2026-04-18 | 單機單人，無登入 | 使用者明確指定，移除 auth 複雜度 |
| 2026-04-18 | 獨立 repo（不放 ipig_system 內） | 技術棧不同（Node vs Rust）、部署目標不同 |
| 2026-04-18 | dash 自身僅綁 `127.0.0.1` | 單機單人前提下最簡安全模型 |
