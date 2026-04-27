# 客戶入口計畫 — 邀請制存取

> 目標：讓外部客戶（PI / 計畫主持人）透過邀請制進入系統，自助提交 IACUC 動物實驗計劃書並追蹤審查進度。

---

## 一、現況分析

### 已有的基礎
- ✅ 完整的 IACUC 審查流程（16 種狀態、3 階段審查）
- ✅ 角色權限系統（RBAC，已有 PI、CLIENT 角色）
- ✅ Email 發送功能（SMTP + Welcome Email + Password Reset）
- ✅ JWT + 2FA 認證系統
- ✅ 帳號到期日機制（`expires_at` 欄位已存在）
- ✅ 計劃書 10 段表單 + 完整驗證規則
- ✅ PDF 匯出（計劃書、審查結果、審查意見）

### 已有但需確認的
- ✅ `animal.animal.view_project` 權限 — PI/CLIENT 可看到自己計劃書下的動物
- ✅ `MyProjectDetailPage` 已透過 `iacuc_no` 串聯顯示動物列表
- ✅ 動物紀錄唯讀（觀察、手術、血檢）+ 匯出功能
- ✅ 動物存取過濾：`view_project` 用戶只能看到有 `iacuc_no` 的動物

### 目前缺少的
- ❌ 邀請碼 / 邀請連結機制
- ❌ 客戶自助設定密碼流程（目前是 Admin 建帳 → Welcome Email）
- ❌ 客戶專屬入口頁面（簡化介面）
- ❌ 客戶可見範圍限制（只看自己的計劃書）
- ❌ 執行秘書 Pre-Review AI 標註輔助

---

## 二、邀請制流程設計

### 核心原則

- **一個動作**：您只需輸入客戶 Email → 送出，其他都是客戶自己完成
- **用過即丟**：邀請連結一次性使用，7 天過期
- **雙通道送達**：系統自動寄 Email + 畫面顯示連結（方便您複製貼到 LINE）
- **重複邀請保護**：同一 Email 只能邀請一次，已有帳號則導向重設密碼
- **預設角色 PI**：不需選擇角色，所有客戶都是 PI

### 流程圖

```
管理員（IACUC_STAFF）
    │
    ▼
[建立邀請] ─── 只需輸入：客戶 Email（組織名稱可選）
    │
    ▼
[系統檢查 Email]
    │
    ├─ Email 已有帳號 → 提示「此 Email 已註冊」→ [跳轉重設密碼頁面]
    │
    ├─ Email 已被邀請（pending）→ 提示「已邀請過」→ 可選 [重新發送]
    │
    └─ Email 全新 → 產生一次性邀請連結
         │
         ├─→ ① 自動發送邀請 Email 給客戶
         │
         └─→ ② 畫面顯示連結（可複製，貼到 LINE/其他管道）
              │
              ▼
        客戶點擊連結
              │
              ▼
        [自助註冊頁面]（不需您介入）
          - 姓名（必填）
          - 電話（必填）
          - 組織（必填）
          - 職稱
          - 設定密碼
          - 同意服務條款
              │
              ▼
        [帳號自動建立]
          - 角色 = PI（自動分配）
          - Email = 邀請時指定的（不可更改，即為驗證）
          - 邀請連結標記為 accepted（不可再用）
              │
              ▼
        [自動登入] → 進入「我的計劃書」頁面
```

### 管理員操作體驗

```
┌──────────────────────────────────────────────────────┐
│  邀請客戶                                             │
├──────────────────────────────────────────────────────┤
│                                                      │
│  客戶 Email *                                        │
│  ┌────────────────────────────────────┐              │
│  │ wang.daming@hospital.org           │              │
│  └────────────────────────────────────┘              │
│                                                      │
│  組織名稱（選填）                                     │
│  ┌────────────────────────────────────┐              │
│  │ 台大醫院                           │              │
│  └────────────────────────────────────┘              │
│                                                      │
│                           [取消]  [送出邀請]          │
└──────────────────────────────────────────────────────┘

    ↓ 送出後 ↓

┌──────────────────────────────────────────────────────┐
│  ✅ 邀請已送出                                       │
├──────────────────────────────────────────────────────┤
│                                                      │
│  已發送邀請 Email 至 wang.daming@hospital.org         │
│                                                      │
│  或複製以下連結，透過其他管道傳給客戶：               │
│  ┌────────────────────────────────────────────┐      │
│  │ https://ipigsystem.asia/invite/a8f3...x9z  │ 📋  │
│  └────────────────────────────────────────────┘      │
│  ⏰ 此連結將於 2026-04-05 過期                        │
│                                                      │
│                                        [完成]        │
└──────────────────────────────────────────────────────┘
```

