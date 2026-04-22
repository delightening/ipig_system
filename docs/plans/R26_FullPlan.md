# R26 Service-driven Audit Refactor — FullPlan

> **這份文件的目的**：R26 epic（~465 人時規模的多 PR 系列）的**單一事實來源**。
> 回答三種問題：
> 1. 「為什麼我們當初決定這樣做？」 → §2 Motivation + §3 Decision Log
> 2. 「現在做到哪裡了？後面還有什麼？」 → §4 PR Catalog + §7 Current State + §8 Forward Path
> 3. 「code review 提了什麼我們有沒有處理？」 → §5 Code Review Journey + §6 Carried Debt
>
> **維護規則**（不維護這份文件會喪失追溯力）：
> - 新決策：在 §3 Decision Log 時間序追加新條目，**永不刪除舊條目**（用 "Superseded by ..." 標註取代）
> - 新 PR：在 §4 PR Catalog 新增一列；merge 後更新狀態列
> - Review feedback：解決後更新 §5 Review Response Matrix；新增 carried debt 進 §6
> - Epic 狀態：每完成一個階段（e.g. HR epic done、R26-3 全部 handler 遷完）在 §7 更新
>
> 目標分支：`integration/r26`（長期整合分支，所有 R26 PR 的匯聚點，epic 完成後才 merge 回 main）。

---

## 目錄

