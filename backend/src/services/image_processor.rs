use crate::{AppError, Result};
use reqwest::multipart;
use serde::{Deserialize, Serialize};

/// Image Processor 微服務 HTTP Client（方案 D）
///
/// 所有圖片解碼/處理皆委派給獨立的 image-processor 服務，
/// 後端本身不執行任何圖片解碼操作。
#[derive(Clone)]
pub struct ImageProcessorClient {
    base_url: String,
    client: reqwest::Client,
}

/// 圖片處理操作
#[derive(Debug, Clone, Serialize, Default)]
pub struct ImageOperations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resize: Option<ResizeOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
}

/// 縮放選項
#[derive(Debug, Clone, Serialize)]
pub struct ResizeOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fit: Option<String>,
}

/// 圖片處理回應
#[derive(Debug, Deserialize)]
pub struct ProcessResponse {
    pub metadata: ImageMetadata,
    pub output: ImageOutput,
}

/// 圖片元資料
#[derive(Debug, Deserialize)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub channels: u32,
    pub size: u64,
    #[serde(default)]
    pub has_alpha: bool,
}

/// 處理後的圖片輸出
#[derive(Debug, Deserialize)]
pub struct ImageOutput {
    pub data: String,
    pub info: OutputInfo,
}

/// 輸出圖片資訊
#[derive(Debug, Deserialize)]
pub struct OutputInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub size: u64,
    pub channels: u32,
}

impl ImageProcessorClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// 處理圖片：傳送原始資料至 image-processor 微服務
    ///
    /// - `data`: 原始圖片二進位資料
    /// - `mime_type`: 圖片 MIME 類型（如 `image/jpeg`）
    /// - `operations`: 處理操作（resize、format、quality）
    pub async fn process(
        &self,
        data: &[u8],
        mime_type: &str,
        operations: &ImageOperations,
    ) -> Result<ProcessResponse> {
        let url = format!("{}/process", self.base_url);

        let file_part = multipart::Part::bytes(data.to_vec())
            .file_name("image")
            .mime_str(mime_type)
            .map_err(|e| {
                AppError::Internal(format!("Failed to create multipart: {}", e))
            })?;

        let ops_json = serde_json::to_string(operations).map_err(|e| {
            AppError::Internal(format!("Failed to serialize operations: {}", e))
        })?;

        let form = multipart::Form::new()
            .part("file", file_part)
            .text("operations", ops_json);

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(format!("Image processor request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(AppError::Internal(format!(
                "Image processor returned {}: {}",
                status, body
            )));
        }

        let result: ProcessResponse =
            response.json().await.map_err(|e| {
                AppError::Internal(format!(
                    "Failed to parse image processor response: {}",
                    e
                ))
            })?;

        Ok(result)
    }

    /// 健康檢查：確認 image-processor 服務可用
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
