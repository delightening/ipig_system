use crate::{AppError, Result};

/// 快取的 render 結果：`(PDF bytes, X-PDF-Renderer header)`。以 `Arc` 包裹，使 cache hit
/// 為廉價的指標 clone，而非複製整份 PDF bytes。
type CachedRender = std::sync::Arc<(Vec<u8>, Option<String>)>;

/// PDF Service (print-pdf, FastAPI + WeasyPrint) HTTP Client
///
/// 呼叫 Python 端 FastAPI `print-pdf` 微服務，由其使用 Jinja2 HTML 模板
/// 透過 WeasyPrint 直接 render 為 PDF（取代舊三件式 pdf-service + gotenberg
/// + word-convert daemon stack）。
#[derive(Clone)]
pub struct PdfServiceClient {
    base_url: String,
    token: String,
    client: reqwest::Client,
    /// AUP 計畫書 PDF render 快取（見 `constants::AUP_PDF_CACHE_*`）。
    /// key = `sha256(送進 print-pdf 的 body bytes)`、value = `Arc<(PDF bytes, X-PDF-Renderer)>`。
    /// moka Cache 內部為 `Arc`，故 clone `PdfServiceClient` 不複製快取內容。
    aup_pdf_cache: moka::future::Cache<String, CachedRender>,
}

