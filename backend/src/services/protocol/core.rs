use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Datelike, Utc};

use super::ProtocolService;
use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, CreateProtocolRequest, Protocol, ProtocolActivityType,
        ProtocolListItem, ProtocolQuery, ProtocolResponse, ProtocolRole, ProtocolStatus,
        UpdateProtocolRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

const CONFLICT_MSG: &str = "此記錄已被其他人修改，請重新載入後再試。";

impl ProtocolService {
    /// 生成計畫編號
    /// 格式：Pre-{民國年}-{序號:03}
    /// 例如：Pre-114-001, Pre-114-002
    async fn generate_protocol_no(pool: &PgPool) -> Result<String> {
        let now = Utc::now();
        let year = now.year();
        // 民國年 = 西元年 - 1911
        let roc_year = year - 1911;
        
        // 查詢該民國年的所有計畫編號
        let prefix = format!("Pre-{}-", roc_year);
        let protocol_nos: Vec<String> = sqlx::query_scalar(
            "SELECT protocol_no FROM protocols WHERE protocol_no LIKE $1"
        )
        .bind(format!("{}%", prefix))
        .fetch_all(pool)
        .await?;

        // 解析序號並找出最大值
        let max_seq = protocol_nos
            .iter()
            .filter_map(|no| {
                // 格式：Pre-114-001，提取最後的數字部分
                let parts: Vec<&str> = no.split('-').collect();
                if parts.len() >= 3 {
                    parts[2].parse::<i32>().ok()
                } else {
                    None
                }
            })
            .max();

        let seq = max_seq.map(|s| s + 1).unwrap_or(1);

        Ok(format!("{}{:03}", prefix, seq))
    }

