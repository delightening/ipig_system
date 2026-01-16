use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;
use validator::Validate;

use crate::{
    models::{
        AssignReviewerRequest, AssignCoEditorRequest, ChangeStatusRequest, CreateCommentRequest, CreateProtocolRequest,
        Protocol, ProtocolListItem, ProtocolQuery, ProtocolResponse, ProtocolStatus,
        ProtocolStatusHistory, ProtocolVersion, ReplyCommentRequest, ReviewAssignment, ReviewComment,
        ReviewCommentResponse, UpdateProtocolRequest, ProtocolRole, UserProtocol, CreatePartnerRequest, PartnerType,
    },
    services::PartnerService,
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
        let mut protocols: Vec<ProtocolListItem> = sqlx::query_as(&sql)
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

        // 如果狀態變為 Submitted，生成 APIG 編號（在計劃被提交審查與核准前）
        let new_iacuc_no = if new_status == ProtocolStatus::Submitted {
            // 如果還沒有 APIG 編號，則生成
            let needs_apig = protocol.iacuc_no.as_ref()
                .map(|no| !no.starts_with("APIG-"))
                .unwrap_or(true);
            
            if needs_apig {
                Some(Self::generate_apig_no(pool).await?)
            } else {
                protocol.iacuc_no.clone()
            }
        } else {
            protocol.iacuc_no.clone()
        };

        // 更新狀態和 IACUC 編號
        let updated = sqlx::query_as::<_, Protocol>(
            "UPDATE protocols SET status = $2, iacuc_no = $3, updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .bind(new_status)
        .bind(&new_iacuc_no)
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

        // IACUC 編號生成規則：
        // 1. 在計劃被提交審查與核准前（Submitted 狀態），生成 APIG-{ROC}{03}
        // 2. 在計劃被核准時（Approved 狀態），生成 PIG-{ROC}{03}
        let new_iacuc_no = if req.to_status == ProtocolStatus::Submitted {
            // 如果還沒有 APIG 編號，則生成
            let needs_apig = protocol.iacuc_no.as_ref()
                .map(|no| !no.starts_with("APIG-"))
                .unwrap_or(true);
            
            if needs_apig {
                Some(Self::generate_apig_no(pool).await?)
            } else {
                protocol.iacuc_no.clone()
            }
        } else if req.to_status == ProtocolStatus::PreReview {
            // 如果狀態變為 PreReview 但還沒有 APIG 編號，則生成（備用邏輯）
            let needs_apig = protocol.iacuc_no.as_ref()
                .map(|no| !no.starts_with("APIG-"))
                .unwrap_or(true);
            
            if needs_apig {
                Some(Self::generate_apig_no(pool).await?)
            } else {
                protocol.iacuc_no.clone()
            }
        } else if req.to_status == ProtocolStatus::Approved || req.to_status == ProtocolStatus::ApprovedWithConditions {
            // 核准時生成 IACUC 編號（PIG-{ROC}{03}）
            Some(Self::generate_iacuc_no(pool).await?)
        } else {
            protocol.iacuc_no.clone()
        };

        let updated = sqlx::query_as::<_, Protocol>(
            r#"
            UPDATE protocols SET 
                status = $2, 
                iacuc_no = $3,
                updated_at = NOW() 
            WHERE id = $1 
            RETURNING *
            "#
        )
        .bind(id)
        .bind(req.to_status)
        .bind(&new_iacuc_no)
        .fetch_one(pool)
        .await?;

        // 記錄狀態變更
        Self::record_status_change(pool, id, Some(protocol.status), req.to_status, changed_by, req.remark.clone()).await?;

        // 當計劃通過時，自動依照 IACUC No. 自動填入客戶
        if (req.to_status == ProtocolStatus::Approved || req.to_status == ProtocolStatus::ApprovedWithConditions) 
            && new_iacuc_no.is_some() 
        {
            let iacuc_no = new_iacuc_no.as_ref().unwrap();
            
            // 檢查是否已存在該客戶（客戶代碼 = IACUC No.）
            let existing_customer: Option<uuid::Uuid> = sqlx::query_scalar(
                "SELECT id FROM partners WHERE partner_type = 'customer' AND code = $1"
            )
            .bind(iacuc_no)
            .fetch_optional(pool)
            .await?;

            // 如果不存在，則創建新客戶
            if existing_customer.is_none() {
                let create_req = CreatePartnerRequest {
                    partner_type: PartnerType::Customer,
                    code: Some(iacuc_no.clone()),
                    supplier_category: None,
                    name: iacuc_no.clone(),
                    tax_id: None,
                    phone: None,
                    email: None,
                    address: None,
                    payment_terms: None,
                };
                
                // 驗證請求
                if let Err(validation_errors) = create_req.validate() {
                    tracing::warn!("Failed to validate customer creation request for IACUC {}: {:?}", iacuc_no, validation_errors);
                } else {
                    // 創建客戶，忽略錯誤（例如代碼衝突），因為可能已經存在
                    if let Err(e) = PartnerService::create(pool, &create_req).await {
                        tracing::warn!("Failed to create customer for IACUC {}: {}", iacuc_no, e);
                    } else {
                        tracing::info!("Automatically created customer for IACUC: {}", iacuc_no);
                    }
                }
            }
        }

        // 當計劃結案時，自動停用對應的客戶
        if req.to_status == ProtocolStatus::Closed && protocol.iacuc_no.is_some() {
            let iacuc_no = protocol.iacuc_no.as_ref().unwrap();
            
            // 查找對應的客戶（客戶代碼 = IACUC No.）
            let customer_id: Option<uuid::Uuid> = sqlx::query_scalar(
                "SELECT id FROM partners WHERE partner_type = 'customer' AND code = $1"
            )
            .bind(iacuc_no)
            .fetch_optional(pool)
            .await?;

            // 如果找到客戶，則停用該客戶
            if let Some(customer_id) = customer_id {
                let result = sqlx::query(
                    "UPDATE partners SET is_active = false, updated_at = NOW() WHERE id = $1"
                )
                .bind(customer_id)
                .execute(pool)
                .await?;

                if result.rows_affected() > 0 {
                    tracing::info!("Automatically deactivated customer for closed IACUC: {}", iacuc_no);
                } else {
                    tracing::warn!("Failed to deactivate customer for IACUC {}: customer not found", iacuc_no);
                }
            } else {
                tracing::warn!("No customer found for closed IACUC: {}", iacuc_no);
            }
        }

        Ok(updated)
    }

    /// 生成 APIG 編號
    /// 格式：APIG-{ROC}{03}
    /// {ROC} 為民國年（西元年 - 1911）
    /// {03} 為流水號（3位數，補零）
    /// 
    /// 注意：需要避免重複使用已經轉換為 PIG 的編號
    /// 例如：如果 APIG-115001 已經變成 PIG-115001，則流水號 001 不應再被使用
    async fn generate_apig_no(pool: &PgPool) -> Result<String> {
        let now = Utc::now();
        use chrono::Datelike;
        let year = now.year();
        // 民國年 = 西元年 - 1911
        let roc_year = year - 1911;
        
        // 查詢該民國年的所有 APIG 編號
        // 格式：APIG-{ROC年}{3位數流水號}，例如：APIG-114001, APIG-115001
        let apig_prefix = format!("APIG-{}", roc_year);
        let apig_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL"
        )
        .bind(format!("{}%", apig_prefix))
        .fetch_all(pool)
        .await?;

        // 查詢該民國年的所有 PIG 編號（因為 PIG 編號可能曾經是 APIG 編號）
        // 格式：PIG-{ROC年}{3位數流水號}，例如：PIG-115001
        // 這些流水號不應再被用於新的 APIG 編號
        let pig_prefix = format!("PIG-{}", roc_year);
        let pig_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL"
        )
        .bind(format!("{}%", pig_prefix))
        .fetch_all(pool)
        .await?;

        // 解析 APIG 編號的流水號
        let apig_seqs: Vec<i32> = apig_nos
            .iter()
            .filter_map(|no| {
                if no.starts_with(&apig_prefix) {
                    let seq_str = &no[apig_prefix.len()..];
                    seq_str.parse::<i32>().ok()
                } else {
                    None
                }
            })
            .collect();

        // 解析 PIG 編號的流水號（這些流水號曾經是 APIG 編號，不應重複使用）
        let pig_seqs: Vec<i32> = pig_nos
            .iter()
            .filter_map(|no| {
                if no.starts_with(&pig_prefix) {
                    let seq_str = &no[pig_prefix.len()..];
                    seq_str.parse::<i32>().ok()
                } else {
                    None
                }
            })
            .collect();

        // 合併所有已使用的流水號（包括當前的 APIG 和曾經是 APIG 的 PIG）
        let mut all_used_seqs: Vec<i32> = apig_seqs;
        all_used_seqs.extend(pig_seqs);
        
        // 找出最大值
        let max_seq = all_used_seqs.iter().max().copied();

        // 下一個流水號（從1開始，如果沒有現有編號）
        let seq = max_seq.map(|s| s + 1).unwrap_or(1);
        
        // 確保流水號不超過999
        if seq > 999 {
            return Err(AppError::Internal(
                format!("APIG 編號流水號已達上限（999），無法生成新編號")
            ));
        }

        Ok(format!("{}{:03}", apig_prefix, seq))
    }

    /// 生成 IACUC 編號
    /// 格式：PIG-{ROC}{03}
    /// {ROC} 為民國年（西元年 - 1911）
    /// {03} 為流水號（3位數，補零）
    async fn generate_iacuc_no(pool: &PgPool) -> Result<String> {
        let now = Utc::now();
        use chrono::Datelike;
        let year = now.year();
        // 民國年 = 西元年 - 1911
        let roc_year = year - 1911;
        
        // 查詢該民國年的所有 IACUC 編號
        // 格式：PIG-{ROC年}{3位數流水號}，例如：PIG-114017, PIG-115001
        let prefix = format!("PIG-{}", roc_year);
        let iacuc_nos: Vec<String> = sqlx::query_scalar(
            "SELECT iacuc_no FROM protocols WHERE iacuc_no LIKE $1 AND iacuc_no IS NOT NULL"
        )
        .bind(format!("{}%", prefix))
        .fetch_all(pool)
        .await?;

        // 解析流水號並找出最大值
        // IACUC 編號格式：PIG-{ROC年}{3位數流水號}
        // 例如：PIG-114017 → ROC年=114, 流水號=017
        let max_seq = iacuc_nos
            .iter()
            .filter_map(|no| {
                // 移除前綴 "PIG-{ROC年}"，取得流水號部分
                if no.starts_with(&prefix) {
                    let seq_str = &no[prefix.len()..];
                    seq_str.parse::<i32>().ok()
                } else {
                    None
                }
            })
            .max();

        // 下一個流水號（從1開始，如果沒有現有編號）
        let seq = max_seq.map(|s| s + 1).unwrap_or(1);
        
        // 確保流水號不超過999
        if seq > 999 {
            return Err(AppError::Internal(
                format!("IACUC 編號流水號已達上限（999），無法生成新編號")
            ));
        }

        Ok(format!("{}{:03}", prefix, seq))
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

    /// 指派 co-editor（試驗工作人員）
    pub async fn assign_co_editor(
        pool: &PgPool,
        req: &AssignCoEditorRequest,
        assigned_by: Uuid,
    ) -> Result<UserProtocol> {
        // 驗證協議存在
        let protocol_exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM protocols WHERE id = $1)"
        )
        .bind(req.protocol_id)
        .fetch_one(pool)
        .await?;

        if !protocol_exists.0 {
            return Err(AppError::NotFound("Protocol not found".to_string()));
        }

        // 驗證用戶存在且是 EXPERIMENT_STAFF 角色
        let user_has_role: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_roles ur
                INNER JOIN roles r ON ur.role_id = r.id
                WHERE ur.user_id = $1 AND r.code = 'EXPERIMENT_STAFF'
            )
            "#
        )
        .bind(req.user_id)
        .fetch_one(pool)
        .await?;

        if !user_has_role.0 {
            return Err(AppError::Validation("User must have EXPERIMENT_STAFF role to be assigned as co-editor".to_string()));
        }

        // 指派為 co-editor
        let assignment = sqlx::query_as::<_, UserProtocol>(
            r#"
            INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
            VALUES ($1, $2, 'CO_EDITOR', NOW(), $3)
            ON CONFLICT (user_id, protocol_id) 
            DO UPDATE SET 
                role_in_protocol = 'CO_EDITOR',
                granted_at = NOW(),
                granted_by = $3
            RETURNING *
            "#
        )
        .bind(req.user_id)
        .bind(req.protocol_id)
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

    /// 取得審查意見（含回覆）
    pub async fn get_comments(pool: &PgPool, protocol_version_id: Uuid) -> Result<Vec<ReviewCommentResponse>> {
        let comments = sqlx::query_as::<_, ReviewCommentResponse>(
            r#"
            SELECT 
                c.id, c.protocol_version_id, c.reviewer_id,
                u.display_name as reviewer_name, u.email as reviewer_email,
                c.content, c.is_resolved, c.resolved_by, c.resolved_at, 
                c.parent_comment_id, c.replied_by,
                ru.display_name as replied_by_name, ru.email as replied_by_email,
                c.created_at
            FROM review_comments c
            LEFT JOIN users u ON c.reviewer_id = u.id
            LEFT JOIN users ru ON c.replied_by = ru.id
            WHERE c.protocol_version_id = $1
            ORDER BY 
                COALESCE(c.parent_comment_id, c.id) ASC,
                c.created_at ASC
            "#
        )
        .bind(protocol_version_id)
        .fetch_all(pool)
        .await?;

        Ok(comments)
    }

    /// 回覆審查意見
    pub async fn reply_comment(
        pool: &PgPool,
        req: &ReplyCommentRequest,
        replied_by: Uuid,
    ) -> Result<ReviewComment> {
        // 驗證父評論存在並獲取 protocol_version_id
        let parent_comment: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT protocol_version_id 
            FROM review_comments 
            WHERE id = $1 AND parent_comment_id IS NULL
            "#
        )
        .bind(req.parent_comment_id)
        .fetch_optional(pool)
        .await?;

        let (protocol_version_id,) = parent_comment
            .ok_or_else(|| AppError::NotFound("Parent comment not found".to_string()))?;

        // 插入回覆
        let comment = sqlx::query_as::<_, ReviewComment>(
            r#"
            INSERT INTO review_comments (
                id, protocol_version_id, reviewer_id, content, 
                parent_comment_id, replied_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(protocol_version_id)
        .bind(replied_by) // 回覆者作為 reviewer_id（用於顯示）
        .bind(&req.content)
        .bind(req.parent_comment_id)
        .bind(replied_by) // replied_by 欄位
        .fetch_one(pool)
        .await?;

        Ok(comment)
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
    /// 支援委託單位主管：如果用戶是 CLIENT 角色且為主管，可查看同組織下所有用戶的計畫
    pub async fn get_my_protocols(pool: &PgPool, user_id: Uuid) -> Result<Vec<ProtocolListItem>> {
        // 檢查用戶是否有 CLIENT 角色
        let user_info: Option<(String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT 
                string_agg(DISTINCT r.code, ',') as roles,
                u.organization
            FROM users u
            LEFT JOIN user_roles ur ON u.id = ur.user_id
            LEFT JOIN roles r ON ur.role_id = r.id
            WHERE u.id = $1
            GROUP BY u.id, u.organization
            "#
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let (roles_str, organization) = user_info.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        let roles: Vec<&str> = roles_str.split(',').filter(|s| !s.is_empty()).collect();
        let has_client_role = roles.contains(&"CLIENT");

        // 如果是 CLIENT 角色且有組織，則查看同組織下所有用戶的計畫（委託單位主管權限）
        let protocols = if has_client_role && organization.is_some() {
            sqlx::query_as::<_, ProtocolListItem>(
                r#"
                SELECT DISTINCT
                    p.id, p.protocol_no, p.iacuc_no, p.title, p.status,
                    p.pi_user_id, u.display_name as pi_name, u.organization as pi_organization,
                    p.start_date, p.end_date, p.created_at,
                    NULLIF(p.working_content->'basic'->>'apply_study_number', '') as apply_study_number
                FROM protocols p
                LEFT JOIN users u ON p.pi_user_id = u.id
                WHERE (
                    -- 用戶直接相關的計畫
                    EXISTS (
                        SELECT 1 FROM user_protocols up 
                        WHERE up.protocol_id = p.id AND up.user_id = $1
                    )
                    OR
                    -- 同組織下所有用戶的計畫（委託單位主管權限）
                    (u.organization = $2 AND p.status != 'DELETED')
                )
                AND p.status != 'DELETED'
                ORDER BY p.created_at DESC
                "#
            )
            .bind(user_id)
            .bind(organization.as_deref())
            .fetch_all(pool)
            .await?
        } else {
            // 一般用戶：只查看自己相關的計畫
            sqlx::query_as::<_, ProtocolListItem>(
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
            .await?
        };

        Ok(protocols)
    }
}
