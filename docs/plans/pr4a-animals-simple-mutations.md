# PR #4a — Animals Simple Mutations + AuditRedact 基礎

- **基於**：`integration/r26` HEAD + PR #155 merged
- **對應 R26 Section**：R26-3 的 animals 子集起點 / PR #4 系列第一個
- **產出**：
  1. 12 個 `AuditRedact` empty impl（aka stub）落在各 animal model 檔
  2. 18 個 simple（single-table）mutation 改寫為 Service-driven
  3. 對應 handler 的 audit 呼叫從 handler 層刪除
- **工時估計**：~40 person-hours（12 stubs ~1h + 18 × ~2h + 測試調整 ~3h）
- **測試標準**（CLAUDE.md R26 執行紀律）：**handler 類必須** `cargo test --all-targets` 全綠
- **Commit 粒度**：按檔拆 commit（源自 CLAUDE.md #3 放寬原則），估 6-8 個 commit

---

## Step 0 — 前置

- 確認 base：`git fetch origin && git checkout -b feat/animals-simple-mutations origin/integration/r26`
- 確認 PR #155 已 merged：檢查 `git log --oneline -1 origin/integration/r26` 含 Service-driven submit pattern
- CI：無需調整（`integration/**` trigger 已於 PR #154 落地）

---

## Step 1 — AuditRedact empty impls（12 entities）

在對應 model 檔案（多半為 `backend/src/models/animal/entities.rs` 或其 re-export）尾端加入：

```rust
impl crate::models::audit_diff::AuditRedact for Animal {}
impl crate::models::audit_diff::AuditRedact for AnimalWeight {}
impl crate::models::audit_diff::AuditRedact for AnimalBloodTest {}
impl crate::models::audit_diff::AuditRedact for AnimalBloodTestItem {}
impl crate::models::audit_diff::AuditRedact for AnimalObservation {}
impl crate::models::audit_diff::AuditRedact for AnimalSurgery {}
impl crate::models::audit_diff::AuditRedact for AnimalVaccination {}
impl crate::models::audit_diff::AuditRedact for AnimalSacrifice {}
impl crate::models::audit_diff::AuditRedact for AnimalSuddenDeath {}
impl crate::models::audit_diff::AuditRedact for AnimalTransfer {}
impl crate::models::audit_diff::AuditRedact for AnimalPathologyReport {}
impl crate::models::audit_diff::AuditRedact for VetRecommendation {}
```

所有 animal 實體目前皆無敏感欄位，empty impl 足夠。未來若新增加密欄位或 session token，再覆寫 `fn redact_paths() -> Vec<&'static str>`。

**Commit 1**：`feat(models): AuditRedact empty impls for 12 animal entities`

---

## Step 2 — 18 Simple mutations 改寫（按檔批次）

**改寫模板**（所有 simple mutation 共用）：

```rust
// Before
pub async fn mutate(pool: &PgPool, /* args */, user_id: Uuid) -> Result<Entity> {
    let row: Entity = sqlx::query_as("INSERT INTO ... RETURNING *")
        .bind(...)
        .fetch_one(pool)
        .await?;
    Ok(row)
}

// After
pub async fn mutate(pool: &PgPool, actor: &ActorContext, /* args */) -> Result<Entity> {
    let _user = actor.require_user()?;
    let mut tx = pool.begin().await?;

    // INSERT：before=None，after=新建 row
    let after: Entity = sqlx::query_as("INSERT INTO ... RETURNING *")
        .bind(...)
        .fetch_one(&mut *tx)
        .await?;
    let diff = DataDiff::compute::<Entity>(None, Some(&after));
    AuditService::log_activity_tx(&mut tx, actor, ActivityLogEntry {
        module: "ANIMAL",
        action: "CREATE",
        target_type: Some("animal_weight"),  // per entity
        target_id: Some(after.id),
        target_name: None,
        before_data: diff.before,
        after_data: diff.after,
        changed_fields: diff.changed_fields,
        ..Default::default()
    }).await?;

    tx.commit().await?;
    Ok(after)
}
```

**UPDATE 版本**：先 `SELECT FOR UPDATE` before → 執行 UPDATE RETURNING * → 取 after → `DataDiff::compute(Some(&before), Some(&after))`。