---

## 三、功能規格

### 3.1 邀請管理（Admin 端）

#### 資料庫：`invitations` 表

```sql
CREATE TABLE invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,   -- 同一 Email 只能有一筆有效邀請
    organization VARCHAR(255),
    invitation_token VARCHAR(255) UNIQUE NOT NULL,
    invited_by UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
        -- pending | accepted | expired | revoked
    expires_at TIMESTAMPTZ NOT NULL,     -- 邀請有效期（預設 7 天）
    accepted_at TIMESTAMPTZ,
    created_user_id UUID REFERENCES users(id),  -- 接受後建立的 user
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 確保同一 Email 只有一筆 pending 邀請
CREATE UNIQUE INDEX idx_invitations_email_pending
    ON invitations (email) WHERE status = 'pending';
```

> **注意**：不再需要 `role_ids` 欄位，角色固定為 PI，在 accept 時自動分配。

#### API Endpoints

```
POST   /api/invitations              # 建立邀請（只需 Email + 可選組織名稱）
GET    /api/invitations              # 列出所有邀請
DELETE /api/invitations/{id}         # 撤銷邀請
POST   /api/invitations/{id}/resend  # 重新發送邀請 Email + 回傳連結
```

#### 建立邀請的後端邏輯

```rust
// POST /api/invitations
fn create_invitation(email, organization) {
    // 1. 檢查 Email 是否已有帳號
    if user_exists(email) {
        return Error::EmailAlreadyRegistered {
            // 前端收到此錯誤 → 顯示「已有帳號，是否重設密碼？」
            redirect_to: "/forgot-password"
        }
    }

    // 2. 檢查是否已有 pending 邀請
    if pending_invitation_exists(email) {
        return Error::AlreadyInvited {
            // 前端收到此錯誤 → 顯示「已邀請過，是否重新發送？」
            invitation_id: existing.id
        }
    }

    // 3. 產生邀請
    let token = generate_crypto_random_token(64);
    let invitation = insert_invitation(email, token, organization);

    // 4. 發送 Email（非同步，不阻塞回應）
    spawn(send_invitation_email(email, token));

    // 5. 回傳邀請資訊 + 連結（給管理員複製）
    return Ok({
        invitation,
        invite_link: format!("{}/invite/{}", base_url, token)
    })
}
```

#### 權限

```
invitation.create   → IACUC_STAFF, SYSTEM_ADMIN
invitation.view     → IACUC_STAFF, SYSTEM_ADMIN
invitation.revoke   → IACUC_STAFF, SYSTEM_ADMIN
invitation.resend   → IACUC_STAFF, SYSTEM_ADMIN
```

### 3.2 邀請接受（客戶端）

#### 公開 Endpoints（無需認證）

```
GET    /api/invitations/verify/{token}    # 驗證邀請碼是否有效
POST   /api/invitations/accept            # 接受邀請並建立帳號
```

#### verify 回應

```json
// 有效邀請
{ "valid": true, "email": "wang@hospital.org", "organization": "台大醫院" }

// 已使用
{ "valid": false, "reason": "already_accepted" }

// 已過期
{ "valid": false, "reason": "expired" }
```

#### 接受邀請 Request

```json
{
    "invitation_token": "a8f3...x9z",
    "display_name": "王大明",
    "phone": "0912345678",
    "organization": "台大醫院",
    "password": "SecurePass123!",
    "position": "主治醫師",
    "agree_terms": true
}
```

> **Email 不在 request 中**：從 invitation 記錄取得，客戶不可更改。
> 連結本身即為 Email 驗證（因為只有收到 Email/連結的人才能存取）。

#### accept 後端邏輯

```rust
fn accept_invitation(req) {
    // 1. 驗證 token
    let invitation = find_by_token(req.token)?;  // 不存在 → 404
    if invitation.status != "pending" { return Error::InvalidToken }
    if invitation.expires_at < now() { return Error::TokenExpired }

    // 2. 建立帳號
    let user = create_user({
        email: invitation.email,        // 從邀請取得，不可改
        display_name: req.display_name,
        phone: req.phone,
        organization: req.organization,
        password: hash(req.password),
        must_change_password: false,     // 已自行設定，不需強制更改
    });

    // 3. 自動分配 PI 角色
    assign_role(user.id, PI_ROLE_ID);

    // 4. 標記邀請為已使用
    update_invitation(invitation.id, {
        status: "accepted",
        accepted_at: now(),
        created_user_id: user.id,
    });

    // 5. 自動登入（回傳 JWT token）
    let tokens = generate_auth_tokens(user);
    return Ok({ user, tokens })
}
```

