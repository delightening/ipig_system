# AI 資料查詢接口使用指南

## 概述

iPig System 提供 AI 資料查詢接口，允許外部 AI 系統（如 Claude、ChatGPT、自建 Agent）透過 API Key 認證，以唯讀方式查詢系統資料。

---

> ## ⚠️ 兩種 AI 接入的定位區分
>
> iPig 有兩個不同的 AI 接入機制，用途不同，請勿混淆：
>
> | | **AI 資料查詢接口**（本文件） | **MCP Review Server** |
> |--|------------------------------|----------------------|
> | **路徑** | `POST /api/v1/ai/query` | `POST /api/v1/mcp` |
> | **金鑰格式** | `ipig_ai_xxxxxxxx` | `mcp_xxxx_xxxxxxxx` |
> | **金鑰管理** | 系統管理 → AI API Key | 個人設定 → MCP 連線金鑰 |
> | **用途** | 外部 Agent/程式 **唯讀查詢**系統資料（動物、計畫、設施等） | 審查人員（STAFF/CHAIR/VET）透過 claude.ai **操作審查流程** |
> | **寫入能力** | ❌ 無（唯讀） | ✅ 有（建立意見、退回補件） |
> | **認證對象** | 系統級（程式使用） | 個人級（人員使用） |
>
> 👉 若要設定 Claude 審查計畫書的工作流，請參考 [MCP_Review_Server.md](./MCP_Review_Server.md)。

---

## 1. 管理 API Key

### 在系統中建立金鑰

1. 登入 iPig System（需系統管理員權限）
2. 進入 **系統管理 → 系統設定**
3. 捲動至底部的「AI API Key 管理」區塊
4. 點擊「建立金鑰」
5. 填寫：
   - **金鑰名稱**：用途描述（例如「Claude Desktop 查詢用」）
   - **權限範圍**：目前支援「唯讀查詢」
   - **速率限制**：每分鐘允許的請求次數（預設 60）
   - **有效天數**：留空表示永不過期
6. 建立後**立即複製 API Key**（格式：`ipig_ai_xxxxxxxxxxxxxxxx`）
   > ⚠️ 金鑰僅顯示一次，關閉後無法再次查看。若遺失需刪除並重新建立。

### 管理操作

- **啟用/停用**：透過開關切換金鑰狀態，停用後該金鑰立即失效
- **刪除**：永久移除金鑰，所有使用該金鑰的連線將中斷

## 2. API 認證方式

在每個 HTTP 請求中加入自訂 Header：

```
X-AI-API-Key: ipig_ai_xxxxxxxxxxxxxxxx
```

## 3. 可用端點

### 3.1 系統概覽

```
GET /api/ai/overview
```

回傳系統基本統計：動物總數、活躍計畫數、設施數量等。適合讓 AI 了解系統全貌。

### 3.2 資料結構描述

```
GET /api/ai/schema
```

回傳各資料領域的欄位名稱與型別，供 AI 理解可查詢的欄位。

### 3.3 資料查詢

```
POST /api/ai/query
Content-Type: application/json
```

**請求格式：**

```json
{
  "domain": "animals",
  "filters": { "status": "active" },
  "page": 1,
  "per_page": 20,
  "sort_by": "created_at",
  "sort_order": "desc"
}
```

**可查詢領域（domain）：**

| domain | 說明 | 常用篩選欄位 |
|--------|------|-------------|
| `animals` | 動物基本資料 | status, species, gender |
| `observations` | 觀察紀錄 | animal_id, date_range |
| `surgeries` | 手術紀錄 | animal_id, surgery_type |
| `weights` | 體重紀錄 | animal_id |
| `protocols` | 實驗計畫 (AUP) | status, pi_name |
| `facilities` | 設施 / 欄位 | type |
| `stock` | 庫存資料 | product_name |
| `hr_summary` | 人資概況 | - |

**回應格式：**

```json
{
  "domain": "animals",
  "data": [ ... ],
  "total": 42,
  "page": 1,
  "per_page": 20,
  "total_pages": 3,
  "summary": "共 42 隻動物，其中 38 隻為活躍狀態"
}
```

## 4. 使用範例

### cURL

```bash
# 查看系統概覽
curl -H "X-AI-API-Key: ipig_ai_xxxxxxxx" \
  https://your-server/api/ai/overview

# 查詢活躍動物
curl -X POST \
  -H "X-AI-API-Key: ipig_ai_xxxxxxxx" \
  -H "Content-Type: application/json" \
  -d '{"domain":"animals","filters":{"status":"active"}}' \
  https://your-server/api/ai/query
```

### Python

```python
import requests

API_KEY = "ipig_ai_xxxxxxxxxxxxxxxx"
BASE_URL = "https://your-server/api/ai"
HEADERS = {"X-AI-API-Key": API_KEY, "Content-Type": "application/json"}

# 系統概覽
overview = requests.get(f"{BASE_URL}/overview", headers=HEADERS).json()

# 查詢動物
resp = requests.post(f"{BASE_URL}/query", headers=HEADERS, json={
    "domain": "animals",
    "filters": {"status": "active"},
    "per_page": 50
}).json()

print(f"共 {resp['total']} 隻動物")
for animal in resp["data"]:
    print(f"  {animal['ear_tag']} - {animal['name']}")
```

## 5. 安全注意事項

- API Key 以 SHA-256 hash 儲存於資料庫，系統不保留明文
- 所有 AI 查詢皆為**唯讀**，無法修改或刪除任何資料
- 每次查詢自動記錄至 `ai_query_logs`（含端點、來源 IP、處理時間）
- 建議為不同用途建立個別金鑰，以便追蹤與管理
- 定期檢視金鑰的「使用次數」與「最後使用時間」，停用不再需要的金鑰
