# iPig MCP Server — AI 接入審查流程規格

> **版本**：1.0  
> **最後更新**：2026-04-13  
> **對象**：後端開發、系統管理員、IACUC 執行秘書

---

## 一、設計動機

### 問題背景

IACUC 審查流程中，執行秘書（IACUC_STAFF）在 Pre-Review 階段需要：
1. 閱讀計畫書全文
2. 標記問題（needs_attention / concern / suggestion）
3. 退回申請人補件（建立 review_comments + 變更狀態）

目前系統已有「AI 標註面板」（`StaffReviewAssistPanel`），提供 Level 1 規則檢查 + Level 2 Claude API 分析。但這個架構的問題：
- **費用**：後端每次呼叫 Anthropic API 按 token 計費，屬 API 帳單
- **視野有限**：AI 只能看到一次性推送的計畫書內容，無法主動查詢歷史資料

### 解決方案：MCP Server

讓各審查角色在 **claude.ai**（月費制訂閱）中，透過 MCP（Model Context Protocol）直接連接 iPig 系統：

```
各角色（claude.ai 月費帳號）
    ↕ MCP Protocol (JSON-RPC 2.0, HTTP POST)
iPig Backend — POST /api/v1/mcp
    ↕ 內部呼叫
現有 Services / DB（protocols, review_comments, protocol_activities）
```

**核心優勢**：
- **費用走訂閱**：AI 使用費用歸屬使用者的 claude.ai 月費，iPig 不需要自有 Anthropic API Key
- **AI 主動查詢**：Claude 可呼叫多個工具串接資訊（讀計畫書 → 查歷史退件 → 標記 → 退回）
- **對話式操作**：秘書用自然語言描述，Claude 代為執行系統操作

---

## 二、架構概覽

### 通訊協議

MCP 使用 JSON-RPC 2.0，目前實作 **HTTP POST-only**：

```
POST /api/v1/mcp
Authorization: Bearer mcp_xxxx_xxxxxxxxxxxxxxxx
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "read_protocol",
    "arguments": { "protocol_id": "..." }
  }
}
```

> **SSE 推播（GET /mcp）**：暫緩，claude.ai Remote MCP 對 SSE 依賴不強，POST-only 足以支撐核心工作流。待使用者有即時通知需求時再補。

### 認證機制

個人長效 **MCP API Key**，有別於系統級的 `ai_api_keys`：

| | 系統 AI API Key | 個人 MCP Key |
|--|----------------|-------------|
| 用途 | 後端程式讀取系統資料 | 個人 Claude 連線審查系統 |
| 格式 | `ipig_ai_xxxxxxxx` | `mcp_xxxx_xxxxxxxxxxxxxxxx` |
| 權限 | 唯讀查詢 | 依使用者角色動態 |
| 管理入口 | 系統管理 → AI API Key | 個人設定 → MCP 連線金鑰 |
| 儲存方式 | SHA-256 hash | argon2 hash（更安全，長效 key 需要更強保護） |

使用者在 **iPig 個人設定** 產生/撤銷 MCP Key，claude.ai 以 `Authorization: Bearer <key>` 送出每個請求。

---

## 三、角色權限矩陣

| MCP Tool | IACUC_STAFF | IACUC_CHAIR | REVIEWER | VET | SYSTEM_ADMIN |
|----------|:-----------:|:-----------:|:--------:|:---:|:------------:|
| `list_protocols` | ✅ 全部 | ✅ 全部 | ✅ 全部 | ✅ 全部 | ✅ 全部 |
| `read_protocol` | ✅ | ✅ | ✅ + 稽核日誌 | ✅ + 稽核日誌 | ✅ |
| `create_review_flag` | ✅ | ✅ | ❌ 倫理限制 | ❌ | ✅ |
| `batch_return_to_pi` | ✅ | ✅ | ❌ | ❌ | ✅ |
| `add_review_comment` | ✅ | ✅ | ❌ 必須手動 | ❌ | ✅ |
| `get_review_history` | ✅ | ✅ | ❌ | ❌ | ✅ |
| `submit_vet_review` | ❌ | ❌ | ❌ | ✅ | ✅ |

### REVIEWER 倫理限制說明

REVIEWER（審查委員）可使用 MCP **閱讀**任何計畫書，但**禁止所有寫入工具**。

**原因**：IACUC 委員審查是倫理責任，必須由人親自閱讀並形成判斷。若允許 AI 代替委員撰寫審查意見，違反「對動物福祉的倫理道德標準」。MCP 對委員的價值僅在於「方便存取計畫書」。

**稽核機制**：REVIEWER / VET 呼叫 `read_protocol` 時，系統自動記錄至 `protocol_activities`（type = `McpRead`），作為「曾閱覽」的法律佐證。

---

## 四、MCP Tools 規格

