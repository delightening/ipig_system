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
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to build Gotenberg HTTP client");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
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

    /// 將 HTML 字串轉換為 PDF bytes，並附帶頁首頁尾模板
    ///
    /// - `html`: 完整的 HTML 文件
    /// - `header_html`: 頁首 HTML（支援 `<span class="pageNumber">` / `<span class="totalPages">`）
    /// - `footer_html`: 頁尾 HTML
    pub async fn html_to_pdf_with_headers(
        &self,
        html: &str,
        header_html: &str,
        footer_html: &str,
    ) -> Result<Vec<u8>> {
        let url = format!("{}/forms/chromium/convert/html", self.base_url);

        let html_part = multipart::Part::bytes(html.as_bytes().to_vec())
            .file_name("index.html")
            .mime_str("text/html")
            .map_err(|e| AppError::Internal(format!("Failed to create multipart: {}", e)))?;

        let header_part = multipart::Part::bytes(header_html.as_bytes().to_vec())
            .file_name("header.html")
            .mime_str("text/html")
            .map_err(|e| AppError::Internal(format!("Failed to create header part: {}", e)))?;

        let footer_part = multipart::Part::bytes(footer_html.as_bytes().to_vec())
            .file_name("footer.html")
            .mime_str("text/html")
            .map_err(|e| AppError::Internal(format!("Failed to create footer part: {}", e)))?;

        let form = multipart::Form::new()
            .part("files", html_part)
            .part("files", header_part)
            .part("files", footer_part)
            .text("paperWidth", "8.27")
            .text("paperHeight", "11.7")
            .text("marginTop", "15")   // mm — 為頁首留空間
            .text("marginBottom", "15") // mm — 為頁尾留空間
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
