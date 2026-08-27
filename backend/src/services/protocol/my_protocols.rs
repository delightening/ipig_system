use sqlx::PgPool;
use uuid::Uuid;

use super::ProtocolService;
use crate::{
    models::{ProtocolListItem, ProtocolQuery, ProtocolStatus},
    Result,
};

impl ProtocolService {
    /// 取得「我的計劃」列表（純成員制，對所有角色一致）。
    ///
    /// 範圍 = 計畫成員（`user_protocols`: PI / CLIENT / CO_EDITOR）＋ 計劃負責人（SD）
    /// ＋ 被指派的審查委員 / 獸醫。**不**依 `aup.protocol.view_all` 看全部——
    /// 全覽請至「計畫書管理」(`list_protocols`)。要把計畫歸到某使用者，將其加為成員即可。
    /// `assignable`=true 時僅回傳可指派計畫（已核准 + 有 iacuc_no），用於動物
    /// IACUC No. 下拉；不改變「我的計劃」成員制可見範圍（過濾在範圍內取交集）。
    /// `sd_only`=true 時進一步收斂至「自己是該計畫 SD」的計畫（SO 銷貨計畫下拉，
    /// 2026-07-22 裁定，對齊 `authorize_sales_document`；admin 的豁免由 handler
    /// 判斷，本函式不重複檢查角色）。
    ///
    /// 可選過濾（`status` / `keyword` / `pi_user_id` / `start_date` / `end_date` /
    /// `assignable`）取自 `query`，語義與 view_all 路徑 [`ProtocolService::list`] 對齊，
    /// 只在成員可見範圍內收斂、不放大。修正先前成員路徑僅套用 `assignable`、
    /// 前端 status/keyword 篩選被忽略的問題。
    pub async fn get_my_protocols(
        pool: &PgPool,
        user_id: Uuid,
        query: &ProtocolQuery,
        sd_only: bool,
    ) -> Result<Vec<ProtocolListItem>> {
        // 綁定順序須與 super::push_optional_protocol_filters 產生的佔位符一致：
        // $1=user_id，接著 status → keyword → pi_user_id → start_date → end_date。
        let mut q = sqlx::query_as::<_, ProtocolListItem>(sqlx::AssertSqlSafe(
            Self::my_protocols_sql(query, sd_only),
        ))
        .bind(user_id);
        if let Some(status) = query.status {
            if status != ProtocolStatus::Deleted {
                q = q.bind(status);
            }
        }
        if let Some(ref k) = query.keyword {
            q = q.bind(format!("%{}%", k.trim()));
        }
        if let Some(pid) = query.pi_user_id {
            q = q.bind(pid);
        }
        if let Some(sd) = query.start_date {
            q = q.bind(sd);
        }
        if let Some(ed) = query.end_date {
            q = q.bind(ed);
        }
        let mut protocols = q.fetch_all(pool).await?;

        // 「卡在誰」批次補算（同 `core::list`）。「我的計劃」正是 PI 最常來看
        // 「我送審的案子卡在哪」的地方。
        let ids: Vec<Uuid> = protocols.iter().map(|p| p.id).collect();
        if !ids.is_empty() {
            let mut owners =
                crate::services::pending_owner::resolve_for_protocols(pool, &ids).await?;
            for row in &mut protocols {
                row.pending_owner = owners.remove(&row.id);
            }
        }

        Ok(protocols)
    }

