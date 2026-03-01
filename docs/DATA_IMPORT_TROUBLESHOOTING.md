# 全庫 IDXF 匯入故障排除

## 匯入規則

**遇重複則取代**：依主鍵或唯一鍵（code/email 等）判斷，若已存在則以匯入內容覆蓋該筆資料（ON CONFLICT DO UPDATE）。

## 錯誤詳情檢視

匯入完成後若有錯誤，系統會顯示「匯入錯誤詳情」Dialog，列出每個失敗資料表的錯誤訊息。請將完整錯誤內容提供給開發人員以協助修正。

## 常見錯誤與對應處理

| 錯誤類型 | 可能原因 | 建議做法 |
|----------|----------|----------|
| `null value in column "X" violates not-null constraint` | 匯出檔缺少目標表必填欄位 | 確認匯出/匯入端 schema 版本一致；或於 `schema_mapping.rs` 新增欄位預設值轉換 |
| `invalid input syntax for type uuid` | JSON 中 UUID 格式不正確 | 檢查匯出檔該欄位是否為有效 UUID 字串 |
| `foreign key constraint` | 參照的父表資料不存在 | 確認匯入順序（應依 EXPORT_TABLE_ORDER）；或先匯入父表 |
| `duplicate key value violates unique constraint` | 主鍵衝突 | 使用 Append 模式時應自動略過（ON CONFLICT DO NOTHING）；若仍報錯，可能為非 PK 的 unique 約束 |
| `略過未知表: X` | 匯出檔含未在 EXPORT_TABLE_ORDER 的表 | 若為新表，可於 `data_export.rs` 的 EXPORT_TABLE_ORDER 新增 |
| `relation "X" does not exist` | 目標 DB 缺少該表 | 確認目標 DB 已執行完整 migrations |

## Schema 版本對應

- 匯出檔 `meta.schema_version` 與目標 DB 的 `_sqlx_migrations` 版本應一致。
- 若版本不同，`schema_mapping::transform_row` 會嘗試轉換欄位名；可於 `schema_mapping.rs` 擴充對應規則。

## 匯入至全新 DB

若目標為全新安裝，建議：

1. 先執行 `sqlx migrate run` 建立完整 schema
2. 再執行 IDXF 匯入

## 匯入至既有 DB（含資料）

- Append 模式：已存在的主鍵會略過，僅新增新資料
- 若有 FK 參照衝突（例如匯入的 user_id 在目標不存在），該表會整批失敗