#### 驗證規則
- Token 存在且狀態為 pending、未過期
- 密碼符合強度要求（≥ 8 字元，含大小寫 + 數字）
- 必須同意服務條款

### 3.3 客戶儀表板

客戶登入後看到的是**簡化版介面**，只顯示與其相關的功能：

#### 客戶完整生命週期與可見頁面

```
階段 1：邀請 → 註冊
  客戶收到邀請 Email → 設定帳號 → 登入

階段 2：撰寫 → 提交（Draft → Submitted）
  可見：我的計劃書、新增計劃書、編輯草稿

階段 3：審查中（Submitted → Pre-Review → Vet → Under_Review）
  可見：計劃書狀態追蹤、審查意見（匿名化）、回覆意見

階段 4：需修改（Revision_Required）
  可見：審查意見 + 修改建議、編輯計劃書、重新提交

階段 5：核准（Approved）→ 實驗進行中
  可見：核准通知、IACUC 編號、我的動物（唯讀）
  ├─ 動物列表（透過 iacuc_no 自動串聯）
  ├─ 動物健康紀錄（觀察、體重、血檢）
  ├─ 手術紀錄
  └─ 匯出報表（醫療、觀察、手術）

階段 6：實驗結束（Closed）
  可見：歷史計劃書、歷史動物紀錄（唯讀歸檔）
```

| 頁面 | 功能 | 權限 | 可見階段 |
|------|------|------|----------|
| `/my-projects` | 我的計劃書列表 | `aup.protocol.view_own` | 2-6 |
| `/my-projects/{id}` | 計劃書詳情 + 審查進度 + 動物列表 | `aup.protocol.view_own` | 2-6 |
| `/my-projects/{id}/animals` | 我的動物（唯讀）| `animal.animal.view_project` | 5-6 |
| `/protocols/new` | 提交新計劃書 | `aup.protocol.create` | 2 |
| `/protocols/{id}/edit` | 編輯草稿 / 修改計劃書 | `aup.protocol.edit_own` | 2, 4 |
| `/profile` | 個人資料設定 | 所有人 | 1-6 |

#### 動物存取控制（已有機制）

```
客戶請求 GET /api/animals?iacuc_no=PIG-114-001
    │
    ▼
Backend 檢查：用戶有 view_project（非 view_all）
    │
    ▼
過濾：只回傳 iacuc_no IS NOT NULL 的動物
    │
    ▼
前端：MyProjectDetailPage 已能顯示動物列表
    │
    ▼
客戶可看：動物基本資訊、健康紀錄、手術紀錄（全部唯讀）
客戶不可：建立/編輯/刪除動物、看到其他計劃書的動物
```

#### 客戶不可見頁面
- ❌ Admin 管理頁面
- ❌ HR 模組
- ❌ ERP / 庫存模組
- ❌ 動物管理（管理功能 — 建立、編輯、匯入）
- ❌ 其他客戶的計劃書與動物
- ❌ 審查委員介面（審查者身分匿名化）

### 3.4 邀請 Email 模板

```
主旨：[豬博士] 邀請您加入實驗動物管理平台

您好，

豬博士動物科技邀請您加入實驗動物管理平台。

透過此平台，您可以：
• 線上提交動物實驗計劃書（AUP）
• 即時追蹤審查進度
• 與審查委員線上溝通

請點擊以下連結完成註冊（連結僅限一次使用）：
{invitation_link}

⏰ 此連結將於 {expires_at} 到期。

如有任何問題，請聯繫我們：
電話：037-433789
Email：{support_email}

豬博士動物科技有限公司
```

> **注意**：邀請 Email 不包含客戶姓名（因為尚未註冊），也不揭露邀請者身分。

---

## 四、執行秘書 AI 輔助標註（Pre-Review 階段）

> 目標：當計劃書進入 Pre-Review 時，AI 自動為執行秘書標註應該注意的地方，降低審查遺漏風險。

### 4.1 觸發時機

```
計劃書狀態變更為 Pre_Review
    │
    ▼
系統自動觸發 AI 分析
    │
    ▼
產生「審查注意事項摘要」
    │
    ▼
顯示在執行秘書的 Pre-Review 介面上方
```

### 4.2 AI 標註內容

為執行秘書標註**應該特別注意**的地方，分為三類：

#### A. 格式與完整性提醒（紅旗 🚩）

```
□ 必填欄位是否有空白或過短的填寫（如「略」「同上」）
□ 3Rs 三項是否都有實質內容（非敷衍）
□ 動物數量是否有統計依據支持
□ 疼痛分類與麻醉方案是否一致
□ 人員是否都有填寫訓練證照
□ 附件是否齊全
```