### 4.1 `list_protocols`

列出計畫書清單，可依狀態篩選。

**inputSchema**
```json
{
  "type": "object",
  "properties": {
    "status": {
      "type": "string",
      "description": "計畫書狀態，留空=全部。可用值：DRAFT, SUBMITTED, PRE_REVIEW, PRE_REVIEW_REVISION_REQUIRED, VET_REVIEW, UNDER_REVIEW, APPROVED, REJECTED 等"
    },
    "limit": { "type": "integer", "default": 20 }
  }
}
```

**回傳**
```json
[
  {
    "id": "uuid",
    "protocol_no": "AUP-2026-042",
    "title": "豬隻心臟支架植入模型建立",
    "pi_name": "王大明",
    "status": "PRE_REVIEW",
    "submitted_at": "2026-04-10T08:30:00Z"
  }
]
```

---

### 4.2 `read_protocol`

讀取計畫書完整內容。REVIEWER / VET 呼叫時自動記錄稽核日誌。

**inputSchema**
```json
{
  "type": "object",
  "properties": {
    "protocol_id": {
      "type": "string",
      "description": "計畫書 UUID 或編號（AUP-YYYY-NNN）"
    }
  },
  "required": ["protocol_id"]
}
```

**回傳（含脈絡資訊）**
```json
{
  "id": "uuid",
  "protocol_no": "AUP-2026-042",
  "title": "...",
  "status": "PRE_REVIEW",
  "version": 2,
  "pi": { "name": "王大明", "organization": "台大醫院", "email": "..." },
  "pi_history": {
    "total_protocols": 5,
    "past_return_count": 3,
    "common_return_reasons": ["人道終點未量化", "交叉引用失效"]
  },
  "content": {
    "basic": { "study_title": "...", "start_date": "...", "end_date": "..." },
    "purpose": { "significance": "...", "replacement": "...", "reduction": "...", "refinement": "..." },
    "design": { ... },
    "animals": { ... },
    "personnel": { ... },
    "surgery": { ... },
    "anesthesia": { ... },
    "postop": { ... },
    "euthanasia": { ... },
    "attachments": [...]
  },
  "previous_return_comments": [
    { "section": "3.2", "message": "替代方案說明不足" }
  ]
}
```

---

### 4.3 `create_review_flag`

建立 Pre-Review 審查標註，寫入 `protocol_ai_reviews` 記錄。

**權限**：IACUC_STAFF / IACUC_CHAIR / SYSTEM_ADMIN

**inputSchema**
```json
{
  "type": "object",
  "properties": {
    "protocol_id": { "type": "string" },
    "flag_type": {
      "type": "string",
      "enum": ["needs_attention", "concern", "suggestion"],
      "description": "needs_attention=格式/規範缺漏，concern=實質性疑慮，suggestion=建議改善"
    },
    "section": { "type": "string", "description": "問題所在章節，e.g. '3.2 替代方案'" },
    "message": { "type": "string", "description": "問題描述" },
    "suggestion": { "type": "string", "description": "建議修正方式" }
  },
  "required": ["protocol_id", "flag_type", "section", "message", "suggestion"]
}
```

---

### 4.4 `batch_return_to_pi`

**原子操作**：建立 `review_comments`（每個 flag 一筆）+ 變更 protocol 狀態為 `PRE_REVIEW_REVISION_REQUIRED`。

**權限**：IACUC_STAFF / IACUC_CHAIR / SYSTEM_ADMIN

**前提條件**：Protocol 當前狀態必須為 `PRE_REVIEW`。

**inputSchema**
```json
{
  "type": "object",
  "properties": {
    "protocol_id": { "type": "string" },
    "flags": {
      "type": "array",
      "description": "要退回的問題清單（可來自 AI 分析或人工判斷）",
      "items": {
        "type": "object",
        "properties": {
          "section": { "type": "string" },
          "message": { "type": "string" },
          "suggestion": { "type": "string" }
        },
        "required": ["section", "message", "suggestion"]
      }
    },
    "additional_note": {
      "type": "string",
      "description": "秘書補充說明（可選），會另建立一筆 comment"
    }
  },
  "required": ["protocol_id", "flags"]
}
```

**回傳**
```json
{ "created_comments": 3, "status": "PRE_REVIEW_REVISION_REQUIRED" }
```

---

### 4.5 `get_review_history`

查詢 PI 的歷史退件紀錄，讓 Claude 建立先驗印象。

**權限**：IACUC_STAFF / IACUC_CHAIR / SYSTEM_ADMIN

**inputSchema**
```json
{
  "type": "object",
  "properties": {
    "pi_user_id": { "type": "string" },
    "limit": { "type": "integer", "default": 5 }
  },
  "required": ["pi_user_id"]
}
```

---

### 4.6 `submit_vet_review`

