use std::collections::HashMap;

use lopdf::Document;

/// 從 PDF bytes 中搜尋章節標記文字，回傳 marker → 頁碼 的對應表。
///
/// 標記格式為 `§SECn§`（n = 1..8），嵌入在 HTML 模板中作為不可見文字。
/// 頁碼從 1 開始計算。
pub fn find_section_pages(pdf_bytes: &[u8]) -> HashMap<String, usize> {
    let mut result = HashMap::new();

    let doc = match Document::load_mem(pdf_bytes) {
        Ok(d) => d,
        Err(_) => return result,
    };

    let pages = doc.get_pages();
    let mut sorted_pages: Vec<(u32, _)> = pages.into_iter().collect();
    sorted_pages.sort_by_key(|(num, _)| *num);

    for (page_num, page_id) in &sorted_pages {
        let text = extract_page_text(&doc, *page_id);

        for i in 1..=8 {
            let marker = format!("§SEC{}§", i);
            if text.contains(&marker) && !result.contains_key(&marker) {
                result.insert(marker, *page_num as usize);
            }
        }
    }

    result
}

/// 從 PDF 取得總頁數
pub fn get_total_pages(pdf_bytes: &[u8]) -> usize {
    match Document::load_mem(pdf_bytes) {
        Ok(doc) => doc.get_pages().len(),
        Err(_) => 0,
    }
}

/// 簡易文字提取：遍歷 page 的 content stream，
/// 收集 Tj / TJ 運算子中的文字片段。
fn extract_page_text(doc: &Document, page_id: lopdf::ObjectId) -> String {
    let mut text = String::new();

    let content_data = match doc.get_page_content(page_id) {
        Ok(data) => data,
        Err(_) => return text,
    };

    let content = match lopdf::content::Content::decode(&content_data) {
        Ok(c) => c,
        Err(_) => return text,
    };

    for operation in &content.operations {
        match operation.operator.as_str() {
            "Tj" => {
                for operand in &operation.operands {
                    if let Ok(s) = operand.as_string() {
                        text.push_str(&String::from_utf8_lossy(s.as_bytes()));
                    }
                }
            }
            "TJ" => {
                if let Ok(arr) = operation.operands.first().map(|o| o.as_array()).transpose() {
                    if let Some(arr) = arr {
                        for item in arr {
                            if let Ok(s) = item.as_string() {
                                text.push_str(&String::from_utf8_lossy(s.as_bytes()));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    text
}