### Commit 2 — weight.rs（2 simple）

| 函數 | Lines | SQL 結構 | action 字串 |
|---|---|---|---|
| `create` | 55-76 | 1 INSERT | `"CREATE"` |
| `update` | 79-100 | SELECT FOR UPDATE + 1 UPDATE | `"UPDATE"` |

Handler：[backend/src/handlers/animal/weight_vaccination.rs](backend/src/handlers/animal/weight_vaccination.rs) 第 62、96 行 `log_activity` 呼叫移除。

### Commit 3 — source.rs（3 simple）

| 函數 | Lines | SQL 結構 | action 字串 |
|---|---|---|---|
| `create_source` | 28-50 | 1 INSERT | `"CREATE"` |
| `update_source` | 53-85 | SELECT FOR UPDATE + 1 UPDATE | `"UPDATE"` |
| `delete_source` | 88-97 | 1 UPDATE (is_active=false) | `"DELETE"` |

Handler 側：animal core handler（具體檔案需確認），對應的 audit 呼叫刪除。

### Commit 4 — vet_advice.rs（3 simple）

| 函數 | Lines | SQL 結構 | action 字串 |
|---|---|---|---|
| `VetAdviceService::upsert` | 46-71 | 1 INSERT ON CONFLICT | `"UPSERT"`（或分流 CREATE/UPDATE） |
| `VetAdviceRecordService::create` | 120-141 | 1 INSERT | `"CREATE"` |
| `VetAdviceRecordService::update` | 143-168 | SELECT FOR UPDATE + 1 UPDATE | `"UPDATE"` |

Handler：[backend/src/handlers/animal/vet_recommendation.rs](backend/src/handlers/animal/vet_recommendation.rs) 第 65、121、177、233 行 4 個 log_activity 點。

**注意 upsert**：因 `ON CONFLICT DO UPDATE` 的 before 難以精準取得（INSERT 前無 row 可能、UPDATE 前 row 存在），建議拆為 `create_or_update` 先 SELECT → 分流 INSERT 或 UPDATE 路徑，這樣 before/after 都清楚。或保留 `upsert` 但 action 用 `"UPSERT"` + `before_data = None`（審計員可從前後 hash 鏈推斷）。此項需與使用者確認選哪個。

### Commit 5 — care_record.rs（3 simple）

| 函數 | Lines | SQL 結構 | action 字串 |
|---|---|---|---|
| `create` | 173-206 | 1 INSERT | `"CREATE"` |
| `update` | 209-253 | SELECT FOR UPDATE + 1 UPDATE | `"UPDATE"` |
| `soft_delete_with_reason` | 256-282 | 1 UPDATE | `"DELETE"` |

Handler：[backend/src/handlers/animal/care_record.rs:112](backend/src/handlers/animal/care_record.rs:112) 的 log_activity 移除。

### Commit 6 — medical.rs vaccination 子集（2 simple + 1 borderline）

| 函數 | Lines | SQL 結構 | 分類 |
|---|---|---|---|
| `create_vaccination` | 42-64 | 1 INSERT | Simple |
| `update_vaccination` | 67-90 | SELECT FOR UPDATE + 1 UPDATE | Simple |
| `soft_delete_vaccination_with_reason` | 103-137 | 1 INSERT change_reasons + 1 UPDATE vaccination | Moderate（2 tables） |

**注意**：soft delete 版本同時寫 `change_reasons` + `vaccinations`，理論上屬 moderate。為讓 PR #4a scope 一致，兩個選項：
- (A) 只納入 2 個 simple，soft_delete 留給 PR #4b
- (B) 包進來並在 commit 說明「change_reasons 也視為 audit trail 的一部分，一起放 tx 內」

建議 (B)，因 change_reasons 本就是 audit trail，視為 mutation 伴生。

Handler：[backend/src/handlers/animal/weight_vaccination.rs](backend/src/handlers/animal/weight_vaccination.rs) 第 132、200、237、278 行的 4 個 vaccination audit 點。

### Commit 7 — observation.rs + surgery.rs 的 `create` 子集（2 simple）