對應 `POST /api/v1/reviews/vet-form`，協助 VET 填寫獸醫查檢表。

**權限**：VET / SYSTEM_ADMIN

**欄位結構**（對應現有 `VetReviewForm`）：
```rust
VetReviewItem {
    item_name: String,
    compliance: "V" | "X" | "-",  // 符合 / 不符合 / 不適用
    comment: Option<String>,
}
```

**inputSchema**
```json
{
  "type": "object",
  "properties": {
    "protocol_id": { "type": "string" },
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "item_name": { "type": "string" },
          "compliance": { "type": "string", "enum": ["V", "X", "-"] },
          "comment": { "type": "string" }
        },
        "required": ["item_name", "compliance"]
      }
    },
    "vet_signature": { "type": "string", "description": "獸醫師姓名（人工確認後填入）" }
  },
  "required": ["protocol_id", "items"]
}
```

> ⚠️ **VET 使用注意**：Claude 可協助預填查檢表，但 `vet_signature` 必須由 VET 本人確認後才提交。建議工作流：Claude 預填 → VET 在 iPig 介面審閱修改 → 正式簽章送出。

---

## 五、資料庫變更

### 新增表：`user_mcp_keys`

```sql
CREATE TABLE user_mcp_keys (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash     TEXT        NOT NULL UNIQUE,  -- argon2 hash，明文只顯示一次
    key_prefix   VARCHAR(16) NOT NULL,          -- 顯示用，e.g. "mcp_a1b2c3d4"
    name         TEXT        NOT NULL,          -- 使用者自訂名稱
    last_used_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at   TIMESTAMPTZ                    -- NULL = 有效
);

CREATE INDEX idx_user_mcp_keys_user_id ON user_mcp_keys (user_id) WHERE revoked_at IS NULL;
```

### 新增 `ProtocolActivityType::McpRead`

在現有 enum 加入 `McpRead`，記錄 REVIEWER / VET 透過 MCP 閱讀計畫書的稽核軌跡。

```sql
-- 若 protocol_activity_type 是 PostgreSQL ENUM
ALTER TYPE protocol_activity_type ADD VALUE 'MCP_READ';
```

---

## 六、後端實作結構

### 新增檔案

| 檔案 | 職責 |
|------|------|
| `backend/migrations/0NN_user_mcp_keys.sql` | 建立 `user_mcp_keys` 表 |
| `backend/src/handlers/mcp.rs` | MCP JSON-RPC dispatch，認證解析 |
| `backend/src/services/mcp/mod.rs` | Tool 執行入口，權限二次檢查 |
| `backend/src/services/mcp/tools.rs` | 各 tool 實作（呼叫現有 services） |
| `backend/src/routes/mcp.rs` | 掛載 `POST /api/v1/mcp` |

### 修改檔案

| 檔案 | 變更 |
|------|------|
| `backend/src/routes/mod.rs` | 掛載 mcp routes |
| `backend/src/models/protocol.rs` | `ProtocolActivityType` 加入 `McpRead` |

### 核心 Handler 邏輯

```rust
pub async fn mcp_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>> {
    // 1. Bearer token → 查 user_mcp_keys → 取得 CurrentUser
    let user = authenticate_mcp_key(&state.db, &headers).await?;

    // 2. 分派
    match req.method.as_str() {
        "initialize"  => Ok(handle_initialize()),
        "tools/list"  => Ok(handle_tools_list(&user)),   // 依角色過濾工具清單
        "tools/call"  => handle_tool_call(&state, &user, &req).await,
        _             => Err(AppError::BadRequest("Unknown MCP method".into())),
    }
}
```

---

## 七、前端：MCP Key 管理

### 路徑
`frontend/src/pages/settings/` — 嵌入個人設定頁

### 功能
1. **產生金鑰**：輸入名稱 → 顯示完整 key（一次性）→ 複製按鈕
2. **金鑰列表**：顯示前綴 + 名稱 + 最後使用時間
3. **撤銷金鑰**：點撤銷 → 確認 → 立即失效

### API
```typescript
// frontend/src/lib/api/mcpKeys.ts
export const mcpKeysApi = {
    list: (): Promise<McpKey[]>
    create: (name: string): Promise<{ key: string; meta: McpKey }> // key 只回傳一次
    revoke: (id: string): Promise<void>
}
```

---

## 八、使用者設定指南

### claude.ai Remote MCP 設定

