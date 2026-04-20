use crate::{AppError, Result};
use serde::Serialize;

/// PDF Service (FastAPI) HTTP Client
///
/// 呼叫 Python 端 FastAPI `pdf-service` 微服務，讓其使用 Jinja2 模板
/// 產生 HTML 後轉交 Gotenberg 渲染為 PDF。
///
/// 與 `GotenbergClient` 的差別：
/// - 這個 client 呼叫的是 FastAPI 抽象層（`/render/{doc_type}`）
/// - Rust 只需送 JSON payload，模板與資料組裝皆由 Python 負責
#[derive(Clone)]
pub struct PdfServiceClient {
    base_url: String,
    token: String,
    client: reqwest::Client,
}

impl PdfServiceClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to build PdfService HTTP client");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            client,
        }
    }

    /// 呼叫 `POST /render/{doc_type}` 並回傳 PDF bytes。
    pub async fn render<T: Serialize + ?Sized>(
        &self,
        doc_type: &str,
        payload: &T,
    ) -> Result<Vec<u8>> {
        let url = format!("{}/render/{}", self.base_url, doc_type);
        let response = self
            .client
            .post(&url)
            .header("X-Internal-Token", &self.token)
            .json(payload)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("PDF service request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(AppError::Internal(format!(
                "PDF service returned {}: {}",
                status, body
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read PDF response: {}", e)))?;
        Ok(bytes.to_vec())
    }

    /// 健康檢查：確認 PDF service 可用
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
