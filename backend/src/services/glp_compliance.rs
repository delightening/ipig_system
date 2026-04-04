// Service 層：GLP 合規模組業務邏輯

use crate::error::AppError;
use crate::models::glp_compliance::*;
use crate::repositories::glp_compliance as repo;
use crate::Result;
use sqlx::PgPool;
use uuid::Uuid;

pub struct GlpComplianceService;

impl GlpComplianceService {
    // ========================================================================
    // Reference Standards
    // ========================================================================

    pub async fn list_reference_standards(pool: &PgPool) -> Result<Vec<ReferenceStandard>> {
        repo::find_reference_standards(pool).await
    }

    pub async fn get_reference_standard(pool: &PgPool, id: Uuid) -> Result<ReferenceStandard> {
        repo::find_reference_standard_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("參考標準器不存在".into()))
    }

    pub async fn create_reference_standard(pool: &PgPool, req: &CreateReferenceStandardRequest, user_id: Uuid) -> Result<ReferenceStandard> {
        repo::insert_reference_standard(pool, req, user_id).await
    }

    pub async fn update_reference_standard(pool: &PgPool, id: Uuid, req: &UpdateReferenceStandardRequest) -> Result<ReferenceStandard> {
        repo::find_reference_standard_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("參考標準器不存在".into()))?;
        repo::update_reference_standard(pool, id, req).await
    }

    // ========================================================================
    // Controlled Documents
    // ========================================================================

    pub async fn list_controlled_documents(pool: &PgPool, params: &ControlledDocumentQuery) -> Result<Vec<ControlledDocumentWithOwner>> {
        repo::find_controlled_documents(pool, params).await
    }

    pub async fn get_controlled_document(pool: &PgPool, id: Uuid) -> Result<ControlledDocumentWithOwner> {
        repo::find_controlled_document_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("受控文件不存在".into()))
    }

    pub async fn create_controlled_document(pool: &PgPool, req: &CreateControlledDocumentRequest, owner_id: Uuid) -> Result<ControlledDocument> {
        let number = Self::generate_doc_number(pool, &req.doc_type).await?;
        repo::insert_controlled_document(pool, &number, req, owner_id).await
    }

    pub async fn update_controlled_document(pool: &PgPool, id: Uuid, req: &UpdateControlledDocumentRequest) -> Result<ControlledDocument> {
        repo::find_controlled_document_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("受控文件不存在".into()))?;
        repo::update_controlled_document(pool, id, req).await
    }

    pub async fn approve_controlled_document(pool: &PgPool, id: Uuid, approver_id: Uuid) -> Result<ControlledDocument> {
        let doc = repo::find_controlled_document_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("受控文件不存在".into()))?;
        if doc.status != "under_review" && doc.status != "draft" {
            return Err(AppError::BusinessRule("只有草稿或審查中的文件可以核准".into()));
        }
        repo::approve_controlled_document(pool, id, approver_id).await
    }

    pub async fn get_document_revisions(pool: &PgPool, document_id: Uuid) -> Result<Vec<DocumentRevision>> {
        repo::find_document_revisions(pool, document_id).await
    }

    pub async fn create_revision(pool: &PgPool, document_id: Uuid, req: &CreateRevisionRequest, user_id: Uuid) -> Result<DocumentRevision> {
        let doc = repo::find_controlled_document_by_id(pool, document_id)
            .await?
            .ok_or(AppError::NotFound("受控文件不存在".into()))?;
        let new_version = doc.current_version + 1;

        // 更新主表版本號
        let update_req = UpdateControlledDocumentRequest {
            title: None, category: None, status: Some("draft".to_string()),
            effective_date: None, review_due_date: None, retention_years: None, file_path: None,
        };
        repo::update_controlled_document(pool, document_id, &update_req).await?;

        // 更新 current_version
        sqlx::query("UPDATE controlled_documents SET current_version = $2, updated_at = NOW() WHERE id = $1")
            .bind(document_id)
            .bind(new_version)
            .execute(pool)
            .await?;

        repo::insert_document_revision(pool, document_id, new_version, req, user_id).await
    }

    pub async fn acknowledge_document(pool: &PgPool, document_id: Uuid, user_id: Uuid) -> Result<DocumentAcknowledgment> {
        let doc = repo::find_controlled_document_by_id(pool, document_id)
            .await?
            .ok_or(AppError::NotFound("受控文件不存在".into()))?;
        repo::insert_document_acknowledgment(pool, document_id, user_id, doc.current_version).await
    }

    async fn generate_doc_number(pool: &PgPool, doc_type: &str) -> Result<String> {
        let prefix = match doc_type {
            "quality_manual" => "QM",
            "sop" => "SOP",
            "form" => "FM",
            "external" => "EXT",
            "policy" => "POL",
            "report" => "RPT",
            _ => "DOC",
        };
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM controlled_documents WHERE doc_type = $1 AND EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())"
        )
        .bind(doc_type)
        .fetch_one(pool)
        .await?;
        Ok(format!("{prefix}-{year}-{:04}", count.0 + 1))
    }

    // ========================================================================
    // Management Reviews
    // ========================================================================

    pub async fn list_management_reviews(pool: &PgPool, params: &ManagementReviewQuery) -> Result<Vec<ManagementReview>> {
        repo::find_management_reviews(pool, params).await
    }

    pub async fn get_management_review(pool: &PgPool, id: Uuid) -> Result<ManagementReview> {
        repo::find_management_review_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("管理審查不存在".into()))
    }

    pub async fn create_management_review(pool: &PgPool, req: &CreateManagementReviewRequest) -> Result<ManagementReview> {
        let number = Self::generate_review_number(pool).await?;
        repo::insert_management_review(pool, &number, req).await
    }

    pub async fn update_management_review(pool: &PgPool, id: Uuid, req: &UpdateManagementReviewRequest) -> Result<ManagementReview> {
        repo::find_management_review_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("管理審查不存在".into()))?;
        repo::update_management_review(pool, id, req).await
    }

    async fn generate_review_number(pool: &PgPool) -> Result<String> {
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM management_reviews WHERE EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())"
        )
        .fetch_one(pool)
        .await?;
        Ok(format!("MR-{year}-{:04}", count.0 + 1))
    }

    // ========================================================================
    // Risk Register
    // ========================================================================

    pub async fn list_risks(pool: &PgPool, params: &RiskQuery) -> Result<Vec<RiskEntryWithOwner>> {
        repo::find_risks(pool, params).await
    }

    pub async fn get_risk(pool: &PgPool, id: Uuid) -> Result<RiskEntryWithOwner> {
        repo::find_risk_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("風險項目不存在".into()))
    }

    pub async fn create_risk(pool: &PgPool, req: &CreateRiskRequest) -> Result<RiskEntry> {
        let number = Self::generate_risk_number(pool).await?;
        repo::insert_risk(pool, &number, req).await
    }

    pub async fn update_risk(pool: &PgPool, id: Uuid, req: &UpdateRiskRequest) -> Result<RiskEntry> {
        repo::find_risk_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("風險項目不存在".into()))?;
        repo::update_risk(pool, id, req).await
    }

    async fn generate_risk_number(pool: &PgPool) -> Result<String> {
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM risk_register WHERE EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())"
        )
        .fetch_one(pool)
        .await?;
        Ok(format!("RISK-{year}-{:04}", count.0 + 1))
    }

    // ========================================================================
    // Change Requests
    // ========================================================================

    pub async fn list_change_requests(pool: &PgPool, params: &ChangeRequestQuery) -> Result<Vec<ChangeRequestWithNames>> {
        repo::find_change_requests(pool, params).await
    }

    pub async fn get_change_request(pool: &PgPool, id: Uuid) -> Result<ChangeRequest> {
        repo::find_change_request_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("變更申請不存在".into()))
    }

    pub async fn create_change_request(pool: &PgPool, req: &CreateChangeRequestRequest, user_id: Uuid) -> Result<ChangeRequest> {
        let number = Self::generate_change_number(pool).await?;
        repo::insert_change_request(pool, &number, req, user_id).await
    }

    pub async fn update_change_request(pool: &PgPool, id: Uuid, req: &UpdateChangeRequestRequest) -> Result<ChangeRequest> {
        repo::find_change_request_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("變更申請不存在".into()))?;
        repo::update_change_request(pool, id, req).await
    }

    pub async fn approve_change_request(pool: &PgPool, id: Uuid, approver_id: Uuid) -> Result<ChangeRequest> {
        let cr = repo::find_change_request_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("變更申請不存在".into()))?;
        if cr.status != "submitted" && cr.status != "under_review" {
            return Err(AppError::BusinessRule("只有已提交或審查中的變更申請可以核准".into()));
        }
        repo::approve_change_request(pool, id, approver_id).await
    }

    async fn generate_change_number(pool: &PgPool) -> Result<String> {
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM change_requests WHERE EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())"
        )
        .fetch_one(pool)
        .await?;
        Ok(format!("CR-{year}-{:04}", count.0 + 1))
    }

    // ========================================================================
    // Environment Monitoring
    // ========================================================================

    pub async fn list_monitoring_points(pool: &PgPool, active_only: bool) -> Result<Vec<EnvironmentMonitoringPoint>> {
        repo::find_monitoring_points(pool, active_only).await
    }

    pub async fn get_monitoring_point(pool: &PgPool, id: Uuid) -> Result<EnvironmentMonitoringPoint> {
        repo::find_monitoring_point_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("監控點不存在".into()))
    }

    pub async fn create_monitoring_point(pool: &PgPool, req: &CreateMonitoringPointRequest) -> Result<EnvironmentMonitoringPoint> {
        repo::insert_monitoring_point(pool, req).await
    }

    pub async fn update_monitoring_point(pool: &PgPool, id: Uuid, req: &UpdateMonitoringPointRequest) -> Result<EnvironmentMonitoringPoint> {
        repo::find_monitoring_point_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("監控點不存在".into()))?;
        repo::update_monitoring_point(pool, id, req).await
    }

    pub async fn list_readings(pool: &PgPool, params: &ReadingQuery) -> Result<Vec<EnvironmentReading>> {
        repo::find_readings(pool, params).await
    }

    pub async fn create_reading(pool: &PgPool, req: &CreateReadingRequest, user_id: Uuid) -> Result<EnvironmentReading> {
        let point = repo::find_monitoring_point_by_id(pool, req.monitoring_point_id)
            .await?
            .ok_or(AppError::NotFound("監控點不存在".into()))?;

        // 檢查是否超出範圍
        let (is_oor, oor_params) = Self::check_out_of_range(&point.parameters, &req.readings);
        repo::insert_reading(pool, req, user_id, is_oor, oor_params).await
    }

    fn check_out_of_range(parameters: &serde_json::Value, readings: &serde_json::Value) -> (bool, Option<serde_json::Value>) {
        let mut oor_list: Vec<String> = Vec::new();

        if let (Some(params_arr), Some(readings_obj)) = (parameters.as_array(), readings.as_object()) {
            for param in params_arr {
                let name = param.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let min = param.get("min").and_then(|v| v.as_f64());
                let max = param.get("max").and_then(|v| v.as_f64());
                let value = readings_obj.get(name).and_then(|v| v.as_f64());

                if let Some(val) = value {
                    if let Some(mn) = min {
                        if val < mn { oor_list.push(name.to_string()); continue; }
                    }
                    if let Some(mx) = max {
                        if val > mx { oor_list.push(name.to_string()); }
                    }
                }
            }
        }

        if oor_list.is_empty() {
            (false, None)
        } else {
            (true, Some(serde_json::json!(oor_list)))
        }
    }

    // ========================================================================
    // Competency Assessments
    // ========================================================================

    pub async fn list_competency_assessments(pool: &PgPool, params: &CompetencyQuery) -> Result<Vec<CompetencyAssessmentWithNames>> {
        repo::find_competency_assessments(pool, params).await
    }

    pub async fn create_competency_assessment(pool: &PgPool, req: &CreateCompetencyRequest, assessor_id: Uuid) -> Result<CompetencyAssessment> {
        repo::insert_competency_assessment(pool, req, assessor_id).await
    }

    pub async fn update_competency_assessment(pool: &PgPool, id: Uuid, req: &UpdateCompetencyRequest) -> Result<CompetencyAssessment> {
        repo::update_competency_assessment(pool, id, req).await
    }

    pub async fn list_training_requirements(pool: &PgPool, role_code: Option<&str>) -> Result<Vec<RoleTrainingRequirement>> {
        repo::find_training_requirements(pool, role_code).await
    }

    pub async fn create_training_requirement(pool: &PgPool, req: &CreateTrainingRequirementRequest) -> Result<RoleTrainingRequirement> {
        repo::insert_training_requirement(pool, req).await
    }

    pub async fn delete_training_requirement(pool: &PgPool, id: Uuid) -> Result<()> {
        repo::delete_training_requirement(pool, id).await
    }

    // ========================================================================
    // Study Final Reports
    // ========================================================================

    pub async fn list_study_reports(pool: &PgPool, params: &StudyReportQuery) -> Result<Vec<StudyFinalReport>> {
        repo::find_study_reports(pool, params).await
    }

    pub async fn get_study_report(pool: &PgPool, id: Uuid) -> Result<StudyFinalReport> {
        repo::find_study_report_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("最終報告不存在".into()))
    }

    pub async fn create_study_report(pool: &PgPool, req: &CreateStudyReportRequest) -> Result<StudyFinalReport> {
        let number = Self::generate_report_number(pool).await?;
        repo::insert_study_report(pool, &number, req).await
    }

    pub async fn update_study_report(pool: &PgPool, id: Uuid, req: &UpdateStudyReportRequest) -> Result<StudyFinalReport> {
        repo::find_study_report_by_id(pool, id)
            .await?
            .ok_or(AppError::NotFound("最終報告不存在".into()))?;
        repo::update_study_report(pool, id, req).await
    }

    async fn generate_report_number(pool: &PgPool) -> Result<String> {
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM study_final_reports WHERE EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())"
        )
        .fetch_one(pool)
        .await?;
        Ok(format!("SFR-{year}-{:04}", count.0 + 1))
    }

    // ========================================================================
    // Formulation Records
    // ========================================================================

    pub async fn list_formulation_records(pool: &PgPool, params: &FormulationQuery) -> Result<Vec<FormulationRecordWithNames>> {
        repo::find_formulation_records(pool, params).await
    }

    pub async fn create_formulation_record(pool: &PgPool, req: &CreateFormulationRequest, user_id: Uuid) -> Result<FormulationRecord> {
        repo::insert_formulation_record(pool, req, user_id).await
    }
}