- [§1 Executive Summary](#1-executive-summary)
- [§2 Motivation（為什麼要做）](#2-motivation為什麼要做)
  - §2.1 真正起點：釐清 GLP 合規門檻（2026-04-20）
  - §2.2 觸發事件：2026-04-21 Rust 後端架構審查
  - §2.3 為什麼選 Service-driven pattern
  - §2.4 單人長期維護的視角
- [§3 Decision Log（決策過程，時間序）](#3-decision-log決策過程時間序)
- [§4 PR Catalog（實際產出）](#4-pr-catalog實際產出)
- [§5 Code Review Journey（review 建議與回饋）](#5-code-review-journey)
- [§6 Carried Debt & Known Gaps](#6-carried-debt--known-gaps)
- [§7 Current State（截至 2026-04-22）](#7-current-state截至-2026-04-22)
- [§8 Forward Path（合流順序與里程碑）](#8-forward-path合流順序與里程碑)
- [§9 Document Maintenance](#9-document-maintenance)

---

## §1 Executive Summary

- **What**：把 audit log 的寫入邏輯從 handler 層（fire-and-forget `tokio::spawn`）移到 service 層（與資料變更同一 transaction），稱為 **Service-driven audit pattern**。
- **Why（深層動機）**：客戶含藥廠 GLP 實驗室 / CRO，需通過 **21 CFR Part 11 §11.10 audit trail** 條款的 4 項硬要求（secure / time-stamped / independent / records create-modify-delete）。2026-04-20 完成合規盤點（[docs/R26_compliance_requirements.md](../R26_compliance_requirements.md)）→ 2026-04-21 委託 Rust backend 四階段審查（含 Phase 3 GLP 合規）→ 4 Critical 中 3 條明確 GLP 不達標（CRIT-01/02/03）+ WARN-06 + SUGG-03 也提 GLP，這 5 個 finding 共享同一技術根因 → R26 epic 啟動。
- **Scope**：97 個 `AuditService::log_activity / ::log` call sites 橫跨 27 個 handler 檔；原估 ~20 處，訂正為 4.85× 差距；總工時估 ~465 人時。
- **Strategy**：長期 integration branch（`integration/r26`）+ 多 PR 系列，先做 INFRA（PR #153/154）+ pattern demo（PR #155），再按模組複製到 animals / document / hr / user / 其他。Epic 完成後才 PR `integration/r26 → main`。
- **Status as of 2026-04-22**：3 PR merged（INFRA + review-fix + pattern demo），8 PR open（animals / document / hr leave / hr overtime / user / 兩 docs / HMAC 驗證 cron），53 / 97 mutations 已落地或 open。

> **⚠️ 命名警告**：本文件 R26-N 編號（R26-1 ~ R26-12）指 **Service-driven Audit refactor** 的子項。`docs/R26_compliance_requirements.md` 中的「R26」是另一個 epic（資安強化 nice-to-have：2FA / 欄位加密 / 備份演練 / pentest），請勿混淆。詳見 §2.1。

---

## §2 Motivation（為什麼要做）

### 2.1 真正的起點：GLP 合規多階段累積（2026-02 ~ 2026-04-21）

R26 audit refactor 的起點 **不是 2026-04-21 的架構審查報告**。它是 2 個月內三波合規工作累積後，才發現底層 audit log 架構有 gap 的結果。

**業務脈絡**：本系統（`ipig_system`）的目標客戶包含**藥廠 GLP 實驗室 / CRO / 異種器官移植研究**。GLP 客戶若採購本系統作為動物試驗資料管理平台，會被美國 FDA / 歐盟 EMA 稽核員要求出示**電子紀錄與電子簽章合規證明**（21 CFR Part 11）。

#### Wave 1：應用層合規（2026-02-25）

完成 **P1-6 / P1-7** — 兩份正式合規文件：

| 文件 | 範圍 | 結論 |
|---|---|---|
| [docs/security/GLP_VALIDATION.md](../security/GLP_VALIDATION.md) | IQ / OQ / PQ 驗證 | Pass — 系統部署、操作、性能均符合 GLP 驗證流程 |
| [docs/security/ELECTRONIC_SIGNATURE_COMPLIANCE.md](../security/ELECTRONIC_SIGNATURE_COMPLIANCE.md) | 21 CFR Part 11 §11.10 / §11.50 / §11.100 — **電子簽章** | 所有條款 ✓ 已實作（簽章 + 鎖定 + 稽核軌跡 + 雙因素身分綁定） |

**這是 GLP 的「面向客戶可展示的合規」**，當時的判斷是「Part 11 已通過審查」。

#### Wave 2：資安完整盤點（2026-04-20）

commit `e710265` 產出 [docs/security/SECURITY_COMPLETED.md](../security/SECURITY_COMPLETED.md) — **彙整 46 項已完成資安強化**（橫跨 7 大面向：認證授權 / 密碼憑證 / SQL/XSS/CSRF / Rate Limit / 稽核告警 / CI Secrets / 合規）。

同日另寫 [docs/R26_compliance_requirements.md](../R26_compliance_requirements.md)（**檔案 untracked，git 中找不到**）— 對 6 大法規框架（21 CFR Part 11 / SOC 2 / ISO 27001 / GDPR / HIPAA / PCI DSS）做**適用決策樹**分析。結論：

> 「本專案目前主要場景為動物試驗 / 畜牧管理，若客戶為藥廠 GLP 實驗室 → **21 CFR Part 11 優先**；若僅本地實驗 → SOC 2 Type II 作為最通用背書。」

**這次盤點的真正價值不是發現新問題**，而是把 46 項打散的 audit / signature / authz work 集中起來，發現一件事：

> **「Application-layer security 已飽和，但 architectural-layer audit log 還沒被深入檢視。」**
>
> Part 11 §11.10 雖然在電子簽章層面合規（Wave 1 ✓），但要求「audit trail 涵蓋 create / modify / delete electronic records」是**全系統範圍**。簽章只是 record 的子集；資料 CRUD 本身的 audit trail 是否同樣達標？

#### Wave 3：架構層審查（2026-04-21）

承接 Wave 2 的問句，於 2026-04-21 委託 Rust backend 四階段全流程審查（架構 / 並發 / **GLP 合規** / 維護）— 詳見 §2.2。

#### 21 CFR Part 11 §11.10 audit trail 條款 — R26 要解什麼

對 audit log 的硬要求：
> "Use of **secure, computer-generated, time-stamped audit trails** to independently record the date and time of operator entries and actions that **create, modify, or delete electronic records**."
>
> "Such audit trail documentation shall be retained for a period at least as long as that required for the subject electronic records."

對應 4 個關鍵能力 → R26 audit refactor 的目標：
1. **Secure**（不可竄改）→ HMAC chain（Wave 1 已部分實作；R26-2 加每日驗證 cron 強化偵測）
2. **Computer-generated, time-stamped**（自動產生、精確時戳）→ Wave 1 已實作
3. **Independent**（無法被繞過、不會半成品）→ **R26 主戰場**：必須與資料變更同 tx，不能 fire-and-forget
4. **Records create/modify/delete**（含完整 before/after）→ **R26 主戰場**：必須有 snapshot 而非僅 metadata

> **⚠️ 命名碰撞警告**：
>
> 1. [`docs/R26_compliance_requirements.md`](../R26_compliance_requirements.md) 中的「**R26**」指**另一條 epic — 資安強化 nice-to-have**（R26-1 強制 2FA / R26-2 欄位加密 / R26-3 備份還原演練 / R26-4 外部 pentest）。與本文件的「**R26 Service-driven Audit refactor**」是兩個不同 epic。
> 2. 本文件之後所有 R26-N 編號（R26-1 ~ R26-12）皆指 **Service-driven Audit** 的子項，**不要**和合規 doc 的 R26-1 ~ R26-4 混淆。
> 3. 為什麼會碰撞：TODO.md 的 章節編號是按時間流水號（R20 ~ R26 ~ R27 ...），4/20 寫合規 doc 時取 R26、4/21 寫 audit refactor TODO 時也取 R26（因為前者未進入 TODO 主章節）。歷史包袱，不改動。

### 2.2 觸發事件：2026-04-21 Rust 後端架構審查

承接 §2.1 的合規動機，於 2026-04-21 委託對 Rust backend（`erp-backend` v0.1.0, axum 0.7 + tokio + sqlx 0.8，~230 個 `.rs` 檔）做**四階段全流程審查**，4 階段為：

1. **架構盤點**（layered design / module boundaries / 依賴方向）
2. **並發模型**（tokio worker / locking / spawn_blocking 應用 / 背景任務生命週期）
3. **GLP 合規**（audit trail 完整性 / 資料變更可還原 / electronic signature / 21 CFR Part 11 §11.10 對齊）
4. **長期維護**（單人視角 / 檔案大小 / clippy 規則 / 死碼）

產出 [docs/reviews/2026-04-21-rust-backend-review.md](../reviews/2026-04-21-rust-backend-review.md)，**Phase 3 GLP 合規**抓出 4 個 Critical，3 個明確標記 GLP 不達標：

- **CRIT-01｜IACUC 編號產生 race condition**
  > 「GLP 情境下，編號的連續性與唯一性是稽核重點」
  - 對應 §11.10(a)（系統驗證）+（編號產生不確定性違反「準確紀錄」）
- **CRIT-02｜多表變更未包 transaction，GLP 合規風險**
  > 「GLP 要求『資料變更必有完整稽核軌跡且可還原』，此設計違反此要求」
  - 對應 §11.10(b)+(c)（紀錄完整性 + 可還原性）
  - **這條最痛**：`pool.begin()` 僅出現於 8/80 service 檔，覆蓋率 10%
- **CRIT-03｜UPDATE 類稽核日誌無 before/after snapshot**
  > 「GLP 要求對資料變更保留『變更前 vs 變更後』完整快照」
  > 「稽核員要求『能從稽核軌跡還原任一時刻的資料狀態』，目前做不到 → Critical」
  - 對應 §11.10(c)（可產生準確、完整的紀錄副本）+ §11.10(e)（操作型 audit trail）
- **CRIT-04**：`services/mod.rs` crate-wide `#![allow(dead_code)]` — 與 GLP 無直接關係，是 CLAUDE.md 內部規則違反

**WARN 中與 GLP 相關**：
- **WARN-06｜audit 使用 `tokio::spawn` fire-and-forget**
  > 「3. GLP 要求稽核寫入可靠性，fire-and-forget 不符合『可靠寫入』」
  - 對應 §11.10 對「Independent」的要求（spawn 後失敗只記 tracing::error，稽核員查不到）

**SUGG 中與 GLP 相關**：
- **SUGG-03｜HMAC chain 缺自動驗證**
  > 「GLP 稽核時，若有人繞過 trigger 直接改 DB，HMAC chain 會斷 — 但得有人主動驗證才發現」
  - 對應 §11.10(b) 對 audit trail 「Secure」的要求

**審查報告結論**（單人維護視角下最重要的三件事）：
> 「1. CRIT-01 / CRIT-02 — 這是 **GLP 稽核最可能被挑戰的地方**」
>
> 「2. CRIT-04 + WARN-01 — 把 `#[allow(dead_code)]` 移掉後，會暴露多少未用碼是個信號」
>
> 「3. WARN-02（Argon2）— 登入路徑效能」

**這就是 R26 epic 的觸發點**：CRIT-01 + CRIT-02 + CRIT-03 + WARN-06 + SUGG-03 五個 GLP 相關 finding 必須一起解，因為它們都指向同一個技術根因 — audit log 寫入不在資料變更的 tx 內，且不能完整保留變更內容。

### 2.3 為什麼選 Service-driven pattern 而不是別的

考量過的替代方案：

| 方案 | 捨棄原因 |
|---|---|
| DB trigger 自動寫 audit | 和現有 HMAC chain 設計衝突（trigger 無法存取 actor context / 無 redact 能力） |
| Handler 層加 `tokio::spawn(audit).await` | 這只是把 fire-and-forget 改成 await，仍不能保證 audit 與資料變更同一 tx（CRIT-02 不解） |
| Middleware 層 wrap handler 攔截 | 無法獲得 mutation 前後的資料 snapshot（CRIT-03 不解） |
| **Service 層單 tx 內寫 audit**（採用） | ✅ 資料變更 + audit + HMAC chain 原子；✅ 有完整 before/after；✅ 失敗自動 rollback |

### 2.4 單人長期維護的視角

所有決策都要過這個濾鏡：

- **不要為完美而 over-engineer**：例如 `AuditRedact` 選「doc warning + code review」而不是 compile-time macro（後者 > 1 週工時）
- **R26-3 progressive migration**：舊 `log_activity` 標 `#[deprecated]`，允許 ~80 個 call sites 漸進遷移；不一次性改完所有 handler 以免單次 PR 失控
- **長期 integration branch**：隔離半成品狀態，避免污染 main，也能安全放棄

---

## §3 Decision Log（決策過程，時間序）

> 規則：新決策追加到**底部**，不修改舊條目；若推翻舊決策，在舊條目下方新增「⚠️ Superseded by D-XX」。

### D-01 [2026-04-21] 長期 integration branch `integration/r26`

**決策**：所有 R26 相關 PR 的 base 分支為 `integration/r26`，**非 main**。Epic 完成後（所有 8 個 R26 子項均 done），才發起 `integration/r26 → main` 單一 merge PR。

**理由**：
1. R26 是多 PR 系列（估計 ~10 個 PR），持續數週；main 若每個 PR 都合入會造成半成品狀態曝光。
2. AI reviewer（Gemini + CodeRabbit）feedback 產生 follow-up PR，不應打斷 main 穩定度。
3. 若 epic 中途發現重大問題，可直接捨棄整個 `integration/r26` 而不污染 main。

**副作用**：
- CI workflow 需加 `integration/**` trigger（PR #154 commit `9b72d7b`）
- CodeRabbit 對非預設 base 分支顯示 "Review skipped"，每個 PR 需手動留言 `@coderabbitai review` 觸發
- PR description 需註明「不直接合併到 main」

### D-02 [2026-04-21] INFRA 先行（PR #153），pattern demo 後發（PR #155）

**決策**：把 `ActorContext` / `DataDiff` / `AuditRedact` / `log_activity_tx` / `CancellationToken` 這些基礎設施做成 **PR #153** 先合入；再用 **PR #155** 示範一個完整 Service-driven mutation（`ProtocolService::submit`），作為後續 10 個模組 PR 的複製模板。

**理由**：先確定 API shape 能穩定承載所有後續 PR，避免每個模組 PR 都來回調整 INFRA。

**確認機制**：PR #155 被明確標為 "pattern demo"，reviewer 重點放在「這個模板好不好複製」，不是「這個 mutation 的業務邏輯」。

### D-03 [2026-04-21] 採用「hybrid actor guard」策略

**決策**：Service 層的 mutation 方法根據操作性質選擇不同的 actor 守門：
- **強制使用者操作**（submit / approve / reject / delete 敏感資料）：`actor.require_user()?`
- **CRUD 類**（可接受 System 觸發，例如 batch 匯入、內部自動化）：`actor.actor_user_id().unwrap_or(SYSTEM_USER_ID)` 允許 User 或 System

**理由**：PR #155 初版全用 `require_user`，但 PR #156 實作時發現 `AnimalWeightService::create` 在動物建立時會被 `AnimalService::create` 自動呼叫（系統行為），若強制 User 會破壞現有流程。

**Codex review #7 建議**：「按操作性質選 guard」，而非統一。被採納。

**⚠️ Superseded by D-09**：CodeRabbit 在 PR #156 指出 `unwrap_or(SYSTEM_USER_ID)` 會把 Anonymous actor 靜默降級成 System — 需明確 match，拒絕 Anonymous。預計在後續 PR 修正。

### D-04 [2026-04-21] AuditRedact 用「doc warning」而非 compile-time macro

**決策**：`AuditRedact` trait 的預設 impl 是空的 `redacted_fields() -> &[]`；trait 層 doc 明列「以下型別 MUST 覆寫：User / Session / JwtBlacklist / TwoFactorSecret / McpKey / Partner / OAuthCredential」。透過 **code review** 而非編譯器強制。

**替代方案**（未採用）：
- `#[must_use]`-style marker — 無法表達「X 型別必須覆寫 Y 方法」的語意
- Proc macro `#[derive(AuditRedact)]` + 自動掃 `password_hash` 等 field name — 工時 > 1 週，得失不對等
- `trait sealed` + 手動 impl — 雜訊高、無助安全

**來源**：Codex review #2（🔴 Blocker），PR #156 第一個 commit 修復。

**⚠️ Superseded by D-10**：PR #156 裡 CodeRabbit 對 `CareRecord`（醫療紀錄）和 `VetAdviceRecord`（自由文字）空 impl 給 🟠 Major feedback，認為連業務資料都該用 allowlist 或至少檢視。後續 PR 會補。PR #162 的 `User` impl 是第一個非空 `redacted_fields()` override，作為未來範例。

### D-05 [2026-04-21] `upsert` 拆為 `create_or_update`（取得完整 before/after）

**決策**：遇到 `INSERT ... ON CONFLICT DO UPDATE` 類 upsert，改寫為「先 SELECT FOR UPDATE → 判斷 existence → INSERT 或 UPDATE」兩分支流程。

**Tradeoff**：多一次 SELECT round-trip，換取 `DataDiff::compute(before, after)` 的完整性。

**首次落地**：PR #156 `AnimalVetAdviceService::upsert → create_or_update`。

**⚠️ CodeRabbit PR #156 Major feedback**：SELECT FOR UPDATE 在 row 不存在時不鎖任何東西，併發兩個 request 可能同時走到 None 分支 → 都 INSERT → 其中一個撞 UNIQUE。**Gap**：需改鎖父層 animal row 或改用原子 upsert。預計在遷移 upsert 類時統一處理。

### D-06 [2026-04-21] 舊 API `log_activity(&pool, ...)` 用 `#[deprecated]` 漸進遷移，不一次性刪除

**決策**：PR #153 保留舊 API + 加 `#[deprecated]` 標籤；CI clippy 加 `-A deprecated` flag（避免 91 個 warnings 變成 errors）；各模組 PR 漸進把 handler 從舊 API 改到 `log_activity_tx`；最後的清理 PR（R26-4）才刪除舊 API + 從 CI 移除 `-A deprecated`。

**理由**：一次性刪除需改 20+ handler（實際 97 處），單一 PR 不可 review、失敗代價太高。

**進度量測**：`cargo build 2>&1 | grep "use of deprecated" | wc -l`（PR #153 後 91 → 預計 R26-4 merge 後 0）。

### D-07 [2026-04-21] `ActivityLogEntry` 加 `Default` + 4 constructors（`update` / `create` / `delete` / `simple`）

**決策**：第 5-field struct 有 3 個 `Option`，pattern demo 寫出來發現每個 call site 都要填 `request_context: None`。加 constructors 讓常見 case 一行寫完。

**落地**：PR #156 第一個 commit（基礎建設升級）。

**效益量測**：~45 個 call sites × 4 行冗詞 → 節省 ~180 行樣板。

**來源**：Codex review #1（🟡 Warning）。

### D-08 [2026-04-21] soft_delete 留在 simple PR 而非 moderate

**決策**：soft_delete（`change_reasons INSERT` + 主表 `UPDATE`）涉及 2 表，技術上是 moderate，但視為 audit trail 伴生，放進同一 PR（#4a）處理。

**理由**：`change_reasons` 本身就是 audit 的一部分，與主表 mutation 強耦合；拆開 PR 反而破壞語意。

### D-09 [2026-04-22] 「CodeRabbit PR #156」:「Anonymous actor 不應靜默降級成 System」

**決策**：將 D-03 的 `actor.actor_user_id().unwrap_or(SYSTEM_USER_ID)` 模式改為明確 match，拒絕 Anonymous：

```rust
let created_by = match actor {
    ActorContext::User(u) => u.id,
    ActorContext::System { .. } => SYSTEM_USER_ID,
    ActorContext::Anonymous => {
        return Err(AppError::Forbidden("此操作不接受 Anonymous".into()));
    }
};
```

**落地**：預計在後續 animal / hr / user 模組 PR 統一套用（目前 PR #156 保留舊模式，待統一處理）。

### D-10 [2026-04-22] 「CodeRabbit PR #156」:「CareRecord / VetAdviceRecord 空 impl 不夠保守」

**決策**：承認空 `AuditRedact` 對醫療紀錄 + 自由文字欄位風險過高。未來遇到含大量自由文字或 JSON 欄位的 entity，需評估三個選項：
- (A) 完整 allowlist（`data_diff` 只含特定欄位）
- (B) 明確 redact 自由文字欄位的內容 + 保留欄位名（這是現有 `AuditRedact` 的模式）
- (C) 在 service 層改用 summary log（只記 event + entity name，不記完整 content）

**首次應用 (B)**：PR #162 `User` entity — `password_hash` / `totp_secret_encrypted` / `totp_backup_codes` 走 redact。作為未來複製範例。

### D-11 [2026-04-22] document approve 跨 service tx 串接保留，內部 audit 延後

**決策**：`DocumentService::approve` 會呼叫 `StockService::process_document(&mut tx)` + `AccountingService::post_document(&mut tx)`。PR #5a（#159）只 audit Document 層的狀態變更，**不** 在同個 PR 做 Stock / Accounting 的內部 audit。

**理由**：擴張 scope 會讓 PR #5a 從 ~60 人時膨脹至 ~100+ 人時；Stock/Accounting 內部 audit 是獨立業務域，留給 R26-3 延伸（預計 PR #6+）。

**風險標記**：`approve` 現在是全原子 —— audit / stock / accounting / document 狀態**任一**失敗會 rollback 全部。這是 GLP 要求的正確行為，但需要團隊理解「以前 audit 失敗不擋主流程，現在會擋」。

### D-12 [2026-04-22] Batch operation 的 audit 粒度選 N+1（per-row + summary）

**決策**：批次操作（`recalculate_all_po_receipt_status`、`batch_auto_calculate_annual_leave` 等）的 audit 採「每個 affected row 各記一筆 + 最後一筆 batch summary」，而非 per-row only 或 summary only。

**理由**：
- Per-row only：批次若 1000 筆，audit 表膨脹、難以查詢「這次 batch 做了什麼」
- Summary only：稽核員無法回查「某隻豬為什麼在這次 batch 被 recalc」
- **N+1**：兩邊需求都能滿足，成本可控

**首次應用**：PR #5a（#159）`DOCUMENT_PO_RECEIPT_RECALC` summary + 各 PO 獨立 audit；PR #5c（#161）`ANNUAL_LEAVE_BATCH_AUTO_CALC`。

**⚠️ 衝突註記**：PR #163 Gemini Medium feedback 指出 PROGRESS 描述在「batch summary」和「summary + N+1」之間不一致 — 正確是 N+1，需在 PROGRESS 統一措辭。

### D-13 [2026-04-22] PR #159 改用 struct literal（避免 PR #156 rebase 衝突）

**決策**：PR #159 document 遷移時**不用** `ActivityLogEntry::update(...)` constructor，改用 struct literal `ActivityLogEntry { ... }`。

**理由**：PR #156 第一個 commit 新增 constructors；PR #159 fork 自 `integration/r26` 時 #156 還沒 merge。若 #159 也用 constructor，兩 PR 會在 `audit.rs` 同區塊衝突。用 struct literal 無需 import constructor，rebase 友善。

**Post-merge plan**：兩 PR 都 merge 後，可以把 struct literal 的 call sites 改用 constructor 清理一致性（非 blocker）。

### D-14 [2026-04-22] 2026-04-22 四個並行 PR（#5a/#5b/#5c/#6a）共用 base + 彼此獨立

**決策**：同一天開出 `document` / `hr/leave` / `hr/overtime` / `user` 四個 PR，全部從 `integration/r26` HEAD 分支，**彼此模組獨立不重疊**，可並行 review + merge。

**理由**：到 2026-04-22 時 pattern 已經 proven（PR #155 + #156 pattern 穩定），四個模組的 CodeRabbit 審閱量小（平均 2-4 comments / PR），並行風險低。

**PROGRESS 衝突處理政策**：PR #163（記錄這四個 PR）和 PR #157（記錄 PR #156）都會在 `PROGRESS.md` §9 頂端加 2026-04-22 條目。兩者不可並存於同一版本，whichever merges 第二就要 rebase — 解決方式：反向時間序排列（later-opened PR 的條目排在 earlier-opened PR 條目之下）。

---

## §4 PR Catalog（實際產出）

### 4.1 Overview Table

| PR | 類型 | Head Branch | Commits | Mutations | 狀態 |
|----|-----|-------------|---------|-----------|------|
| [#153](https://github.com/delightening/ipig_system/pull/153) | INFRA | `claude/lucid-maxwell-a9f861` | 10 | 0 | ✅ Merged 2026-04-21 |
| [#154](https://github.com/delightening/ipig_system/pull/154) | INFRA fix | `fix/pr1-review-feedback` | 8 | 0 | ✅ Merged 2026-04-21 |
| [#155](https://github.com/delightening/ipig_system/pull/155) | Pattern Demo | `feat/protocol-service-driven` | 6 | 1 (Protocol::submit) | ✅ Merged 2026-04-21 |
| [#156](https://github.com/delightening/ipig_system/pull/156) | Module #4a animals simple | `feat/animals-simple-mutations` | 8 | 17 | 🟡 Open（CI clean） |
| [#157](https://github.com/delightening/ipig_system/pull/157) | Docs #4a aftermath | `docs/r26-pr4a-aftermath` | 1 | - | 🟡 Open |
| [#158](https://github.com/delightening/ipig_system/pull/158) | R26-2 Chain verify cron | `feat/audit-chain-verify-cron` | 1 | 0 | 🟡 Open（含 2 🔴 Critical reviews 待處理） |
| [#159](https://github.com/delightening/ipig_system/pull/159) | Module #5a document | `feat/document-service-driven-audit` | 5 | 10 | 🟡 Open |
| [#160](https://github.com/delightening/ipig_system/pull/160) | Module #5b hr/leave | `feat/hr-leave-service-driven-audit` | 3 | 7 + 7 balance helpers | 🟡 Open |
| [#161](https://github.com/delightening/ipig_system/pull/161) | Module #5c hr/overtime | `feat/hr-overtime-service-driven-audit` | 4 | 14 | 🟡 Open |
| [#162](https://github.com/delightening/ipig_system/pull/162) | Module #6a user | `feat/user-service-driven-audit` | 3 | 4 | 🟡 Open（CI running） |
| [#163](https://github.com/delightening/ipig_system/pull/163) | Docs four-PR record | `docs/r26-pr5-6a-aftermath` | 1 | - | 🟡 Open |

### 4.2 Per-PR Scope Detail

#### PR #153 — INFRA
- `ActorContext` enum + `SYSTEM_USER_ID` + migration 033
- `DataDiff` + `AuditRedact` trait（length-prefix canonical encoding、JSON Pointer、11 tests）
- `log_activity_tx` tx 版本 API + `ActivityLogEntry` struct（取代 11 位置參數）
- Advisory lock `pg_advisory_xact_lock(hashtext('audit_log_chain'))`
- `(created_at, id)` tuple tiebreaker for prev_hash ordering
- Migrations 034（impersonated_by_user_id column）、035（log_activity v2 stored proc）
- `CancellationToken` 貫穿 `AppState` / `main.rs` / `JwtBlacklist::start_cleanup_task` / 14 cron jobs
- 建 integration branch `integration/r26`
- Handler 尚**未**遷移（下個 PR 才做）

#### PR #154 — Review feedback（Gemini + CodeRabbit）
- `unwrap_or(None)` 吞 DB error 改 `?` 傳播
- HMAC payload 改 `HmacInput<'_>` struct + length-prefix canonical encoding（防碰撞）
- Migration 036 `changed_fields` fallback: EXCEPT → UNION + `IS DISTINCT FROM` + `jsonb_typeof` 守衛
- `main.rs` jwt_cleanup timeout log/alignment + named const
- `.coderabbit.yaml` `zh-TW` → `zh`
- CI `-A deprecated` + `integration/**` trigger
- `.claude/hooks/block-dangerous.sh`（Python shlex 避開 commit message 誤攔）

#### PR #155 — Pattern Demo
- `ProtocolService::submit()` 完整 Service-driven 改寫
  - `&ActorContext` 參數 + `actor.require_user()` 守門
  - 單 tx：SELECT FOR UPDATE / INSERT protocol_versions / generate_apig_no + advisory lock / UPDATE protocols / record_status_change_tx / log_activity_tx
- CRIT-01 IACUC race 完整修復：3 個 numbering generator 接 `&mut Transaction` + 共用 `pg_advisory_xact_lock(hashtext('protocol_iacuc_number_gen'))`
- CRIT-04 `#![allow(dead_code)]` crate-level 移除；10 處 per-item `#[allow(dead_code)]` + 理由（delay 到 R26-7）
- `impl AuditRedact for Protocol`
- Handler `submit_protocol` 改傳 `ActorContext::User(...)`
- `change_status` 延後到 R26-8（300+ 行、涉及 PartnerService）
- 產出 `docs/plans/pr4a-animals-simple-mutations.md` + `pr5-hr-document-roadmap.md`

#### PR #156 — Animals simple mutations（17）
- 涵蓋：`source`（3）/ `weight`（3）/ `vet_advice`（3 + create_or_update 拆分）/ `care_record`（3）/ `vaccination`（3）/ `observation`+`surgery` create（2）
- 16 個 animal entity AuditRedact 空 impl
- `ActivityLogEntry::{update,create,delete,simple}` constructors
- 內部 caller 適配：`AnimalService::create` 初始體重、`import_export.rs` 批次匯入改用 `ActorContext::System { reason: "..." }`
- Handler 13 處 audit 呼叫移除

#### PR #157 — Docs aftermath
- PROGRESS §9：2026-04-22 PR #4a 條目
- TODO R26-3 數字訂正：~20 → 97 call sites × 27 檔
- 新增 `docs/plans/pr5a-document-mutations.md`（document 模組執行計畫，5-commit 切分）

#### PR #158 — R26-2 HMAC chain daily verify cron
- `AuditService::verify_chain_range(pool, from, to)` 公開驗證 API
- `ChainVerificationReport` / `BrokenChainLink` 結構
- `AuditService::compute_hmac_for_fields()` pub(crate) 純運算
- 新檔 `services/audit_chain_verify.rs`（`verify_yesterday_chain` + 2 單元測試）
- `scheduler.rs` 加 `register_audit_chain_verify_job` cron（`0 0 2 * * *` 每日 02:00 UTC）
- 斷鏈時 INSERT `security_alerts` + `SecurityNotifier::dispatch`

#### PR #159 — Document（10）
- `crud`（3：create / update / delete，8-table cascade delete）
- `workflow`（5：submit / approve / admin_approve / admin_reject / cancel）
  - `approve` 保留跨 service tx（StockService + AccountingService 同 tx）
- `grn`（2：create_additional_grn / recalculate_all_po_receipt_status）
  - Recalculate 用 N+1 audit 粒度（per-PO + summary）
- 移除 `audit_document` helper + 8 handler fire-and-forget 呼叫
- Deprecated warnings 97 → ~87

#### PR #160 — HR leave（7 mutations + 7 balance helpers）
- 7 leave mutations 從 0-tx baseline 改為 Service-driven
- 7 balance helpers（deduct / apply / restore × annual_leave + comp_time）改接 `&mut Transaction`
- Event 細分：`LEAVE_APPROVE_INTERIM` vs `LEAVE_APPROVE_FINAL` / `LEAVE_CANCEL` vs `LEAVE_CANCEL_RETROACTIVE`
- FOR UPDATE on entitlement + balance 防併發 over-deduction
- Lock order 紀律：leave row → entitlement（PR #5c 確認沒反向）

#### PR #161 — HR overtime + balance + attendance（14）
- Overtime（6）：`approve_overtime` 現在單 tx 包含 SELECT FOR UPDATE / UPDATE RETURNING / INSERT overtime_approvals / 條件 INSERT comp_time_balances / audit
- Balance（5 + 1 summary）：`batch_auto_calculate_annual_leave` 用 N+1（per-user `ANNUAL_LEAVE_CREATE` + final `ANNUAL_LEAVE_BATCH_AUTO_CALC`）；batch 跨 user **非**原子
- Attendance（3）：clock_in / clock_out / correct 改用 SELECT FOR UPDATE + RETURNING（取代 ON CONFLICT UPSERT）
- 關閉 HR epic（#5b + #5c = 21 mutations）

#### PR #162 — User（4）
- 4 mutations + 7 handler audit 呼叫 consolidation
- **首個 non-empty `AuditRedact::redacted_fields()` override**：`User` 型別 redact `password_hash` / `totp_secret_encrypted` / `totp_backup_codes`
- Event 細分：`USER_UPDATE`（ADMIN 類別，always）+ 條件 `USER_STATUS_CHANGE`（is_active 切換，SECURITY 類別）+ 條件 `USER_ROLE_CHANGE`（role_ids 變更，SECURITY 類別，搭配 `RoleAssignmentSnapshot` helper）
- 2 個 handler `tokio::spawn(log_activity(SECURITY))` 改為同 tx 原子
- Reset_password / impersonate 延後到 PR #6d（涉及 AuthService）
- Deprecated 89 → 86

#### PR #163 — Docs four-PR record
- PROGRESS.md §9 新增 2026-04-22 四 PR 條目（35 service mutations、HR epic closure）
- Deprecated 89 → 86
- 預期和 PR #157 在 §9 頂端有 rebase conflict（解決政策：反向時間序）

---

## §5 Code Review Journey

### 5.1 Aggregate Review Statistics

跨 11 個 PR 的 reviewer feedback（Gemini + CodeRabbit）：

| Severity | Count |
|----------|-------|
| 🔴 Critical (CodeRabbit Critical, Gemini high+) | **2**（都在 PR #158） |
| 🟠 Major (CodeRabbit Major, Gemini high / security-medium+medium) | **23** |
| 🟡 Medium (CodeRabbit Minor, Gemini medium) | **43** |
| 🔵 Nit (CodeRabbit nitpick) | **1** |
| 其他（Note） | **1** |
| **Total** | **70** |

每 PR 平均 ~6.4 comments。PR #153（初版 INFRA）和 PR #158（獨立驗證邏輯）是 comment 最多的兩個（19 + 13）；pattern 成熟後的 PR（#159-#163）平均只有 2-4 comments。

### 5.2 Review Response Matrix — 已處理

| PR | Reviewer | Severity | 位置 | 問題 | Response / Status |
|----|----------|----------|------|------|-------------------|
| #153 | Gemini | 🟠 high | `audit.rs:359` | `unwrap_or(None)` 吞 DB error | ✅ PR #154 commit `90a2419` 改 `?` |
| #153 | CodeRabbit | 🟠 Major | `audit.rs:383` | HMAC 字串串接碰撞 | ✅ PR #154 commit `90a2419` 改 `HmacInput` struct + length-prefix |
| #153 | Gemini | 🟠 security-medium | `audit.rs:389` | HMAC 構造洩漏風險 | ✅ PR #154 同上 |
| #153 | Gemini | 🟡 medium | migration 035 | stored proc changed_fields 漏偵測刪除 key | ✅ PR #154 migration 036 UNION 修正 |
| #154 | Gemini | 🟡 medium | migration 036 | `jsonb_object_keys` 對非-object 會 runtime error | ✅ PR #154 commit `761175c` 加 `jsonb_typeof` 守衛 |
| #155 | Gemini | 🟡 medium | `status.rs:197` | submit 可能產生兩筆 audit | ✅ 確認：record_status_change_tx 和 log_activity_tx 是**不同層級** 的 audit（protocol_activities vs user_activity_logs），各自獨立合理 |
| #155 | Gemini | 🟡 medium | `numbering.rs:207` | `generate_iacuc_no` prefix 邏輯不一致 | ⏳ 延至 R26-8（change_status 重構時統一）|
| #156 | Codex (pre-review) | 🔴 Blocker | AuditRedact 空 impl 安全性 | Pattern demo 前預警 | ✅ PR #156 第一個 commit 加強化 doc warning（trait 層說明必須覆寫的型別清單） |
| #156 | Codex | 🟡 Warning | ActivityLogEntry ergonomics | 45 call sites 冗詞 | ✅ PR #156 第一個 commit 加 4 constructors |

### 5.3 Review Response Matrix — 延至後續 PR 處理

| PR | Reviewer | Severity | 位置 | 問題 | 追蹤項目 |
|----|----------|----------|------|------|----------|
| #154 | Gemini | 🟠 security-medium | `audit.rs:414` | HMAC 版本化缺失 — length-prefix 格式後舊 row 驗證會失敗 | **R26-6**（本 doc §6.1） |
| #154 | CodeRabbit | 🟠 Major | `audit.rs:82` | HMAC 用 pre-insert 欄位算，和持久化後可能分歧 | **R26-6**（合併議題） |
| #155 | Gemini | 🟡 medium | `numbering.rs:114` | 兩次獨立 query 取 APIG + PIG 可優化 | **R26-8**（change_status 重構一併） |
| #156 | CodeRabbit | 🟠 Major | `care_record.rs:64` | CareRecord 空 impl 對醫療內容太寬鬆 | **新追蹤 R26-9**（entity redact allowlist review）|
| #156 | CodeRabbit | 🟠 Major | `medical.rs:49` | Anonymous actor 靜默降級 System | **D-09 applied next PR**（animals / hr 統一）|
| #156 | CodeRabbit | 🟠 Major | `vet_advice.rs:33` | AnimalVetAdvice / VetAdviceRecord 自由文字空 impl | **R26-9** 同上 |
| #156 | CodeRabbit | 🟠 Major | `vet_advice.rs:124` | FOR UPDATE 對不存在 row 不鎖 → 併發兩個 INSERT 撞 UNIQUE | **新追蹤 R26-10**（upsert pattern 安全化）|
| #156 | CodeRabbit | 🟠 Refactor | `vet_advice.rs:124` | Service 層直接寫 SQL（違反 repositories/ 分層） | 延至 P2-4（審查報告 WARN-01）long-running |
| #156 | CodeRabbit | 🟠 Major | `vet_advice.rs:330` | `delete` service 層未檢查專案/動物範圍授權 | **新追蹤 R26-11**（Service-layer 授權補強）|
| #157 | Gemini | 🟡 medium | pr5a plan | plan 提到 `data_diff.after` meta 欄位，目前 DataDiff 未 expose | ⏳ 計畫修正，PR #159 用 struct literal 不觸發 |
| #157 | Gemini | 🟡 medium | pr5a plan | 同 tx 內 created_at 會碰撞（NOW()） | ⏳ 計畫需改措辭，實作不受影響 |
| #158 | CodeRabbit | 🔴 Critical | `audit.rs:517` | `ChainRow.actor_user_id: Uuid` 遇 NULL panic | **必須 PR #158 自身修**（merge 前 blocker）|
| #158 | CodeRabbit | 🔴 Critical | `audit.rs:581` | verifier prev_hash 推進規則和寫入端不一致 | **必須 PR #158 自身修**（merge 前 blocker）|
| #158 | Gemini | 🟠 high | `audit.rs:555` | NULL integrity_hash 略過時機 | 同上 |
| #158 | CodeRabbit | 🟠 Major | 多處 | refactor 50+ 行 fn / Uuid::nil() / repository 層 / legacy HMAC 誤報 | **PR #158 需追加 commit 處理** |
| #159 | Gemini | 🟡 medium | `crud.rs:197` | DOCUMENT_CREATE audit 只 capture Document header，漏 lines | **新追蹤 R26-12**（document lines audit completeness）|
| #160 | Gemini | 🟠 high | `leave.rs:224` | `update_leave` 缺 total_hours/days 倍數驗證 | 審閱中，預計 PR #160 追加 commit |
| #162 | Gemini | 🟡 medium | `user.rs:54/240` | `require_user()?` 阻擋系統自動化（背景停用過期帳號） | **D-03 hybrid actor** 應在 user 模組也套用（`create` / `deactivate_self` 需檢視是否該放寬）|
| #163 | Gemini | 🟡 medium | PROGRESS.md:196 | batch summary vs N+1 措辭不一致 | 文字修正，PR #163 追加 commit |

### 5.4 Reviewer 使用經驗總結

| Reviewer | 特色 | 適用場景 |
|----------|------|----------|
| **Gemini (Google AI Code Review)** | 快、覆蓋面廣、傾向 high-level design（lock order、validation、audit 粒度） | 所有 PR 都跑；尤其能抓「描述 vs code 不符」 |
| **CodeRabbit** | 嚴、愛標 Major、會寫 code suggestion、常指 GLP / 安全 | 非預設 base 需手動觸發；對核心 audit/safety 邏輯最有價值 |
| **Codex (codex-rescue)** | 獨立第二意見，pre-implementation 評估 pattern | 只在 PR #155 pattern demo 和 PR #156 實作前用；shared runtime 有時回空（workaround：改用 general-purpose agent） |
| **General-purpose agent（Claude）** | 可靠 fallback、深入 10 個 checkpoint | Codex 不可用時替代 |

---

## §6 Carried Debt & Known Gaps

這是需要「記得回來處理」的清單。每個 item 對應一個 TODO.md 項目或新追蹤項。

### 6.1 R26-6 — HMAC chain 版本化 + 儲存後雜湊（**最重要**）

**Issue**：PR #154 將 HMAC 編碼從字串串接改為 length-prefix canonical encoding。既有 rows 用舊格式，驗證時需區分；目前 PR #158 的驗證 cron 會對舊格式 row 產生 false positive。

**兩個合併議題**：
1. **版本化**：加 `user_activity_logs.hmac_version SMALLINT`（`1`=string-concat legacy / `2`=length-prefix canonical），寫入時標記版本，驗證依版本分流
2. **儲存後雜湊**：目前 `HmacInput` 在 INSERT 前建構；若呼叫者沒提供 changed_fields，stored proc 的 UNION fallback 會產生不同於 HmacInput 的 changed_fields → HMAC 漏算最終存入值。修法：`log_activity` 返回 id + final_changed_fields，HMAC 用持久化後的值計算

**Source**：PR #154 Gemini SECURITY-MEDIUM + CodeRabbit Major；PR #158 CodeRabbit 再度點名「legacy HMAC writer 產生 false positives」。

**Blocks**：R26-4（舊 `log_activity` 最終移除）、PR #158 斷鏈告警的準確度。

**Estimate**：~1 人日（migration + stored proc + verify_chain_range 調整 + 測試）。

### 6.2 R26-4 — 舊 `log_activity(&pool, ...)` 最終移除

**Blocked by**：97 個 call sites 全部遷完（53 個已落地或 open；~44 個剩餘）+ R26-6。

**Estimate**：~0.5 人日（delete 舊 API + 移除 CI `-A deprecated` + 加 grep guard）。

### 6.3 R26-9（新追蹤）— Entity redact allowlist review

**Trigger**：CodeRabbit PR #156 對 `CareRecord` / `AnimalVetAdvice` / `VetAdviceRecord` 的 🟠 Major feedback。

**Scope**：掃過所有 R26 已實作的 entity（`Animal` / `AnimalObservation` / `AnimalSurgery` / `AnimalWeight` / `AnimalSacrifice` / `AnimalSuddenDeath` / `AnimalTransfer` / `AnimalPathologyReport` / `AnimalBloodTest` / `AnimalBloodTestItem` / `VetRecommendation` / `CareRecord` / `VetAdviceRecord` / `AnimalVetAdvice` / `Document` / `DocumentLine` / `PoReceiptStatus` / `LeaveRequest` / `OvertimeRecord` / `AttendanceRecord` / etc.），決策：空 impl / 完整 allowlist / summary log。

**Estimate**：~1 人日。

### 6.4 R26-10（新追蹤）— Upsert pattern 併發安全化

**Issue**：D-05 的 `SELECT FOR UPDATE → match → INSERT/UPDATE` 模式在 row 不存在時不鎖父層，兩個併發 request 都可走到 None 分支 → 都 INSERT → UNIQUE 衝突。

**Fix options**：
- 鎖父層 animal row（需 animal_id available）
- 改 `INSERT ... ON CONFLICT DO UPDATE RETURNING *, xmax = 0 AS was_inserted`（原子 upsert + 得知是 insert 還是 update）

**Scope**：PR #156 `AnimalVetAdviceService::create_or_update` 目前採此模式；其他類似 service 若有 upsert 也該統一。

**Estimate**：~0.5 人日（pattern 一次定型、多處套用）。

### 6.5 R26-11（新追蹤）— Service-layer 授權補強

**Issue**：CodeRabbit PR #156 `vet_advice.rs:330` 指出 `delete` 服務只確認登入使用者、未確認該使用者可刪該動物/專案的 record。目前依靠 handler 層 `access::require_animal_access`，但 service 層可能被其他 service 呼叫而繞過。

**Scope**：所有 R26 遷移的 service mutation，特別是 `delete` / `soft_delete_with_reason` / `change_status` 類。

**Estimate**：~1 人日（設計決策：在 service 層接 `&Access` object vs handler 負責 → 影響 pattern 本身）。

### 6.6 R26-12（新追蹤）— Document lines audit completeness

**Issue**：Gemini PR #159 `crud.rs:197` 指出 `DOCUMENT_CREATE` / `DOCUMENT_UPDATE` 的 audit 只 capture Document header，漏掉 `document_lines` 子表變動。

**Fix**：擴充 `Document` struct（或 `DocumentWithLines` wrapper）讓 `DataDiff::compute` 能含 lines；或在 service 層額外記 `DOCUMENT_LINE_CHANGE` event。

**Estimate**：~0.5 人日。

### 6.7 R26-1 — 長 Scheduler job 升級 `tokio::select!`

**Status**：原審查 WARN-04 延伸。目前 14 cron 只接 `is_cancelled()` 開頭檢查；對 monthly_report（20-120s）/ db_analyze（30-300s）/ calendar_sync（5-60s）3 個長 job 升級。

**Estimate**：~1 人日（含 shutdown grace period + 長 job 安全中斷點設計）。

### 6.8 R26-7 — Dead code 11 處 cleanup

**Source**：PR #155 A3 移除 crate-level allow 後暴露 11 處 per-item allow；每處需個別判斷（真死 / API DTO 待 openapi / serde 被動欄位）。

**Estimate**：~0.5 人日。

### 6.9 R26-8 — ProtocolService::change_status 完整 Service-driven

**Why deferred**：change_status 涉及 `PartnerService::create` 跨 service（需 `PartnerService::create_tx`）、4 個內部 helper fn、10+ DB 操作；PR #155 只做 submit 作為 pattern demo，change_status 需 1-2 人日。

**Unblocks**：IACUC numbering race 的剩餘 20%、Gemini PR #155 對 `numbering.rs:207` 的 prefix 邏輯統一。

### 6.10 審查報告其他未處理項（優先級低）

- **WARN-01**：Service 層直接寫 SQL（909 處 / 80 檔），ongoing policy（新 code 寫 repositories/，舊 code 不強求）
- **WARN-02**（延伸）：calamine / rust_xlsxwriter / google_calendar credentials 未包 spawn_blocking，Pending 到出現效能問題再處理
- **WARN-03**：clippy.toml 閾值寬鬆於 CLAUDE.md 規範，Pending
- **WARN-05**：cache stampede（alert_threshold / security_notifier / ip_blocklist），Pending
- **WARN-07**：equipment.rs 75KB / scheduler.rs 40KB 拆分，Pending
- **WARN-08**：權限字串硬編碼，Pending

---

## §7 Current State（截至 2026-04-22）

### 7.1 Numbers

| 指標 | 值 |
|------|----|
| 已 merge PR | 3（#153 / #154 / #155） |
| Open PR | 8（#156 / #157 / #158 / #159 / #160 / #161 / #162 / #163） |
| 總 commits on `integration/r26` | ~27（合入後 + open PR 約 44+） |
| Mutations 遷移完成或 open | **53 / 97**（54.6%） |
| Deprecated warnings 進度 | 91（PR #153）→ 預計合入所有 open PR 後 ~50 |
| R26 TODO 項目 | 7 個未完成 + 1 個已完成（R26-5） |
| Review comments 總計 | 70（2🔴 / 23🟠 / 43🟡 / 1🔵 / 1 note） |
| CI 狀況 | 9 PR all green；#162/#163 CI 進行中 |

### 7.2 Mutation 遷移進度

| 模組 | 總 call sites | 已落地 | 對應 PR | 剩餘 |
|------|--------------|--------|---------|------|
| protocol | - | submit 1 | #155 | change_status（R26-8）|
| animals | 49 | 17 (simple) | #156 | 32 (moderate/complex：observation/surgery update + record_versions / blood_test / transfer state machine / import_export / field_correction / vet_patrol / sudden_death) → PR #4b~4e |
| document | 10 | 10 | #159 | 0 |
| hr/leave | 7 | 7 | #160 | 0 |
| hr/overtime+balance+attendance | 14 | 14 | #161 | 0 |
| user | 4 (of ~8) | 4 | #162 | ~4 (reset_password / impersonate → #6d 需 AuthService) |
| 其他 handlers | 48 (user 已數) | 7 (user via #162) | - | ~44 (product 7 / sku 5 / partner 4 / warehouse 4 / equipment 4 / role 3 / ai 3 / auth 3 / hr 3 / two_factor 2 / data_export 2 / audit 1) → PR #6b / #6c / #6d |
| **Total** | **97** | **53** | - | **~44** |

### 7.3 CI / Review 就緒度（merge 前檢查）

| PR | CI | CodeRabbit | Gemini | 待處理 blocker | 可 merge？ |
|----|----|-----------|--------|----------------|-----------|
| #156 | ✅ Clean | 6 🟠 Major | 6 ![medium] | D-09 / D-10 跨 PR 處理；R26-9/-10/-11 追蹤；PR 自身不 block | ⚠️ 可 merge 但標記延後事項 |
| #157 | ✅ Clean | - | 3 ![medium] | 純 docs，medium 為措辭；不 block | ✅ |
| #158 | ✅ Clean | **2 🔴 Critical + 6 🟠** | 1 ![high] + 3 ![medium] | 🔴 verifier/writer 分歧 **必須本 PR 修** | ❌ 需追加 commit |
| #159 | ✅ Clean | - | 4 ![medium] | R26-12 追蹤；本 PR 不 block | ✅ |
| #160 | ✅ Clean | - | 1 ![high] + 1 ![medium] | ![high] validation 缺失，建議本 PR 追加修正 | ⚠️ 建議修後 merge |
| #161 | ✅ Clean | - | 2 ![medium] | 一致性建議，不 block | ✅ |
| #162 | ⏳ CI running | - | 4 ![medium] | D-03/D-09 統一策略建議，不 block | ⏳ 等 CI |
| #163 | ⏳ CI running | - | 2 ![medium] | 措辭不一致，本 PR 追加修正 | ⏳ 等 CI |

---

## §8 Forward Path（合流順序與里程碑）

### 8.1 建議 merge order（2026-04-22 快照）

1. **Block PR #158 直到 2 🔴 Critical 修完**
2. Merge 獨立模組 PR（無互相依賴）：**#156 → #159 → #160 → #161 → #162**
3. Merge 文件 PR（等 1 所有 code PR merged 後解 §9 conflict）：**#157 → #163**（#163 rebase 到 #157 之下）
4. Merge 修完的 PR #158
5. 啟動 **PR #4b**（animals moderate：observation/surgery update with record_versions）
6. 啟動 **PR #4c**（animals transfer state machine）
7. 啟動 **PR #4d**（animals import_export 批次）
8. 啟動 **PR #4e**（animals complex 殘留：field_correction / vet_patrol / sudden_death）
9. 啟動 **R26-6**（HMAC 版本化 + 儲存後雜湊）— PR #158 merged 後愈早做愈好
10. 啟動 **PR #6b**（product + sku 12 sites）
11. 啟動 **PR #6c**（partner + warehouse + equipment 12 sites）
12. 啟動 **PR #6d**（role + ai + auth + two_factor 11 sites，含 reset_password / impersonate）
13. 啟動 **R26-9 / R26-10 / R26-11 / R26-12**（entity allowlist / upsert 安全 / service 授權 / document lines）
14. 啟動 **R26-8**（change_status 完整 Service-driven）
15. 啟動 **R26-1**（scheduler 長 job select!）
16. 啟動 **R26-7**（dead code 11 處）
17. 啟動 **R26-4**（舊 `log_activity` 移除 + CI guard）
18. **R26 epic 完成** → 發起 **PR `integration/r26` → main** 單一 merge PR

### 8.2 里程碑

| 里程碑 | 條件 | 預計工時 |
|--------|------|---------|
| **M1：INFRA + Pattern 定型**（已達成 2026-04-21） | PR #153/154/155 merged | ✅ Done |
| **M2：核心模組遷移完成**（單 PR 批次） | PR #156+#159+#160+#161+#162 merged（+ 2 docs PR） | 剩餘 1-2 天 review+merge |
| **M3：R26-3 全面完成** | Deprecated warnings = 0，R26-6 可進行 | +~2 週（PR #4b-e + #6b-d） |
| **M4：HMAC 安全底線強化** | R26-6 merged；驗證 cron 無 false positives | +~3 天 |
| **M5：Service-driven 語意統一** | R26-9/-10/-11/-12 處理；change_status 重構（R26-8）；scheduler 長 job（R26-1）；dead code 清理（R26-7） | +~1 週 |
| **M6：epic 結束** | 審查報告 P0 全綠、P1 80%+；`integration/r26 → main` | Target 2026-05-15 |

### 8.3 「我需要先決策什麼」checkpoint

在啟動下一個階段前需要使用者決策：

1. **PR #158 的 2 🔴 怎麼修**：要等我追加 commit 還是先 block merge 等 R26-6 一起？（建議：本 PR 修 Critical + Major；R26-6 另案）
2. **PR #160 Gemini 🟠 high 的 validation**：要在 PR #160 追加還是另 PR？（建議：本 PR 追加，因為是 HR leave 合規性）
3. **D-09 / D-10 統一化時機**：在下個 PR 一起做（delay all merges）還是先讓 PR 進 integration/r26 再另案統一？（建議：後者）
4. **R26-9 ~ R26-12 四個新追蹤項的執行順序**：依審查報告優先級排（R26-10 > R26-11 > R26-9 > R26-12）還是依發現時序？

---

## §9 Document Maintenance

### 9.1 何時更新這份文件

- **新 PR 開出**：§4 PR Catalog 新增一列 + §7 Numbers 更新
- **PR merged**：§4 狀態欄改 ✅；§7 mutation 進度更新；§5.2 Review Response Matrix 把延後項改「✅ merged in PR #xxx」
- **新決策**：§3 Decision Log 底部追加（D-NN 編號，絕不重用；絕不刪除舊條目）
- **Review 指出新 issue**：§5.3 追加一列 + §6 若需追蹤則加 R26-N 新項
- **發現新 blocker 或 scope 變動**：§2.3 / §8.2 里程碑更新

### 9.2 格式約定

- 決策編號：`D-01 ~ D-NN`（epic-wide，連續遞增）
- TODO 編號：對應 TODO.md 的 R26-N（與 TODO.md 同步）
- PR 連結：全部使用完整 GitHub URL `https://github.com/delightening/ipig_system/pull/NNN`
- 日期：全部 GMT+8（台北時區），格式 `YYYY-MM-DD`
- 時間估計：人時（person-hour），粗略到 0.5 day 粒度

### 9.3 這份文件不該做什麼

- **不放程式碼**：code review suggestions 請留在 PR comments，本文件只引用結論
- **不記錄過程性細節**（例如「cargo test 跑了 3 分鐘」）：放 PROGRESS.md §9 即可
- **不重複 PROGRESS.md / TODO.md 的內容**：§4 / §6 分別是 PR Catalog 和 TODO 的延伸**解讀**，不是複製
- **不預測未來的 review**：§5 只記已發生的 review；預測性的風險分析放 §6

---

**版本**：v1.0（2026-04-22 初版）
**維護者**：單人（Jason Wang）+ AI 協作（Claude Code + Gemini + CodeRabbit + Codex）
