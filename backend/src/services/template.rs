use std::collections::HashMap;

use crate::{AppError, Result};
use tera::Tera;

/// PDF HTML 模板渲染服務
#[derive(Clone)]
pub struct TemplateService {
    tera: Tera,
}

/// 自訂 Tera filter：將數字左補零到 2 位（如 1 → "01", 12 → "12"）
fn pad2(value: &tera::Value, _args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    let n = value
        .as_i64()
        .or_else(|| value.as_u64().map(|u| u as i64))
        .unwrap_or(0);
    Ok(tera::Value::String(format!("{:02}", n)))
}

impl TemplateService {
    /// 建立空的 TemplateService（測試用，不載入任何模板）
    pub fn empty() -> Self {
        Self {
            tera: Tera::default(),
        }
    }

    /// 從 resources/templates/pdf/ 目錄載入所有模板
    pub fn new() -> Result<Self> {
        let template_dir = "resources/templates/pdf/**/*";
        let mut tera = Tera::new(template_dir).map_err(|e| {
            AppError::Internal(format!("Failed to load PDF templates: {}", e))
        })?;

        // 註冊自訂 filters
        tera.register_filter("pad2", pad2);

        tracing::info!(
            "Loaded {} PDF templates",
            tera.get_template_names().count()
        );

        Ok(Self { tera })
    }

    /// 渲染指定模板，回傳完整 HTML 字串
    ///
    /// - `template_name`: 模板檔名（如 "review_result.html"）
    /// - `context`: Tera 模板上下文（包含要填入的資料）
    pub fn render(&self, template_name: &str, context: &tera::Context) -> Result<String> {
        self.tera
            .render(template_name, context)
            .map_err(|e| AppError::Internal(format!("Template render error: {}", e)))
    }
}