impl PdfServiceClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to build PdfService HTTP client");
        // 容量以位元組計（weigher = PDF bytes 長度），避免少數大 PDF 撐爆 RAM。
        let aup_pdf_cache = moka::future::Cache::builder()
            .weigher(|_k: &String, v: &CachedRender| v.0.len().try_into().unwrap_or(u32::MAX))
            .max_capacity(crate::constants::AUP_PDF_CACHE_MAX_BYTES)
            .time_to_live(std::time::Duration::from_secs(
                crate::constants::AUP_PDF_CACHE_TTL_SECS,
            ))
            .build();
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            client,
            aup_pdf_cache,
        }
    }

    /// R32-A3 收尾：呼叫 `POST /render-aup/from-working-content?format={docx|pdf}`。
    pub async fn render_aup_from_working_content(
        &self,
        working_content: &serde_json::Value,
        format: DocxRenderFormat,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!(
            "{}/render-aup/from-working-content?format={}",
            self.base_url,
            format.as_str()
        );
        let body = serde_json::json!({"working_content": working_content});

        // 快取僅針對 PDF：「計畫內容」分頁預覽每次都觸發 WeasyPrint 全量 render（~15s/份）
        // 且 render 序列化（cap=1）；同一份未修改計畫書反覆預覽是純重算浪費。key 綁「實際送出的
        // body」（含已內嵌照片）→ 同內容必同 PDF，內容一改即 miss 重算，無 staleness。
        let cache_key = match format {
            DocxRenderFormat::Pdf => render_cache_key("render-aup-pdf", &body),
            _ => None,
        };

        if let Some(key) = &cache_key {
            if let Some(hit) = self.aup_pdf_cache.get(key).await {
                metrics::counter!("ipig_aup_pdf_cache_requests_total", "result" => "hit")
                    .increment(1);
                return Ok((*hit).clone());
            }
        }

        let (bytes, renderer) = self.post_binary(&url, &body, "render-aup").await?;

        if let Some(key) = cache_key {
            metrics::counter!("ipig_aup_pdf_cache_requests_total", "result" => "miss").increment(1);
            self.aup_pdf_cache
                .insert(key, std::sync::Arc::new((bytes.clone(), renderer.clone())))
                .await;
        }
        Ok((bytes, renderer))
    }

    /// 渲染 AUP 計畫書為 HTML（供前端「計畫內容」預覽 iframe）。
    ///
    /// 與 `render_aup_from_working_content` 走同一支 endpoint、同一份 Jinja2 模板，
    /// 差別僅在 `?format=html` 回傳 PDF 化前的 HTML，故預覽與匯出 PDF 必然一致。
    pub async fn render_aup_html(&self, working_content: &serde_json::Value) -> Result<String> {
        let url = format!(
            "{}/render-aup/from-working-content?format=html",
            self.base_url
        );
        let body = serde_json::json!({"working_content": working_content});
        let (bytes, _renderer) = self.post_binary(&url, &body, "render-aup-html").await?;
        String::from_utf8(bytes).map_err(|e| AppError::Internal(format!("AUP HTML 非 UTF-8: {e}")))
    }

    /// R32-A8a：呼叫 `POST /render-medical-record/from-animal-data?format={docx|pdf}`。
    ///
    /// `data` 直接傳 `AnimalMedicalService::get_animal_medical_data` 的 JSON 結果
    /// （含 animal/observations/surgeries/weights/vaccinations/sacrifice），由
    /// pdf-service 內部 adapter 翻譯 enum、合併 timeline、格式化日期 / 體重後 render。
    pub async fn render_medical_record_from_animal_data(
        &self,
        data: &serde_json::Value,
        format: DocxRenderFormat,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!(
            "{}/render-medical-record/from-animal-data?format={}",
            self.base_url,
            format.as_str()
        );
        self.post_binary(&url, data, "render-medical-record").await
    }

    /// R32-A8d：呼叫 `POST /render-project-medical/from-project-data`，回 N 動物
    /// 合併後的單一 PDF（pypdf merge）。
    ///
    /// `data` 結構：`{"iacuc_no": "...", "animals": [<get_animal_medical_data>, ...]}`
    pub async fn render_project_medical_from_project_data(
        &self,
        data: &serde_json::Value,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!("{}/render-project-medical/from-project-data", self.base_url);
        self.post_binary(&url, data, "render-project-medical").await
    }

    /// R32-A8e：呼叫 `POST /render-review-reply/from-review-data?format={docx|pdf}`。
    pub async fn render_review_reply_from_review_data(
        &self,
        data: &serde_json::Value,
        format: DocxRenderFormat,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!(
            "{}/render-review-reply/from-review-data?format={}",
            self.base_url,
            format.as_str()
        );
        self.post_binary(&url, data, "render-review-reply").await
    }

    /// R32-A8c：呼叫 `POST /render-review-result/from-review-data?format={docx|pdf}`。
    pub async fn render_review_result_from_review_data(
        &self,
        data: &serde_json::Value,
        format: DocxRenderFormat,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!(
            "{}/render-review-result/from-review-data?format={}",
            self.base_url,
            format.as_str()
        );
        self.post_binary(&url, data, "render-review-result").await
    }

    /// R32-A8b：呼叫 `POST /render-surgery/from-surgery-data?format={docx|pdf}`。
    ///
    /// `data` 直接傳 `AnimalSurgeryService::get_surgery_export_data` 的 JSON
    /// 結果（含 surgery / animal / source_name / recorded_by_name /
    /// pain_assessments），pdf-service 端會展開 JSONB 藥物 / 翻譯 enum 後 render。
    pub async fn render_surgery_from_surgery_data(
        &self,
        data: &serde_json::Value,
        format: DocxRenderFormat,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!(
            "{}/render-surgery/from-surgery-data?format={}",
            self.base_url,
            format.as_str()
        );
        self.post_binary(&url, data, "render-surgery").await
    }

    /// R32-A8h：呼叫 `POST /render-blood-test/from-blood-test-data?format={docx|pdf}`。
    ///
    /// `data` 為扁平 payload：`{animal_ear_tag, animal_iacuc_no, export_date, tests[]}`。
    /// 取代 legacy `render("blood_test", ...)` (Jinja2 HTML registry) 路徑。
    pub async fn render_blood_test_from_blood_test_data(
        &self,
        data: &serde_json::Value,
        format: DocxRenderFormat,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!(
            "{}/render-blood-test/from-blood-test-data?format={}",
            self.base_url,
            format.as_str()
        );
        self.post_binary(&url, data, "render-blood-test").await
    }

    /// R32-A8i：呼叫 `POST /render-audit-log/from-export-data?format={docx|pdf}`。
    ///
    /// `data` 為 backend handler 組好的扁平 payload：`{meta, summary, entries[],
    /// signature}`。取代 legacy frontend client-side HTML + `window.print()` 路徑。
    pub async fn render_audit_log_from_export_data(
        &self,
        data: &serde_json::Value,
        format: DocxRenderFormat,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!(
            "{}/render-audit-log/from-export-data?format={}",
            self.base_url,
            format.as_str()
        );
        self.post_binary(&url, data, "render-audit-log").await
    }

    /// R32-A8g：呼叫 `POST /render-warehouse/from-report-data?format={docx|pdf}`。
    ///
    /// `data` 直接傳 `WarehouseService::get_report_data` 的 `WarehouseReportData`
    /// JSON serialize，pdf-service adapter 把 `inventory[]` 攤平成 `inventory_summary`
    /// 後 render。取代 legacy `PdfService::generate_warehouse_report` (printpdf) 路徑。
    pub async fn render_warehouse_from_report_data(
        &self,
        data: &serde_json::Value,
        format: DocxRenderFormat,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!(
            "{}/render-warehouse/from-report-data?format={}",
            self.base_url,
            format.as_str()
        );
        self.post_binary(&url, data, "render-warehouse").await
    }

    /// R39：呼叫 `POST /render-vet-patrol-report/from-report-data?format={docx|pdf}`。
    ///
    /// `data` 對齊 pdf-service `vet_patrol_report` adapter，含 categories[]
    /// 與 photos data URLs。取代 legacy `vet_patrol_report.html` + Gotenberg
    /// HTML→PDF 路徑。
    pub async fn render_vet_patrol_report_from_report_data(
        &self,
        data: &serde_json::Value,
        format: DocxRenderFormat,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!(
            "{}/render-vet-patrol-report/from-report-data?format={}",
            self.base_url,
            format.as_str()
        );
        self.post_binary(&url, data, "render-vet-patrol-report")
            .await
    }

    /// R32-A3b 收尾：呼叫 `POST /render-vet-patrol/from-animals?format={xlsx|pdf}`。
    ///
    /// L2 (2026-05-12)：除 bytes 外回傳 `X-PDF-Renderer`（如 `excel_daemon`、
    /// `gotenberg_fallback`）。handler 應將其貼到對外 response header，讓前端
    /// 在降級時提示使用者。
    pub async fn render_vet_patrol_from_animals(
        &self,
        animals: &serde_json::Value,
        inspector_name: &str,
        patrol_date: &str,
        period: &str,
        format: XlsxRenderFormat,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!(
            "{}/render-vet-patrol/from-animals?format={}",
            self.base_url,
            format.as_str()
        );
        let body = serde_json::json!({
            "animals": animals,
            "inspector_name": inspector_name,
            "patrol_date": patrol_date,
            "period": period,
        });
        self.post_binary(&url, &body, "render-vet-patrol").await
    }

    /// R53-10: 週報 xlsx 匯出。
    pub async fn render_weekly_medical_report_xlsx(
        &self,
        events: &serde_json::Value,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!("{}/render-xlsx/weekly-medical-report", self.base_url);
        let body = serde_json::json!({ "events": events });
        self.post_binary(&url, &body, "render-weekly-medical-xlsx")
            .await
    }

    /// R53-11: 週報 PDF 匯出。
    pub async fn render_weekly_medical_report_pdf(
        &self,
        events: &serde_json::Value,
        date_range: &str,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!("{}/render-pdf/weekly-medical-report", self.base_url);
        let body = serde_json::json!({ "events": events, "date_range": date_range });
        self.post_binary(&url, &body, "render-weekly-medical-pdf")
            .await
    }

    /// R53-15: 月結 byproduct xlsx 匯出。
    pub async fn render_byproduct_monthly_xlsx(
        &self,
        rows: &serde_json::Value,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let url = format!("{}/render-xlsx/byproduct-monthly", self.base_url);
        let body = serde_json::json!({ "rows": rows });
        self.post_binary(&url, &body, "render-byproduct-monthly-xlsx")
            .await
    }

    /// L2 (2026-05-12)：除 PDF/docx/xlsx bytes 外，回傳 pdf-service `X-PDF-Renderer`
    /// header 值（記錄實際用的渲染器：`excel_daemon` / `word_daemon` /
    /// `gotenberg_fallback` / `gotenberg_only`）。handler 應將其貼到對外 response
    /// header，前端在降級時 toast 提示使用者。
    async fn post_binary(
        &self,
        url: &str,
        body: &serde_json::Value,
        op: &str,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let response = self
            .client
            .post(url)
            .header("X-Internal-Token", &self.token)
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("PDF service {} request failed: {}", op, e)))?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(AppError::Internal(format!(
                "PDF service {} returned {}: {}",
                op, status, text
            )));
        }
        let renderer = response
            .headers()
            .get("x-pdf-renderer")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read {} response: {}", op, e)))?;
        Ok((bytes.to_vec(), renderer))
    }

    /// 健康檢查：確認 PDF service 可用
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// 探測 print-pdf 服務存活（`/pdf-service-health`），回傳原始 JSON + 是否就緒。
    /// frontend 用來判斷能否允許 GLP 匯出。
    pub async fn liveness(&self) -> Result<(serde_json::Value, bool)> {
        let url = format!("{}/pdf-service-health", self.base_url);
        let resp =
            self.client.get(&url).send().await.map_err(|e| {
                AppError::Internal(format!("pdf-service liveness unreachable: {e}"))
            })?;
        let status_ok = resp.status() == 200;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("pdf-service liveness parse: {e}")))?;
        Ok((body, status_ok))
    }
}

