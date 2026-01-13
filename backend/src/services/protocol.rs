use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;

use crate::{
    models::{
        AssignReviewerRequest, ChangeStatusRequest, CreateCommentRequest, CreateProtocolRequest,
        Protocol, ProtocolListItem, ProtocolQuery, ProtocolResponse, ProtocolStatus,
        ProtocolStatusHistory, ProtocolVersion, ReviewAssignment, ReviewComment,
        ReviewCommentResponse, UpdateProtocolRequest, ProtocolRole,
    },
    AppError, Result,
};

pub struct ProtocolService;

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
        
        if let Some(status) = query.status {
            // 如果指定了狀態過濾，且不是 DELETED，則添加狀態條件
            if status != ProtocolStatus::Deleted {
                sql.push_str(&format!(" AND p.status = '{}'", status.as_str()));
            }
        }
        if query.pi_user_id.is_some() {
            sql.push_str(" AND p.pi_user_id = $2");
        }
        if query.keyword.is_some() {
            sql.push_str(" AND (p.title ILIKE $3 OR p.protocol_no ILIKE $3 OR p.iacuc_no ILIKE $3)");
        }

        sql.push_str(" ORDER BY p.created_at DESC");

        // 由於 SQLx 的限制，這裡使用簡化的查詢
        let protocols: Vec<ProtocolListItem> = sqlx::query_as(&sql)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

        Ok(protocols)
    }

    /// 取得單一計畫
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<ProtocolResponse> {
        let protocol = sqlx::query_as::<_, Protocol>(
            "SELECT * FROM protocols WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        // 取得 PI 資訊
        let pi_info: Option<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT display_name, email, organization FROM users WHERE id = $1"
        )
        .bind(protocol.pi_user_id)
        .fetch_optional(pool)
        .await?;

        let (pi_name, pi_email, pi_organization) = pi_info.unwrap_or_default();

        Ok(ProtocolResponse {
            status_display: protocol.status.display_name().to_string(),
            protocol,
            pi_name: Some(pi_name),
            pi_email: Some(pi_email),
            pi_organization,
        })
    }

    /// 更新計畫
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateProtocolRequest,
    ) -> Result<Protocol> {
        // 只有草稿狀態可以編輯
        let protocol = sqlx::query_as::<_, Protocol>(
            "SELECT * FROM protocols WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        if protocol.status != ProtocolStatus::Draft && protocol.status != ProtocolStatus::RevisionRequired {
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

        Ok(updated)
    }

    /// 驗證計畫內容
    fn validate_protocol_content(content: &Option<Value>) -> Result<()> {
        let content = content.as_ref().ok_or_else(|| AppError::Validation("Protocol content is empty".to_string()))?;
        
        // 驗證基本資料
        let basic = content.get("basic").ok_or_else(|| AppError::Validation("Missing 'basic' section".to_string()))?;
        
        // 驗證標題 (AUP 2.2)
        if basic.get("study_title").and_then(|v| v.as_str()).unwrap_or("").trim().is_empty() {
             return Err(AppError::Validation("Study title is required".to_string())); 
        }

        // 驗證 GLP (AUP 2.1)
        let is_glp = basic.get("is_glp").and_then(|v| v.as_bool()).unwrap_or(false);
        if is_glp {
             let authorities = basic.get("registration_authorities").and_then(|v| v.as_array());
             if authorities.map(|a| a.is_empty()).unwrap_or(true) {
                 return Err(AppError::Validation("Registration authorities required for GLP study".to_string()));
             }
        }

        // 驗證計畫類型 (AUP 2.7)
        if basic.get("project_type").and_then(|v| v.as_str()).unwrap_or("").trim().is_empty() {
             return Err(AppError::Validation("Project type is required".to_string()));
        }

        // 驗證動物總數 (AUP 8.1)
        if let Some(_animals_section) = content.get("animals") {
             // 這裡可以做更多檢查
        }

        Ok(())
    }

    /// 提交計畫
    pub async fn submit(pool: &PgPool, id: Uuid, submitted_by: Uuid) -> Result<Protocol> {
        let protocol = sqlx::query_as::<_, Protocol>(
            "SELECT * FROM protocols WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        // 檢查狀態轉移是否合法
        if protocol.status != ProtocolStatus::Draft && protocol.status != ProtocolStatus::RevisionRequired {
            return Err(AppError::BusinessRule(
                format!("Cannot submit protocol in {} status", protocol.status.as_str())
            ));
        }

        // 驗證內容
        Self::validate_protocol_content(&protocol.working_content)?;

        let new_status = if protocol.status == ProtocolStatus::RevisionRequired {
            ProtocolStatus::Resubmitted
        } else {
            ProtocolStatus::Submitted
        };

        // 建立版本快照
        let version_no = Self::get_next_version_no(pool, id).await?;
        sqlx::query(
            r#"
            INSERT INTO protocol_versions (id, protocol_id, version_no, content_snapshot, submitted_at, submitted_by)
            VALUES ($1, $2, $3, $4, NOW(), $5)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(version_no)
        .bind(&protocol.working_content)
        .bind(submitted_by)
        .execute(pool)
        .await?;

        // 更新狀態
        let updated = sqlx::query_as::<_, Protocol>(
            "UPDATE protocols SET status = $2, updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .bind(new_status)
        .fetch_one(pool)
        .await?;

        // 記錄狀態變更
        Self::record_status_change(pool, id, Some(protocol.status), new_status, submitted_by, None).await?;

        Ok(updated)
    }

    /// 變更狀態
    pub async fn change_status(
        pool: &PgPool,
        id: Uuid,
        req: &ChangeStatusRequest,
        changed_by: Uuid,
    ) -> Result<Protocol> {
        let protocol = sqlx::query_as::<_, Protocol>(
            "SELECT * FROM protocols WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        // TODO: 驗證狀態轉移是否合法（根據角色和當前狀態）

        // 如果核准，生成 IACUC NO
        let iacuc_no = if req.to_status == ProtocolStatus::Approved || req.to_status == ProtocolStatus::ApprovedWithConditions {
            Some(Self::generate_iacuc_no())
        } else {
            protocol.iacuc_no.clone()
        };

        let updated = sqlx::query_as::<_, Protocol>(
            r#"
            UPDATE protocols SET 
                status = $2, 
                iacuc_no = COALESCE($3, iacuc_no),
                updated_at = NOW() 
            WHERE id = $1 
            RETURNING *
            "#
        )
        .bind(id)
        .bind(req.to_status)
        .bind(iacuc_no)
        .fetch_one(pool)
        .await?;

        // 記錄狀態變更
        Self::record_status_change(pool, id, Some(protocol.status), req.to_status, changed_by, req.remark.clone()).await?;

        Ok(updated)
    }

    /// 生成 IACUC 編號
    fn generate_iacuc_no() -> String {
        let now = Utc::now();
        let year = now.format("%y");
        let random: u32 = rand::random::<u32>() % 10000;
        format!("PIG-{}{:04}", year, random)
    }

    /// 取得下一個版本號
    async fn get_next_version_no(pool: &PgPool, protocol_id: Uuid) -> Result<i32> {
        let max_version: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(version_no) FROM protocol_versions WHERE protocol_id = $1"
        )
        .bind(protocol_id)
        .fetch_one(pool)
        .await?;

        Ok(max_version.unwrap_or(0) + 1)
    }

    /// 記錄狀態變更
    async fn record_status_change(
        pool: &PgPool,
        protocol_id: Uuid,
        from_status: Option<ProtocolStatus>,
        to_status: ProtocolStatus,
        changed_by: Uuid,
        remark: Option<String>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(protocol_id)
        .bind(from_status)
        .bind(to_status)
        .bind(changed_by)
        .bind(remark)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 取得版本列表
    pub async fn get_versions(pool: &PgPool, protocol_id: Uuid) -> Result<Vec<ProtocolVersion>> {
        let versions = sqlx::query_as::<_, ProtocolVersion>(
            "SELECT * FROM protocol_versions WHERE protocol_id = $1 ORDER BY version_no DESC"
        )
        .bind(protocol_id)
        .fetch_all(pool)
        .await?;

        Ok(versions)
    }

    /// 取得狀態歷程
    pub async fn get_status_history(pool: &PgPool, protocol_id: Uuid) -> Result<Vec<ProtocolStatusHistory>> {
        let history = sqlx::query_as::<_, ProtocolStatusHistory>(
            "SELECT * FROM protocol_status_history WHERE protocol_id = $1 ORDER BY created_at DESC"
        )
        .bind(protocol_id)
        .fetch_all(pool)
        .await?;

        Ok(history)
    }

    /// 指派審查人員
    pub async fn assign_reviewer(
        pool: &PgPool,
        req: &AssignReviewerRequest,
        assigned_by: Uuid,
    ) -> Result<ReviewAssignment> {
        let assignment = sqlx::query_as::<_, ReviewAssignment>(
            r#"
            INSERT INTO review_assignments (id, protocol_id, reviewer_id, assigned_by, assigned_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (protocol_id, reviewer_id) DO UPDATE SET assigned_at = NOW()
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(req.protocol_id)
        .bind(req.reviewer_id)
        .bind(assigned_by)
        .fetch_one(pool)
        .await?;

        Ok(assignment)
    }

    /// 新增審查意見
    pub async fn add_comment(
        pool: &PgPool,
        req: &CreateCommentRequest,
        reviewer_id: Uuid,
    ) -> Result<ReviewComment> {
        let comment = sqlx::query_as::<_, ReviewComment>(
            r#"
            INSERT INTO review_comments (id, protocol_version_id, reviewer_id, content, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(req.protocol_version_id)
        .bind(reviewer_id)
        .bind(&req.content)
        .fetch_one(pool)
        .await?;

        Ok(comment)
    }

    /// 取得審查意見
    pub async fn get_comments(pool: &PgPool, protocol_version_id: Uuid) -> Result<Vec<ReviewCommentResponse>> {
        let comments = sqlx::query_as::<_, ReviewCommentResponse>(
            r#"
            SELECT 
                c.id, c.protocol_version_id, c.reviewer_id,
                u.display_name as reviewer_name, u.email as reviewer_email,
                c.content, c.is_resolved, c.resolved_by, c.resolved_at, c.created_at
            FROM review_comments c
            LEFT JOIN users u ON c.reviewer_id = u.id
            WHERE c.protocol_version_id = $1
            ORDER BY c.created_at DESC
            "#
        )
        .bind(protocol_version_id)
        .fetch_all(pool)
        .await?;

        Ok(comments)
    }

    /// 解決審查意見
    pub async fn resolve_comment(pool: &PgPool, comment_id: Uuid, resolved_by: Uuid) -> Result<ReviewComment> {
        let comment = sqlx::query_as::<_, ReviewComment>(
            r#"
            UPDATE review_comments SET
                is_resolved = true,
                resolved_by = $2,
                resolved_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(comment_id)
        .bind(resolved_by)
        .fetch_one(pool)
        .await?;

        Ok(comment)
    }

    /// 取得我的計畫列表（依使用者）
    pub async fn get_my_protocols(pool: &PgPool, user_id: Uuid) -> Result<Vec<ProtocolListItem>> {
        let protocols = sqlx::query_as::<_, ProtocolListItem>(
            r#"
            SELECT 
                p.id, p.protocol_no, p.iacuc_no, p.title, p.status,
                p.pi_user_id, u.display_name as pi_name, u.organization as pi_organization,
                p.start_date, p.end_date, p.created_at,
                NULLIF(p.working_content->'basic'->>'apply_study_number', '') as apply_study_number
            FROM protocols p
            LEFT JOIN users u ON p.pi_user_id = u.id
            INNER JOIN user_protocols up ON p.id = up.protocol_id
            WHERE up.user_id = $1 AND p.status != 'DELETED'
            ORDER BY p.created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(protocols)
    }
}