    /// 建立計畫
    ///
    /// R30-29：INSERT + user_protocols + audit-in-tx 全 tx 原子。
    /// 失敗整 tx rollback，不留半成品。
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateProtocolRequest,
        created_by: Uuid,
    ) -> Result<Protocol> {
        let protocol_no = Self::generate_protocol_no(pool).await?;
        let pi_user_id = req.pi_user_id.unwrap_or(created_by);

        let mut tx = pool.begin().await?;

        let protocol = sqlx::query_as::<_, Protocol>(
            r#"
            INSERT INTO protocols (
                id, protocol_no, title, status, pi_user_id, working_content,
                start_date, end_date, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&protocol_no)
        .bind(&req.title)
        .bind(ProtocolStatus::Draft)
        .bind(pi_user_id)
        .bind(&req.working_content)
        .bind(req.start_date)
        .bind(req.end_date)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // user_protocols：PI 連結
        sqlx::query(
            r#"
            INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
            VALUES ($1, $2, $3, NOW(), $4)
            ON CONFLICT (user_id, protocol_id) DO NOTHING
            "#
        )
        .bind(pi_user_id)
        .bind(protocol.id)
        .bind(ProtocolRole::Pi)
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        // protocol_activities + user_activity_logs（同 tx）
        Self::record_activity_tx(
            &mut tx,
            actor,
            protocol.id,
            ProtocolActivityType::Created,
            None,
            Some(protocol.status.as_str().to_string()),
            None,
            None,
            None,
        )
        .await?;

        // Service-driven audit：完整 create 快照進 HMAC chain
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_CREATE",
                entity: Some(AuditEntity::new("protocol", protocol.id, &protocol.title)),
                data_diff: Some(DataDiff::create_only(&protocol)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(protocol)
    }

    /// 複製既有計畫建立新草稿
    /// 複製 title、working_content、start_date、end_date，新計畫狀態為 DRAFT
    ///
    /// R30-29：INSERT + user_protocols + audit-in-tx 全 tx 原子。
    pub async fn copy(
        pool: &PgPool,
        actor: &ActorContext,
        source_id: Uuid,
        copied_by: Uuid,
    ) -> Result<Protocol> {
        let source = sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1")
            .bind(source_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("來源計畫不存在".to_string()))?;

        let new_protocol_no = Self::generate_protocol_no(pool).await?;
        let new_title = format!("（複製）{}", source.title);
        let pi_user_id = source.pi_user_id;

        let mut tx = pool.begin().await?;

        let protocol = sqlx::query_as::<_, Protocol>(
            r#"
            INSERT INTO protocols (
                id, protocol_no, title, status, pi_user_id, working_content,
                start_date, end_date, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&new_protocol_no)
        .bind(&new_title)
        .bind(ProtocolStatus::Draft)
        .bind(pi_user_id)
        .bind(&source.working_content)
        .bind(source.start_date)
        .bind(source.end_date)
        .bind(copied_by)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
            VALUES ($1, $2, $3, NOW(), $4)
            ON CONFLICT (user_id, protocol_id) DO NOTHING
            "#
        )
        .bind(pi_user_id)
        .bind(protocol.id)
        .bind(ProtocolRole::Pi)
        .bind(copied_by)
        .execute(&mut *tx)
        .await?;

        Self::record_activity_tx(
            &mut tx,
            actor,
            protocol.id,
            ProtocolActivityType::Created,
            None,
            Some(protocol.status.as_str().to_string()),
            None,
            Some(format!("複製自計畫 {}", source.protocol_no)),
            None,
        )
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_CREATE",
                entity: Some(AuditEntity::new("protocol", protocol.id, &protocol.title)),
                data_diff: Some(DataDiff::create_only(&protocol)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(protocol)
    }

    /// 查詢計畫列表
    pub async fn list(pool: &PgPool, query: &ProtocolQuery) -> Result<Vec<ProtocolListItem>> {
        let mut sql = String::from(
            r#"
            SELECT 
                p.id, p.protocol_no, p.iacuc_no, p.title, p.status,
                p.pi_user_id, u.display_name as pi_name, u.organization as pi_organization,
                p.start_date, p.end_date, p.created_at,
                NULLIF(p.working_content->'basic'->>'apply_study_number', '') as apply_study_number
            FROM protocols p
            LEFT JOIN users u ON p.pi_user_id = u.id
            WHERE 1=1
            "#
        );

        // 始終排除已刪除的計畫書
        sql.push_str(" AND p.status != 'DELETED'");
        
        // R7-P0-2: 使用參數化查詢取代 format! 字串拼接，避免潛在 SQL injection 風險
        // 動態追蹤下一個參數編號
        let mut param_idx = 1u32;

        if let Some(status) = query.status {
            if status != ProtocolStatus::Deleted {
                sql.push_str(&format!(" AND p.status = ${}", param_idx));
                param_idx += 1;
            }
        }
        if query.keyword.is_some() {
            sql.push_str(&format!(
                " AND (p.title ILIKE ${p} OR p.protocol_no ILIKE ${p} OR p.iacuc_no ILIKE ${p})",
                p = param_idx
            ));
            param_idx += 1;
        }
        if query.pi_user_id.is_some() {
            sql.push_str(&format!(" AND p.pi_user_id = ${}", param_idx));
        }

        sql.push_str(" ORDER BY p.created_at DESC");

        let mut query_builder = sqlx::query_as::<_, ProtocolListItem>(&sql);
        if let Some(status) = query.status {
            if status != ProtocolStatus::Deleted {
                query_builder = query_builder.bind(status.as_str());
            }
        }
        if let Some(ref k) = query.keyword {
            let pattern = format!("%{}%", k.trim());
            query_builder = query_builder.bind(pattern);
        }
        if let Some(pid) = query.pi_user_id {
            query_builder = query_builder.bind(pid);
        }
        let mut protocols: Vec<ProtocolListItem> = query_builder
            .fetch_all(pool)
            .await
            .unwrap_or_default();

        // 批量修復缺少 APIG 編號的 Submitted 或 PreReview 狀態計畫書
        let needs_apig_ids: Vec<Uuid> = protocols
            .iter()
            .filter(|p| {
                (p.status == ProtocolStatus::Submitted || p.status == ProtocolStatus::PreReview)
                    && p.iacuc_no.as_ref().map(|no| !no.starts_with("APIG-")).unwrap_or(true)
            })
            .map(|p| p.id)
            .collect();

        if !needs_apig_ids.is_empty() {
            // 批量生成：一次查詢 max seq，然後在記憶體中分配 N 個連續編號
            let apig_nos = Self::generate_apig_nos_batch_pool(pool, needs_apig_ids.len()).await?;

            // 批量 UPDATE：使用 UNNEST 一次更新所有
            sqlx::query(
                r#"
                UPDATE protocols SET iacuc_no = d.apig_no, updated_at = NOW()
                FROM UNNEST($1::uuid[], $2::text[]) AS d(id, apig_no)
                WHERE protocols.id = d.id
                "#,
            )
            .bind(&needs_apig_ids)
            .bind(&apig_nos)
            .execute(pool)
            .await?;

            // 更新列表中的編號（避免重新查詢）
            for (id, apig_no) in needs_apig_ids.iter().zip(apig_nos.iter()) {
                if let Some(protocol) = protocols.iter_mut().find(|p| &p.id == id) {
                    protocol.iacuc_no = Some(apig_no.clone());
                }
            }
        }

        Ok(protocols)
    }

    /// 取得單一計畫
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<ProtocolResponse> {
        let mut protocol = sqlx::query_as::<_, Protocol>(
            "SELECT * FROM protocols WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        // 如果狀態是 Submitted 或 PreReview 但沒有 APIG 編號，自動生成
        // 根據規則：在計劃被提交審查與核准前，應為 APIG-{ROC}{03}
        if protocol.status == ProtocolStatus::Submitted || protocol.status == ProtocolStatus::PreReview {
            let needs_apig = protocol.iacuc_no.as_ref()
                .map(|no| !no.starts_with("APIG-"))
                .unwrap_or(true);
            
            if needs_apig {
                let apig_no = Self::generate_apig_no_pool(pool).await?;
                protocol = sqlx::query_as::<_, Protocol>(
                    "UPDATE protocols SET iacuc_no = $2, updated_at = NOW() WHERE id = $1 RETURNING *"
                )
                .bind(id)
                .bind(&apig_no)
                .fetch_one(pool)
                .await?;
            }
        }

        // 取得 PI 資訊
        let pi_info: Option<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT display_name, email, organization FROM users WHERE id = $1"
        )
        .bind(protocol.pi_user_id)
        .fetch_optional(pool)
        .await?;

        let (pi_name, pi_email, pi_organization) = pi_info.unwrap_or_default();

        // 獲取獸醫審查指派
        let vet_review = sqlx::query_as::<_, crate::models::VetReviewAssignment>(
            "SELECT * FROM vet_review_assignments WHERE protocol_id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(ProtocolResponse {
            status_display: protocol.status.display_name().to_string(),
            protocol,
            pi_name: Some(pi_name),
            pi_email: Some(pi_email),
            pi_organization,
            vet_review,
        })
    }

    /// 更新計畫
    ///
    /// R30-5：version optimistic lock 防 lost update。
    /// R30-29：tx + FOR UPDATE before snapshot + audit-in-tx + 完整 before/after diff。
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateProtocolRequest,
    ) -> Result<Protocol> {
        let mut tx = pool.begin().await?;

        // FOR UPDATE 鎖行 + 完整 before snapshot（在同 tx 內，與後續 UPDATE 一致）
        let before: Protocol = sqlx::query_as::<_, Protocol>(
            "SELECT * FROM protocols WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        if before.status != ProtocolStatus::Draft
            && before.status != ProtocolStatus::RevisionRequired
            && before.status != ProtocolStatus::PreReviewRevisionRequired
            && before.status != ProtocolStatus::VetRevisionRequired
        {
            return Err(AppError::BusinessRule(
                "Only draft or revision-required protocols can be edited".to_string(),
            ));
        }

        // R30-5：version optimistic lock + version+1
        // $6 = NULL → 跳過版本檢查（向後相容）；命中 0 row → 409 Conflict
        let updated = sqlx::query_as::<_, Protocol>(
            r#"
            UPDATE protocols SET
                title = COALESCE($2, title),
                working_content = COALESCE($3, working_content),
                start_date = COALESCE($4, start_date),
                end_date = COALESCE($5, end_date),
                version = version + 1,
                updated_at = NOW()
            WHERE id = $1
              AND ($6::INT IS NULL OR version = $6)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&req.title)
        .bind(&req.working_content)
        .bind(req.start_date)
        .bind(req.end_date)
        .bind(req.version)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict(CONFLICT_MSG.to_string()))?;

        // protocol_activities + user_activity_logs（同 tx，UPDATED 事件）
        Self::record_activity_tx(
            &mut tx,
            actor,
            id,
            ProtocolActivityType::Updated,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        // Service-driven audit：完整 before/after diff 進 HMAC chain
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_UPDATE",
                entity: Some(AuditEntity::new("protocol", id, &updated.title)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(updated)
    }
}