    /// 組裝「我的計劃」查詢 SQL（純成員制 + `can_edit` 投影；$1 = user_id，
    /// 可選過濾參數自 $2 起，順序須與 `get_my_protocols` 的綁定一致）。
    fn my_protocols_sql(query: &ProtocolQuery, sd_only: bool) -> String {
        let assignable_filter = if query.assignable {
            super::ASSIGNABLE_PROTOCOL_FILTER
        } else {
            ""
        };
        // SD-only：成員制範圍再取交集（SD-of ⊆ 成員範圍，等價於直接限定 SD）
        let sd_filter = if sd_only {
            "\n            AND p.study_director_user_id = $1"
        } else {
            ""
        };
        // 可選過濾片段（$1 已用於 user_id，過濾參數自 $2 起）；與 build_list_sql 共用組裝器
        // （super::push_optional_protocol_filters），佔位符與下方 bind 順序單一來源、不再漂移。
        let mut opt_filters = String::new();
        super::push_optional_protocol_filters(&mut opt_filters, query, 2);
        // PI 顯示欄（客人優先 basic.pi、fallback FK；委託單位同理）
        let pi_cols = format!(
            "{} as pi_name, {} as pi_organization",
            crate::utils::pi_sql::pi_display_name("u.display_name"),
            crate::utils::pi_sql::pi_sponsor_org("u.organization"),
        );
        format!(
            r#"
            SELECT
                p.id, p.protocol_no, p.iacuc_no, p.title, p.status,
                p.pi_user_id, {pi_cols},
                p.start_date, p.end_date, p.created_at,
                NULLIF(p.working_content->'basic'->>'apply_study_number', '') as apply_study_number,
                COALESCE(
                    p.pi_user_id = $1
                    OR p.study_director_user_id = $1
                    OR EXISTS (SELECT 1 FROM user_protocols up_e
                               WHERE up_e.protocol_id = p.id AND up_e.user_id = $1
                                 AND up_e.role_in_protocol = 'PI'),
                    false
                ) AS can_edit
            FROM protocols p
            LEFT JOIN users u ON p.pi_user_id = u.id
            WHERE p.status != 'DELETED'
            AND (
                -- 計畫成員（PI / CLIENT / CO_EDITOR）
                EXISTS (
                    SELECT 1 FROM user_protocols up
                    WHERE up.protocol_id = p.id AND up.user_id = $1
                )
                OR
                -- PI（pi_user_id 直接關聯）：避免未建 user_protocols 列時 PI 看不到自己的計畫
                p.pi_user_id = $1
                OR
                -- 計劃負責人（Study Director）
                p.study_director_user_id = $1
                OR
                -- 被指派的審查委員（只看非草稿狀態）
                (
                    EXISTS (
                        SELECT 1 FROM review_assignments ra
                        WHERE ra.protocol_id = p.id AND ra.reviewer_id = $1
                    )
                    AND p.status NOT IN ('DRAFT', 'REVISION_REQUIRED')
                )
                OR
                -- 被指派的獸醫審查員
                EXISTS (
                    SELECT 1 FROM vet_review_assignments vra
                    WHERE vra.protocol_id = p.id AND vra.vet_id = $1
                )
            ){sd_filter}{opt_filters}{assignable_filter}
            -- 終態（已否決 / 已結案）沉底，活躍計畫在上；組內維持新→舊。
            ORDER BY (p.status IN ('REJECTED', 'CLOSED')) ASC, p.created_at DESC
            "#
        )
    }

    /// 儲存獸醫審查表
    pub async fn save_vet_review_form(
        pool: &sqlx::PgPool,
        protocol_id: uuid::Uuid,
        vet_id: uuid::Uuid,
        review_form: &serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO vet_review_assignments (id, protocol_id, vet_id, assigned_at, review_form)
            VALUES ($1, $2, $3, NOW(), $4)
            ON CONFLICT (protocol_id) DO UPDATE SET
                review_form = $4,
                completed_at = NOW()
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(protocol_id)
        .bind(vet_id)
        .bind(review_form)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 無過濾（predefault query）：不應含任何可選過濾片段，且成員可見範圍完整。
    #[test]
    fn my_protocols_sql_no_filter_has_no_optional_clauses() {
        let sql = ProtocolService::my_protocols_sql(&ProtocolQuery::default(), false);
        // 斷言「可選過濾片段」不存在——比對實際附加的 `AND …` 片段，而非裸子字串：
        // 成員可見範圍本就含 `p.pi_user_id = $1`（PI 看自己的計畫），不可誤判為過濾。
        assert!(!sql.contains("AND p.status = $"));
        assert!(!sql.contains("ILIKE"));
        assert!(!sql.contains("AND p.pi_user_id = $"));
        assert!(!sql.contains("AND p.start_date >="));
        assert!(!sql.contains("AND p.end_date <="));
        // 成員可見範圍與 SD-only 交集片段
        assert!(sql.contains("FROM user_protocols up"));
        assert!(!sql.contains("AND p.study_director_user_id = $1"));
    }

    // 前端實際會送的 status + keyword：兩者都要落到 SQL，且參數編號自 $2 起連號。
    #[test]
    fn my_protocols_sql_status_keyword_are_applied_with_sequential_params() {
        let query = ProtocolQuery {
            status: Some(ProtocolStatus::Approved),
            keyword: Some("foo".to_string()),
            ..Default::default()
        };
        let sql = ProtocolService::my_protocols_sql(&query, false);
        // status → $2；keyword → $3（三個 ILIKE 共用同一佔位符）
        assert!(sql.contains("AND p.status = $2"), "status 應綁 $2：{sql}");
        assert!(
            sql.contains("p.title ILIKE $3 OR p.protocol_no ILIKE $3 OR p.iacuc_no ILIKE $3"),
            "keyword 應綁 $3：{sql}"
        );
    }

    // 全部可選過濾 + sd_only：驗證五個過濾片段與參數順序 status→pi→日期不衝號。
    #[test]
    fn my_protocols_sql_all_filters_param_numbering() {
        let query = ProtocolQuery {
            status: Some(ProtocolStatus::Approved),
            keyword: Some("x".to_string()),
            pi_user_id: Some(uuid::Uuid::nil()),
            start_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date")),
            end_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 12, 31).expect("valid date")),
            ..Default::default()
        };
        let sql = ProtocolService::my_protocols_sql(&query, true);
        assert!(sql.contains("AND p.status = $2"));
        assert!(sql.contains("ILIKE $3"));
        assert!(sql.contains("AND p.pi_user_id = $4"));
        assert!(sql.contains("AND p.start_date >= $5"));
        assert!(sql.contains("AND p.end_date <= $6"));
        assert!(sql.contains("AND p.study_director_user_id = $1"));
    }
}