#### B. 內容疑慮標記（黃旗 ⚠️）

```
□ 動物數量異常偏高或偏低
□ 實驗期程過長（> 2 年）或過短（< 1 週）
□ 疼痛分類偏高（C/D/E 類）的特殊關注提醒
□ 安樂死方法是否為 AVMA 推薦方法
□ 術後照護方案是否足夠具體
□ 人道終點描述是否可執行
□ 前後章節邏輯不一致之處
```

#### C. 審查重點建議（藍旗 ℹ️）

```
□ 「建議特別確認此計劃的統計方法是否合理」
□ 「此物種的麻醉方案建議請獸醫確認劑量」
□ 「本計畫涉及多次手術，建議確認恢復期安排」
□ 「此實驗涉及基因改造動物，注意生物安全規範」
```

### 4.3 UI 呈現 — 執行秘書 Pre-Review 頁面

```
┌──────────────────────────────────────────────────────┐
│  📋 Pre-Review 審查輔助                    APIG-114-003 │
├──────────────────────────────────────────────────────┤
│                                                      │
│  🚩 需要注意（2 項）                                  │
│  ├─ [§動物] 動物數量 N=60，但未提供 power analysis    │
│  │   → 建議要求 PI 補充統計依據                       │
│  └─ [§人員] 協同人員 李OO 未填寫訓練證照編號          │
│      → 建議確認是否持有有效證照                       │
│                                                      │
│  ⚠️ 留意事項（3 項）                                  │
│  ├─ [§設計] 疼痛分類為 D 類，涉及中度以上疼痛         │
│  │   → 止痛方案為 Meloxicam 0.4mg/kg，請獸醫確認      │
│  ├─ [§設計] 實驗包含 2 次存活手術                     │
│  │   → 建議確認手術間恢復期（目前寫 14 天）是否足夠    │
│  └─ [§終點] 人道終點描述較抽象                        │
│      → 「體重減少 20% 或 BCS<2」比「明顯消瘦」更可執行  │
│                                                      │
│  ℹ️ 審查建議（1 項）                                  │
│  └─ 本計畫使用 SPF 小型豬，建議確認飼養環境等級       │
│                                                      │
│  AI 標註僅供參考，請依專業判斷審查                     │
│  [摺疊] [匯出為 PDF 附在審查意見中]                   │
└──────────────────────────────────────────────────────┘
```

### 4.4 技術實作

```
Backend:
  POST /api/protocols/{id}/staff-review-assist

  觸發方式：
  A. 自動 — status 變更為 Pre_Review 時，後端自動呼叫
  B. 手動 — 執行秘書點擊「重新分析」按鈕

  流程：
  1. 讀取 protocol.working_content（JSON）
  2. Level 1：規則引擎檢查（格式、完整性、數值範圍）
  3. Level 2：Claude API（內容品質、邏輯一致性、法規符合性）
  4. 組合結果，儲存至 protocol_ai_reviews 表
  5. 回傳結構化標註結果

  快取：同一 protocol_version 只分析一次，修改後重新分析
```

```sql
CREATE TABLE protocol_ai_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id),
    protocol_version_id UUID REFERENCES protocol_versions(id),
    review_type VARCHAR(20) NOT NULL,  -- 'client_pre_submit' | 'staff_pre_review'
    result JSONB NOT NULL,             -- 結構化標註結果
    model_used VARCHAR(50),            -- 'claude-haiku-4-5' | 'claude-sonnet-4-6'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 4.5 與 AIReview.md 的關係

```
AIReview.md 的 AI 預審                    本章節的執行秘書標註
─────────────────────                    ─────────────────────
對象：客戶（PI）                          對象：執行秘書（IACUC_STAFF）
時機：提交前                              時機：Pre-Review 階段
目的：減少退回次數                         目的：減少審查遺漏
視角：「你的計劃書有這些問題」              視角：「審查時請注意這些地方」
語氣：指導性                              語氣：輔助性
共用：同一張 protocol_ai_reviews 表       共用：同一個 AI service 基礎架構
      review_type = 'client_pre_submit'         review_type = 'staff_pre_review'
