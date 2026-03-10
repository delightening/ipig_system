use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use super::ProtocolService;
use crate::{
    models::{
        CreateProtocolRequest, Protocol, ProtocolActivityType, ProtocolListItem, ProtocolQuery,
        ProtocolResponse, ProtocolRole, ProtocolStatus, UpdateProtocolRequest,
    },
    AppError, Result,
};

impl ProtocolService {
    /// 生成計畫編號
    /// 格式：Pre-{民國年}-{序號:03}
    /// 例如：Pre-114-001, Pre-114-002
    async fn generate_protocol_no(pool: &PgPool) -> Result<String> {
        let now = Utc::now();
        use chrono::Datelike;
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
    pub async fn create(
        pool: &PgPool,
        req: &CreateProtocolRequest,
        created_by: Uuid,
    ) -> Result<Protocol> {
        let protocol_no = Self::generate_protocol_no(pool).await?;
        let pi_user_id = req.pi_user_id.unwrap_or(created_by);

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
        .fetch_one(pool)
        .await?;

        // 記錄狀態歷程
        Self::record_status_change(pool, protocol.id, None, ProtocolStatus::Draft, created_by, None).await?;

        // 關聯 PI 使用者
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
        .execute(pool)
        .await?;

        // 記錄活動紀錄
        Self::record_activity(
            pool,
            protocol.id,
            ProtocolActivityType::Created,
            created_by,
            None,
            Some(protocol.status.as_str().to_string()),
            None,
            None,
            None,
        ).await?;

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
        // 根據規則：在計劃被提交審查與核准前，應為 APIG-{ROC}{03}
        let mut updated_protocols = Vec::new();
        for protocol in &protocols {
            if protocol.status == ProtocolStatus::Submitted || protocol.status == ProtocolStatus::PreReview {
                let needs_apig = protocol.iacuc_no.as_ref()
                    .map(|no| !no.starts_with("APIG-"))
                    .unwrap_or(true);
                
                if needs_apig {
                    // 生成並更新 APIG 編號
                    let apig_no = Self::generate_apig_no(pool).await?;
                    sqlx::query(
                        "UPDATE protocols SET iacuc_no = $2, updated_at = NOW() WHERE id = $1"
                    )
                    .bind(protocol.id)
                    .bind(&apig_no)
                    .execute(pool)
                    .await?;
                    
                    // 更新列表中的編號
                    updated_protocols.push((protocol.id, apig_no));
                }
            }
        }

        // 更新列表中的編號（避免重新查詢）
        for (id, apig_no) in updated_protocols {
            if let Some(protocol) = protocols.iter_mut().find(|p| p.id == id) {
                protocol.iacuc_no = Some(apig_no);
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
                let apig_no = Self::generate_apig_no(pool).await?;
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
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateProtocolRequest,
        updated_by: Uuid,
    ) -> Result<Protocol> {
        // 只有草稿狀態可以編輯
        let protocol = sqlx::query_as::<_, Protocol>(
            "SELECT * FROM protocols WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        if protocol.status != ProtocolStatus::Draft 
            && protocol.status != ProtocolStatus::RevisionRequired 
            && protocol.status != ProtocolStatus::PreReviewRevisionRequired
            && protocol.status != ProtocolStatus::VetRevisionRequired {
            return Err(AppError::BusinessRule("Only draft or revision-required protocols can be edited".to_string()));
        }

        let updated = sqlx::query_as::<_, Protocol>(
            r#"
            UPDATE protocols SET
                title = COALESCE($2, title),
                working_content = COALESCE($3, working_content),
                start_date = COALESCE($4, start_date),
                end_date = COALESCE($5, end_date),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(&req.title)
        .bind(&req.working_content)
        .bind(req.start_date)
        .bind(req.end_date)
        .fetch_one(pool)
        .await?;

        // 記錄專屬活動紀錄（record_activity 現在也會自動同步到 AuditService）
        Self::record_activity(
            pool,
            id,
            ProtocolActivityType::Updated,
            updated_by,
            None,
            None,
            None,
            None,
            None,
        ).await?;

        Ok(updated)
    }
}
