use crate::{AppError, Result};
use reqwest::multipart;

/// Gotenberg PDF 服務 HTTP Client
#[derive(Clone)]
pub struct GotenbergClient {
    base_url: String,
    client: reqwest::Client,
}

impl GotenbergClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// 將 HTML 字串轉換為 PDF bytes
    ///
    /// - `html`: 完整的 HTML 文件（包含 `<html>`, `<head>`, `<body>`）
    /// - 回傳 PDF 二進位資料
    pub async fn html_to_pdf(&self, html: &str) -> Result<Vec<u8>> {
        let url = format!("{}/forms/chromium/convert/html", self.base_url);

        let html_part = multipart::Part::bytes(html.as_bytes().to_vec())
            .file_name("index.html")
            .mime_str("text/html")
            .map_err(|e| AppError::Internal(format!("Failed to create multipart: {}", e)))?;

        // 邊距設為 0，由各模板的 CSS @page { margin } 自行控制
        let form = multipart::Form::new()
            .part("files", html_part)
            .text("paperWidth", "8.27")    // A4 寬度（英吋）
            .text("paperHeight", "11.7")   // A4 高度（英吋）
            .text("marginTop", "0")
            .text("marginBottom", "0")
            .text("marginLeft", "0")
            .text("marginRight", "0")
            .text("printBackground", "true");

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Gotenberg request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(AppError::Internal(format!(
                "Gotenberg returned {}: {}",
                status, body
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read PDF response: {}", e)))?;

        Ok(bytes.to_vec())
    }

    /// 健康檢查：確認 Gotenberg 服務可用
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