```

---

## 五、安全考量

| 項目 | 措施 |
|------|------|
| 邀請碼安全性 | 使用 `crypto-random` 產生 64 字元 token |
| 暴力破解防護 | 驗證 endpoint 加 rate limiter（10 次/分鐘） |
| 邀請過期 | 預設 7 天，可自訂，過期自動失效 |
| 單次使用 | 接受後 token 立即標記為 `accepted`，不可重複使用 |
| 資料隔離 | 客戶只能存取 `pi_user_id = 自己` 的計劃書 |
| 帳號到期 | 可設定帳號有效期限（利用既有 `expires_at` 欄位） |
| 操作審計 | 邀請建立、接受、撤銷均記入 audit log |

---

## 五、實作階段

### Phase 1：邀請後台（預估 2-3 天）

```
□ 建立 invitations migration（含 UNIQUE index on pending email）
□ Backend: invitation model + handler + service
    - create: 檢查 Email 已有帳號 → 回傳 redirect 重設密碼
    - create: 檢查 Email 已有 pending → 回傳 already_invited
    - create: 產生 token + 發 Email + 回傳連結
    - resend: 重新發送 + 回傳連結（供複製到 LINE）
□ Backend: invitation email 模板（HTML + plain text）
□ Frontend: Admin 邀請管理頁面
    - 建立邀請 Dialog（只需 Email + 可選組織）
    - 送出成功後顯示可複製的連結
    - 邀請列表（狀態篩選：pending/accepted/expired）
    - 重新發送、撤銷按鈕
□ 權限設定：IACUC_STAFF 可管理邀請
```

### Phase 2：客戶自助註冊（預估 2-3 天）

```
□ Backend: 公開 verify/accept endpoints（無需認證）
    - verify: 回傳 token 狀態 + Email（前端預填）
    - accept: 建帳 + 自動分配 PI 角色 + 回傳 JWT（自動登入）
□ Frontend: /invite/{token} 頁面
    - 開啟時自動 verify → 顯示 Email（不可編輯）
    - 表單：姓名、電話、組織、職稱、設定密碼
    - 同意服務條款 checkbox
    - 提交後自動登入 → 導向「我的計劃書」
    - 錯誤處理：連結過期/已使用 → 友善提示頁面
□ 服務條款頁面（已有 /terms-of-service）
```

### Phase 3：客戶介面（預估 3-4 天）

```
□ 客戶角色 sidebar 導航簡化
□ 「我的計劃書」頁面優化（已有 /my-projects）
□ 計劃書狀態追蹤 timeline UI
□ 審查意見通知（Email + 站內通知）
□ 計劃書提交引導精靈（可選，降低客戶門檻）
```

### Phase 4：測試與上線（預估 2 天）

```
□ 邀請流程 E2E 測試
□ 權限隔離測試（客戶不可存取其他人資料）
□ Email 發送測試
□ 安全測試（token 暴力破解、過期處理）
□ 上線 + 文件更新
```

### Phase 5：執行秘書 AI 標註（預估 3-4 天）

```
□ 建立 protocol_ai_reviews migration
□ Backend: AI 標註 service（與 AIReview.md 共用架構）
    - 規則引擎：格式/完整性/數值範圍檢查
    - Claude API：內容品質/邏輯一致性分析
    - System prompt 設計（執行秘書輔助視角）
□ Backend: POST /api/protocols/{id}/staff-review-assist
□ Backend: status 變更為 Pre_Review 時自動觸發
□ Frontend: Pre-Review 頁面頂部標註面板 UI
□ Frontend: 「重新分析」按鈕 + loading 狀態
□ 快取機制：同 version 不重複呼叫
□ 測試：mock AI response + 規則引擎 unit test
```

---

## 六、與現有系統整合

### 利用既有元件

| 需求 | 既有元件 | 調整 |
|------|---------|------|
| Email 發送 | `services/email/` | 新增 invitation 模板 |
| 密碼 Hash | `Argon2` in `services/user.rs` | 直接複用 |
| JWT 認證 | `middleware/auth.rs` | 不需調整 |
| 角色分配 | `user_roles` + `services/role.rs` | accept 時自動分配 PI 角色 |
| 帳號到期 | `users.expires_at` | 已有欄位，直接使用 |
| 計劃書表單 | `pages/protocols/ProtocolEditPage` | 不需調整 |
| 我的計劃書 | `pages/my-projects/` | 增強顯示 |
| 審計日誌 | `services/audit.rs` | 新增 invitation 事件類型 |

### 新增檔案清單

```
Backend:
  migrations/0XX_invitations.sql
  src/models/invitation.rs
  src/handlers/invitation.rs
  src/services/invitation.rs
  src/services/email/invitation.rs
  resources/templates/email/invitation.html

Frontend:
  src/pages/auth/InvitationAcceptPage.tsx
  src/pages/admin/InvitationsPage.tsx
  src/pages/admin/components/InvitationCreateDialog.tsx
  src/lib/api/invitation.ts
  src/types/invitation.ts
```
