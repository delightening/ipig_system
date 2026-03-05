# OpenAPI 端點文件化與 Rust 單元測試 — 盤點與進度

> **最後更新：** 2026-03-05  
> **目的：** 供「全部端點文件化」與「單元測試補齊」追蹤用。

---

## 1. 端點文件化盤點

### 1.1 總量

| 項目 | 數量 |
|------|------|
| **路由中唯一 handler 總數** | **318** |
| **已列入 OpenAPI paths()** | **132**（含 storage_location 11 + sku 6） |
| **尚未文件化** | **約 186** |

### 1.2 已文件化模組（依 openapi.rs）

- **監控**：health_check, metrics_handler, vitals_handler（3）
- **認證**：login, refresh_token, logout, me, update_me, change_own_password, confirm_password, forgot_password, reset_password_with_token, heartbeat, stop_impersonate, export_me, delete_me_account（13）
- **2FA**：setup_2fa, confirm_2fa_setup, disable_2fa, verify_2fa_login（4）
- **使用者偏好**：get_all_preferences, get_preference, upsert_preference, delete_preference（4）
- **使用者**：create_user, list_users, get_user, update_user, delete_user, reset_user_password, impersonate_user（7）
- **角色**：create_role, list_roles, get_role, update_role, delete_role, list_permissions（6）
- **設施**：species / facilities / buildings / zones / pens / departments 全 CRUD（30）
- **倉儲**：warehouse CRUD（5）+ **storage_location 全模組（11）** + **SKU（6）**：get_sku_categories, get_sku_subcategories, generate_sku, validate_sku, preview_sku, create_product_with_sku
- **計畫書**：protocol crud + export + review（24）
- **電子簽章**：sign_* / get_*_signature_status / annotations（11）
- **動物**：list_animals, list_animals_by_pen, get/create/update/delete_animal, batch_assign_animals, mark_animal_vet_read, get_animal_events（9）

### 1.3 尚未文件化模組（依 routes 與 handler 對照）

| 模組 | 端點數 | 說明 |
|------|--------|------|
| warehouse 延伸 | 2 | import_warehouses, download_warehouse_import_template |
| product | 11 | list/create/get/update/delete, import, check_duplicates, template, create_product_with_sku, categories |
| sku | 5 | get_sku_categories, get_sku_subcategories, generate_sku, validate_sku, preview_sku |
| partner | 8 | list/create/get/update/delete, import, template, generate_code |
| document | 9 | list/create/get/update/delete, submit, approve, cancel |
| stock | 3 | get_inventory_on_hand, get_stock_ledger, get_low_stock_alerts |
| audit | 12 | list_audit_logs, export, activity_logs, login_events, sessions, security_alerts, dashboard, force_logout, resolve_alert 等 |
| report | 7 | stock_on_hand, stock_ledger, purchase_lines, sales_lines, cost_summary, blood_test_cost, blood_test_analysis |
| accounting | 7 | chart_of_accounts, trial_balance, journal_entries, ap_aging, ar_aging, create_ap_payment, create_ar_receipt |
| animal 延伸 | 約 80+ | 動物來源、欄位修正、觀察/手術/體重/疫苗/犧牲/猝死/轉讓/病理/照護/血液檢查/模板/面板/獸醫建議/匯入匯出 等 |
| notification | 19 | 通知、未讀數、已讀、設定、排程報表、報告歷史、下載、庫存/效期警報、手動觸發 等 |
| upload | 12 | 附件列表/下載/刪除、協定/動物/病理/犧牲/獸醫建議/觀察/請假上傳、動物匯入 等 |
| system_settings | 2 | get/update_system_settings |
| config_check | 1 | get_config_warnings |
| notification_routing | 6 | list/create/update/delete, event_types, roles |
| treatment_drug | 6 | list, admin_list, create, update, delete, import_erp |
| data_export | 2 | full_database_export, full_database_import |
| sse | 1 | sse_security_alerts |
| hr | 33 | 打卡、加班、請假、餘額、儀表板、日曆、人員 等 |
| calendar | 12 | status, config, connect, disconnect, sync, history, pending, conflicts, resolve, events |
| training | 5 | list/create/get/update/delete_training_record |
| equipment | 10 | equipment + calibrations CRUD |
| qau | 1 | get_qau_dashboard |
| euthanasia | 7 | create_order, get_pending, get_order, approve, appeal, execute, decide_appeal |
| amendment | 13 | list, get_pending_count, get, update, submit, classify, start_review, decision, status, versions, history, assignments, list_protocol_amendments |

**合計未文件化約 192 個端點**（以上為依模組粗估，實際以 routes 為準）。

### 1.4 單端點文件化步驟（供批次補齊）

1. 在對應 **model** 為 request/response 型別加上 `#[derive(ToSchema)]`（必要時加 `#[schema(value_type = ...)]`，如 Decimal）。
2. 在 **handler** 函式上加上 `#[utoipa::path(get|post|put|patch|delete, path = "...", params(...), request_body = ..., responses(...), tag = "...", security(("bearer" = [])))]`。
3. 在 **openapi.rs** 的 `paths(...)` 中註冊該 handler。
4. 在 **openapi.rs** 的 `components(schemas(...))` 中註冊新增的 schema（若尚未存在）。

---

## 2. Rust 單元測試盤點

### 2.1 目前狀態

| 項目 | 數量 |
|------|------|
| **現有單元測試數（cargo test --lib）** | **148** |
| **已有測試的模組** | constants, config, middleware (csrf, etag, real_ip), models (user, animal, facility, hr, protocol, mod), services (auth, file, sku, warehouse, partition_maintenance) |

### 2.2 可補強測試的模組（建議）

- **models**：document, stock, notification, amendment, euthanasia（列舉/驗證/邊界）。
- **services**：document, stock, partner, data_import（解析/驗證邏輯）。
- **handlers**：僅在具純邏輯時才適合單元測試；多數為整合測試覆蓋。

### 2.3 單測撰寫建議

- 優先為 **純函式 / 解析 / 驗證 / 常數** 撰寫單元測試。
- 每個模組先補 2～5 個代表性 case，再視需要擴充。

---

## 3. 本次已完成（2026-03-05）

- **OpenAPI**  
  - storage_location 全模組 **11 個端點**（create/list/get/update/delete_storage_location, update_warehouse_layout, generate_storage_location_code, get/create/update/transfer inventory）。  
  - SKU **6 個端點**：get_sku_categories, get_sku_subcategories, generate_sku, validate_sku, preview_sku, create_product_with_sku。  
- **Schemas**：StorageLocation 相關型別、SKU 相關型別（CategoriesResponse, GenerateSkuRequest/Response, ValidateSkuRequest/Response, SkuPreviewRequest/Response, CreateProductWithSkuRequest）、ProductWithUom, Product, ProductUomConversion 已加入 ToSchema 並註冊於 openapi.rs。

---

## 4. 後續建議

- **端點**：依業務優先級，依序補齊 product、sku、partner、document、stock、report、accounting，再處理 animal 延伸、notification、upload、hr、calendar、training、equipment、euthanasia、amendment 等。
- **測試**：每完成一輪端點文件化，可順便為該模組 request/response 或相關 service 補 2～5 個單元測試，維持測試數與覆蓋率成長。