1. 登入 iPig → 個人設定 → **MCP 連線金鑰** → **產生新金鑰**
2. 輸入名稱（e.g. 「我的 claude.ai」）→ 複製顯示的完整 key
3. 前往 [claude.ai](https://claude.ai) → Settings → **Integrations** → Add MCP Server：
   - **Name**：iPig 審查系統
   - **URL**：`https://ipig.example.com/api/v1/mcp`
   - **Authorization**：`Bearer mcp_xxxx_xxxxxxxxxxxxxxxx`
4. 儲存後，在任何 Claude 對話中即可呼叫 iPig 工具

### Claude Code 設定（本地開發 / 測試）

在 `~/.claude/settings.json` 中加入：

```json
{
  "mcpServers": {
    "ipig": {
      "type": "http",
      "url": "http://localhost:3000/api/v1/mcp",
      "headers": {
        "Authorization": "Bearer mcp_test_xxxx"
      }
    }
  }
}
```

---

## 九、典型工作流範例

### 執行秘書 Pre-Review

```
秘書：「幫我審查 AUP-2026-042」

Claude 呼叫 read_protocol("AUP-2026-042")
→ 閱讀計畫書全文 + PI 歷史退件記錄

Claude：「這份計畫書有以下問題：
  1. [3.2 替代方案] 未說明資料庫搜尋結果...
  2. [5.1 人道終點] 量化門檻缺失...
  3. [附件] 動物訓練證明未上傳...
  要直接退回給申請人嗎？」

秘書：「把前兩項退回，第三項我先確認」

Claude 呼叫 batch_return_to_pi({
  protocol_id: "...",
  flags: [問題1, 問題2]
})

Claude：「已退回，建立 2 筆審查意見，
  計畫書狀態已更新為『需補件』」
```

### REVIEWER 閱讀計畫書

```
委員：「列出指派給我的計畫書」

Claude 呼叫 list_protocols({ status: "UNDER_REVIEW" })
→ 顯示清單

委員：「幫我讀 AUP-2026-038」

Claude 呼叫 read_protocol("AUP-2026-038")
→ [系統記錄 McpRead 稽核日誌]
→ Claude 呈現計畫書內容摘要

委員：「好，我去 iPig 填寫審查意見」
（後續寫入操作在 iPig 介面完成，不透過 MCP）
```

---

## 十、三項暫緩功能分析

### 10.1 SSE 推播（GET /mcp）

**說明**：MCP 支援 server → client 主動推訊息（如「新計畫書送審」通知）。

| 面向 | 分析 |
|------|------|
| **需求強度** | 低：claude.ai 大多數工作流走 POST 即可 |
| **開發成本** | 1-2 天（方案 A，Axum 原生 SSE）；3-5 天（方案 B，Redis Pub/Sub） |
| **基礎設施成本** | 方案 A：$0；方案 B：~$10-20/月 |
| **實作時機** | 使用者明確反映需要即時通知時再補 |

### 10.2 StaffReviewAssistPanel Checkbox UI

**說明**：在 iPig Web 介面的秘書 AI 標註面板加 checkbox，讓不使用 claude.ai 的秘書也能一鍵退回。

| 面向 | 分析 |
|------|------|
| **適用對象** | 沒有 claude.ai 訂閱的秘書 |
| **開發成本** | 2-3 天（純前端，後端 batch_return endpoint 兩者共用） |
| **Pros** | 降級方案，不依賴 Claude；操作明確；稽核記錄清晰 |
| **Cons** | 與 MCP 工作流重疊；若秘書全數轉用 Claude 則閒置 |
| **實作時機** | 確認秘書 claude.ai 訂閱率後決定；建議先建共用後端 endpoint |

### 10.3 submit_vet_review 欄位定義

**說明**：VET 透過 Claude 填寫獸醫查檢表。

| 面向 | 分析 |
|------|------|
| **現有結構** | `review_form` 為 JSONB，設計彈性 |
| **開發成本** | 0.5 天（欄位已確認，只需設計 inputSchema） |
| **Pros** | Claude 可預填查檢表，VET 確認後簽章；重複性高的表格最適合 AI 協助 |
| **Cons** | 需先確認標準查檢項目清單；VET 工作流與 STAFF 不同，需要獨立設計 |
| **實作時機** | VET 審查流程確認後，約半天可完成 |

---

## 十一、與現有 AI 系統的定位區別

| | 現有 Level 1/2 預審 | 新 MCP Server |
|--|--------------------|--------------------|
| **AI 費用** | iPig 帳單（API key 計費） | 使用者 claude.ai 月費 |
| **觸發方式** | 系統按鈕觸發 | Claude 對話主動呼叫 |
| **使用角色** | PI（提交前自查）、STAFF（Pre-Review）| STAFF、CHAIR、REVIEWER（閱讀）、VET |
| **AI 視野** | 一次性推送計畫書 | 可多輪查詢（計畫書 + 歷史 + 評論） |
| **寫入能力** | 無（只分析） | 有（建立 flags、退回補件、填 VET 表） |
| **適用場景** | 快速自動化分析 | 複雜多步驟審查對話 |

兩者並存互補，不互相取代。