| 函數 | Lines | SQL 結構 | action 字串 |
|---|---|---|---|
| `observation::create` | 81-132 | 1 INSERT（含 JSONB validation） | `"CREATE"` |
| `surgery::create` | 104-152 | 1 INSERT | `"CREATE"` |

**排除 `update`**：兩者 update 皆涉及 `record_versions` 巢狀表寫入（moderate），留給 PR #4b。

Handler：
- [backend/src/handlers/animal/observation.rs:176](backend/src/handlers/animal/observation.rs:176) create audit
- [backend/src/handlers/animal/surgery.rs:84](backend/src/handlers/animal/surgery.rs:84) create audit

### Commit 8 — weight.rs::soft_delete + care_record 已含 + 補回

單獨 commit：`weight.rs::soft_delete_with_reason` (113-147) 同時寫 `change_reasons` + UPDATE weight row（同 moderate 但與 vaccination soft_delete 並行）。

Handler：對應 animal_core.rs 或 weight_vaccination.rs 的 delete 路徑。

---

## Step 3 — 測試

- 跑 `cargo test --all-targets`（需本地啟動 Postgres test DB）
- 重點檢查整合測試：`backend/tests/animal_*` 若存在
- 驗證：
  1. 每個改寫後的 mutation 產生一筆 `user_activity_logs` row，含正確 `before_data` / `after_data` / `changed_fields`
  2. 每個 row 的 `integrity_hash` 非空且成功接鏈
  3. Handler 刪除 audit 呼叫後，response 時間不變（若有效能測試）

---

## Step 4 — Clippy + 格式

```bash
cargo clippy --all-targets -- -D warnings -A deprecated
cargo fmt --check
```

- 新增的 simple mutation 不應產生新 deprecated warning（因已改用 `log_activity_tx`）
- 若減少了 deprecated 使用點（17 個 handler audit 呼叫刪除），`cargo build 2>&1 | grep 'deprecated' | wc -l` 應從 97 減至 ~80

---

## Step 5 — PR 描述模板

- 標題：`feat(animals): Service-driven audit for 18 simple mutations + AuditRedact stubs`
- 描述要點：
  - 12 個 AuditRedact empty impl 落地
  - 18 個 single-table mutation 遷移至 Service-driven pattern（複製 PR #155 `Protocol::submit` 模板）
  - 17 個 handler 層 audit 呼叫移除（含 weight_vaccination/vet_recommendation/care_record/observation/surgery/animal_core 子集）
  - 測試：`cargo test --all-targets` 全綠
  - 不含 moderate/complex（update with record_versions、transfer、blood_test、import_export）→ 這些歸 PR #4b / #4c / #4d / #4e
- base 分支：`integration/r26`
- 預期 CI：clippy + test + CodeRabbit（手動觸發 `@coderabbitai review`）

---

## 風險與決策點

1. **upsert 的 audit 語意**（見 Commit 4）：需使用者決定拆 create_or_update 或保留 upsert + `before_data = None`
2. **soft_delete 視為 simple 還是 moderate**：建議維持放在 PR #4a，因 change_reasons 是 audit trail 的一部分，不算跨業務域
3. **Commit 粒度 8 個 vs CLAUDE.md 放寬後的上限 15**：安全範圍內
4. **actor 參數遷移**：原有的 `user_id: Uuid` 參數統一改為 `actor: &ActorContext`，呼叫端（handler）需改 `&actor` 傳入；extractor 會把 `CurrentUser` 包成 `ActorContext::User`（參照 PR #155 的 handler 變更）

---

## 後續 PR

- **PR #4b**：observation / surgery 的 update（含 record_versions 巢狀表）、blood_test 簡單 CRUD —— Moderate 子集
- **PR #4c**：transfer.rs 狀態機（initiate/vet_evaluate/approve/complete/reject）
- **PR #4d**：import_export.rs 批次匯入（需設計 batch audit 粒度）
- **PR #4e**：field_correction.rs + vet_patrol.rs 殘留 complex + medical.rs::create_sudden_death

全 animal 模組消化 = 49 個 handler audit 呼叫全清，同步解鎖 R26-4（舊 `log_activity` 移除）的一部分。