/// 計算 render 快取 key：`sha256(tag \0 body_bytes)` 的 hex 字串。
///
/// 序列化失敗（in-memory `Value` 理論上不會發生）→ 回 `None`，呼叫端據此略過快取
/// 直接 render，避免靜默產生可能碰撞的退化 key。
fn render_cache_key(tag: &str, body: &serde_json::Value) -> Option<String> {
    use sha2::{Digest, Sha256};
    let body_bytes = serde_json::to_vec(body).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(tag.as_bytes());
    hasher.update([0u8]);
    hasher.update(&body_bytes);
    Some(hex::encode(hasher.finalize()))
}

/// R32-A4: docx render 回傳格式選擇。
///
/// 對應 pdf-service `/render-docx/{doc_type}?format={docx|pdf}` query 參數。
#[derive(Debug, Clone, Copy)]
pub enum DocxRenderFormat {
    /// docxtpl fill 後的原始 .docx（OOXML 格式，使用者可在 Word 編輯）
    Docx,
    /// docx 經 Gotenberg LibreOffice 轉換後的 PDF（GLP 報表正式輸出）
    Pdf,
}

impl DocxRenderFormat {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Pdf => "pdf",
        }
    }

    /// 對應的 MIME type，handler 寫到 response Content-Type header。
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Pdf => "application/pdf",
        }
    }

    /// 對應的副檔名，handler 寫到 Content-Disposition filename 用。
    pub fn extension(&self) -> &'static str {
        self.as_str()
    }
}

