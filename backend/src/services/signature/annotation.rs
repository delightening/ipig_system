// 記錄附註服務

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{repositories, AppError, Result};

/// 附註類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnnotationType {
    Note,       // 一般附註
    Correction, // 更正（需簽章）
    Addendum,   // 補充說明
}

impl AnnotationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnnotationType::Note => "NOTE",
            AnnotationType::Correction => "CORRECTION",
            AnnotationType::Addendum => "ADDENDUM",
        }
    }
}

/// 記錄附註
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RecordAnnotation {
    pub id: Uuid,
    pub record_type: String,
    pub record_id: i32,
    pub annotation_type: String,
    pub content: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub signature_id: Option<Uuid>,
}

/// 建立附註請求
// R26-7: API DTO（見 handlers/signature/annotation.rs）。
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CreateAnnotationRequest {
    pub content: String,
    pub annotation_type: String,
}

pub struct AnnotationService;

impl AnnotationService {
    /// 新增附註
    pub async fn create(
        pool: &PgPool,
        record_type: &str,
        record_id: i32,
        annotation_type: AnnotationType,
        content: &str,
        created_by: Uuid,
        signature_id: Option<Uuid>,
    ) -> Result<RecordAnnotation> {
        // 如果是 CORRECTION 類型，必須有簽章
        if annotation_type == AnnotationType::Correction && signature_id.is_none() {
            return Err(AppError::Validation("更正附註需要電子簽章".to_string()));
        }

        let annotation = sqlx::query_as::<_, RecordAnnotation>(
            r#"
            INSERT INTO record_annotations (
                record_type, record_id, annotation_type, content, created_by, signature_id
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(record_type)
        .bind(record_id)
        .bind(annotation_type.as_str())
        .bind(content)
        .bind(created_by)
        .bind(signature_id)
        .fetch_one(pool)
        .await?;

        Ok(annotation)
    }

    /// 取得記錄的所有附註
    pub async fn get_by_record(
        pool: &PgPool,
        record_type: &str,
        record_id: i32,
    ) -> Result<Vec<RecordAnnotation>> {
        let annotations = sqlx::query_as::<_, RecordAnnotation>(
            r#"
            SELECT * FROM record_annotations
            WHERE record_type = $1 AND record_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(record_type)
        .bind(record_id)
        .fetch_all(pool)
        .await?;

        Ok(annotations)
    }

    /// 取得附註清單並附加建立者姓名
    pub async fn enrich_with_names(
        pool: &PgPool,
        annotations: Vec<RecordAnnotation>,
    ) -> Result<Vec<(RecordAnnotation, Option<String>)>> {
        let mut result = Vec::with_capacity(annotations.len());
        for ann in annotations {
            let name =
                repositories::user::find_user_display_name_by_id(pool, ann.created_by).await?;
            result.push((ann, name));
        }
        Ok(result)
    }
}
