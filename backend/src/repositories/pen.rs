use sqlx::PgConnection;
use uuid::Uuid;

use crate::models::Pen;
use crate::Result;

pub struct PenRepository;

/// 欄位代碼的正規化 SQL 運算式：大寫、去 `-`、去前後空白；若是「區字母 A–G + 數字」，
/// 數字去掉前導零後補滿兩位（`A1`／`a-01`／`A001` 都是 `A01`）。
///
/// 規則必須與 `AnimalUtils::canonical_pen_code`（Rust 端，用在輸入）逐字一致：兩邊用同一套
/// 規則比對，存成 `A1` 的欄位才不會對不到自己、也不會誤選到另一區存成 `A01` 的欄位。
/// `column` 只能是呼叫端寫死的欄名或參數佔位（如 `code`、`$1::text`），不可放使用者輸入。
pub fn canonical_code_expr(column: &str) -> String {
    let c = format!("upper(replace(btrim({column}), '-', ''))");
    // 數字去前導零後的部分（全零時留一個 '0'）。補零不用 `lpad(…, 2, …)`：
    // lpad 在字串比目標長時會「截斷」，`A100` 會變成 `A10`，與 Rust 端不一致；
    // 這裡只在不足兩位時才補。
    let n = format!("COALESCE(NULLIF(ltrim(substr({c}, 2), '0'), ''), '0')");
    format!(
        "CASE WHEN {c} ~ '^[A-G][0-9]+$' \
         THEN left({c}, 1) || repeat('0', greatest(2 - length({n}), 0)) || {n} \
         ELSE {c} END"
    )
}

impl PenRepository {
    /// 依 ID 查詢欄位
    pub async fn find_pen_by_id(conn: &mut PgConnection, id: Uuid) -> Result<Option<Pen>> {
        let pen = sqlx::query_as::<_, Pen>("SELECT * FROM pens WHERE id = $1")
            .bind(id)
            .fetch_optional(conn)
            .await?;
        Ok(pen)
    }

    /// 依「正規化後」代碼查詢啟用中的欄位。
    ///
    /// 呼叫端須先用 `AnimalUtils::canonical_pen_code` 把輸入正規化；資料庫端用
    /// [`canonical_code_expr`] 做同一套正規化（含補零）。欄位代碼只在「同一區、啟用中」
    /// 唯一，不同區理論上可有相同代碼；最多回 2 筆，足以讓呼叫端判斷
    /// 「找不到 / 唯一 / 有歧義」，不需要把全部列出。
    ///
    /// 吃連線（而不是連線池）：更新流程已持有交易與動物列鎖，再向連線池要第二條連線，
    /// 高併發時可能所有連線都卡在等第二條而逾時。
    pub async fn find_active_pens_by_canonical_code(
        conn: &mut PgConnection,
        canonical_code: &str,
    ) -> Result<Vec<Pen>> {
        // 動態 SQL 已審視：只拼入 `canonical_code_expr("code")`（寫死的欄名組成的運算式），
        // 請求帶來的代碼一律走 `$1` 綁定參數，沒有任何使用者輸入進到 SQL 字串。
        let sql = format!(
            "SELECT * FROM pens WHERE is_active AND {} = $1 ORDER BY id LIMIT 2",
            canonical_code_expr("code")
        );
        let pens = sqlx::query_as::<_, Pen>(sqlx::AssertSqlSafe(sql))
            .bind(canonical_code)
            .fetch_all(conn)
            .await?;
        Ok(pens)
    }

    /// 計算欄位中存活動物數量（排除終態）
    pub async fn count_active_animals_in_pen(conn: &mut PgConnection, pen_id: Uuid) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM animals
            WHERE pen_id = $1
            AND deleted_at IS NULL
            AND status NOT IN ('euthanized', 'sudden_death', 'transferred')
            "#,
        )
        .bind(pen_id)
        .fetch_one(conn)
        .await?;
        Ok(count)
    }
}