/// R32-A3b: xlsx render 回傳格式選擇。
#[derive(Debug, Clone, Copy)]
pub enum XlsxRenderFormat {
    Xlsx,
    Pdf,
}

impl XlsxRenderFormat {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Xlsx => "xlsx",
            Self::Pdf => "pdf",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Pdf => "application/pdf",
        }
    }

    pub fn extension(&self) -> &'static str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::render_cache_key;
    use serde_json::json;

    #[test]
    fn cache_key_is_deterministic_for_same_body() {
        let body = json!({"working_content": {"basic": {"title": "X"}, "n": 1}});
        let a = render_cache_key("render-aup-pdf", &body);
        let b = render_cache_key("render-aup-pdf", &body);
        assert!(a.is_some());
        assert_eq!(a, b, "同一份 body 必須得到相同 key（cache 命中前提）");
    }

    #[test]
    fn cache_key_changes_when_body_changes() {
        // 內容一改 → key 改 → 自動 miss 重算（無 staleness）。
        let k1 = render_cache_key(
            "render-aup-pdf",
            &json!({"working_content": {"title": "A"}}),
        );
        let k2 = render_cache_key(
            "render-aup-pdf",
            &json!({"working_content": {"title": "B"}}),
        );
        assert_ne!(k1, k2, "body 不同時 key 必須不同，否則會回舊 PDF");
    }

    #[test]
    fn cache_key_namespaced_by_tag() {
        // tag 隔離不同 render 種類，避免未來共用同一 cache 時跨類碰撞。
        let body = json!({"working_content": {"title": "X"}});
        assert_ne!(
            render_cache_key("render-aup-pdf", &body),
            render_cache_key("other-doc", &body)
        );
    }

    #[test]
    fn cache_key_is_hex_sha256() {
        let key =
            render_cache_key("render-aup-pdf", &json!({"a": 1})).expect("可序列化 body 應得到 key");
        assert_eq!(key.len(), 64, "sha256 hex 應為 64 字元");
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
