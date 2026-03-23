// 電子簽章 API Handlers - GLP 合規

mod sacrifice;
mod observation;
mod euthanasia;
mod transfer;
mod protocol_review;
mod annotation;
mod disposal;

pub use sacrifice::*;
pub use observation::*;
pub use euthanasia::*;
pub use transfer::*;
pub use protocol_review::*;
pub use annotation::*;
pub use disposal::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use validator::Validate;

use crate::services::{ElectronicSignature, SignatureInfoDto};

use serde_json::Value as JsonValue;

// ============================================
// Request/Response DTOs
// ============================================

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SignRecordRequest {
    /// 密碼（密碼驗證模式用）
    pub password: Option<String>,
    pub signature_type: Option<String>,
    /// 手寫簽名 SVG（手寫簽名模式用）
    pub handwriting_svg: Option<String>,
    /// 手寫簽名筆跡點資料
    pub stroke_data: Option<JsonValue>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SignRecordResponse {
    pub signature_id: Uuid,
    pub signed_at: String,
    pub is_locked: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateAnnotationRequest {
    #[validate(length(min = 1, message = "內容為必填"))]
    pub content: String,
    pub annotation_type: String,
    pub password: Option<String>, // CORRECTION 類型需要密碼
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AnnotationResponse {
    pub id: Uuid,
    pub annotation_type: String,
    pub content: String,
    pub created_by_name: Option<String>,
    pub created_at: String,
    pub has_signature: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SignatureStatusResponse {
    pub is_signed: bool,
    pub is_locked: bool,
    pub signatures: Vec<SignatureInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SignatureInfo {
    pub id: Uuid,
    pub signature_type: String,
    pub signer_name: Option<String>,
    pub signed_at: String,
    pub signature_method: Option<String>,
    pub handwriting_svg: Option<String>,
}

// ============================================
// 內部輔助函式
// ============================================

/// 將 SignatureInfoDto 轉換為 handler 層的 SignatureInfo
pub(crate) fn to_signature_infos(dtos: Vec<SignatureInfoDto>) -> Vec<SignatureInfo> {
    dtos.into_iter()
        .map(|dto| SignatureInfo {
            id: dto.id,
            signature_type: dto.signature_type,
            signer_name: dto.signer_name,
            signed_at: dto.signed_at.to_rfc3339(),
            signature_method: dto.signature_method,
            handwriting_svg: dto.handwriting_svg,
        })
        .collect()
}

/// 建構簽章回應
pub(crate) fn sign_response(sig: &ElectronicSignature, is_locked: bool) -> SignRecordResponse {
    SignRecordResponse {
        signature_id: sig.id,
        signed_at: sig.signed_at.to_rfc3339(),
        is_locked,
    }
}
