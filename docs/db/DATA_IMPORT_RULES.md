# 全庫 IDXF 匯入規則

## 重複處理

**遇重複則取代**：匯入時若主鍵或唯一鍵（code、email 等）已存在，以匯入內容覆蓋該筆資料。

- 實作：`ON CONFLICT (columns) DO UPDATE SET col = EXCLUDED.col`
- 衝突欄位：依表而定，見 `data_import.rs` 的 `get_conflict_columns()`

## 衝突欄位對應

| 表 | 衝突欄位 |
|---|----------|
| roles, permissions, animal_sources, blood_test_templates, product_categories, sku_* | code |
| blood_test_panels | key |
| users | email |
| notification_routing | (event_type, role_code) |
| 其餘 | 主鍵 (id) |
