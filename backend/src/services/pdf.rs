// PDF 生成服務
// 使用 printpdf 函式庫生成 AUP 計畫書 PDF

use printpdf::*;
use crate::models::ProtocolResponse;
use crate::{AppError, Result};

/// PDF 頁面配置常數
const PAGE_WIDTH_MM: f32 = 210.0;  // A4 寬度
const PAGE_HEIGHT_MM: f32 = 297.0; // A4 高度
const MARGIN_MM: f32 = 20.0;
const LINE_HEIGHT_MM: f32 = 6.0;
const SECTION_SPACING_MM: f32 = 10.0;

/// PDF 生成服務
pub struct PdfService;

impl PdfService {
    /// 生成 AUP 計畫書 PDF
    pub fn generate_protocol_pdf(protocol: &ProtocolResponse) -> Result<Vec<u8>> {
        // 建立 PDF 文件
        let (doc, page1, layer1) = PdfDocument::new(
            "AUP 動物試驗計畫書",
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            "第1頁"
        );

        // 載入中文字型
        let font_path = std::path::Path::new("resources/fonts/NotoSansSC-Regular.ttf");
        if !font_path.exists() {
            return Err(AppError::Internal(
                "Font file not found: resources/fonts/NotoSansSC-Regular.ttf".to_string()
            ));
        }
        
        let font_bytes = std::fs::read(font_path)
            .map_err(|e| AppError::Internal(format!("Failed to read font file: {}", e)))?;
        
        let font = doc.add_external_font(&*font_bytes)
            .map_err(|e| AppError::Internal(format!("Failed to load font: {}", e)))?;

        // 開始渲染 PDF
        let current_layer = doc.get_page(page1).get_layer(layer1);
        let mut y_position = PAGE_HEIGHT_MM - MARGIN_MM;

        // ========== 標題 ==========
        current_layer.use_text(
            "AUP 動物試驗計畫書",
            24.0,
            Mm(PAGE_WIDTH_MM / 2.0 - 40.0),
            Mm(y_position),
            &font
        );
        y_position -= 12.0;

        // 計畫標題
        current_layer.use_text(
            &protocol.protocol.title,
            14.0,
            Mm(MARGIN_MM),
            Mm(y_position),
            &font
        );
        y_position -= SECTION_SPACING_MM * 2.0;

        // ========== 第1節：研究資料 ==========
        y_position = Self::render_section_header(&current_layer, &font, "1. 研究資料", y_position);
        
        // 基本資訊
        if let Some(ref content) = protocol.protocol.working_content {
            if let Some(basic) = content.get("basic") {
                // GLP 屬性
                let is_glp = basic.get("is_glp").and_then(|v| v.as_bool()).unwrap_or(false);
                y_position = Self::render_label_value(
                    &current_layer, &font,
                    "GLP 屬性",
                    if is_glp { "符合 GLP 規範" } else { "不符合 GLP 規範" },
                    y_position
                );

                // 計畫類型
                if let Some(project_type) = basic.get("project_type").and_then(|v| v.as_str()) {
                    y_position = Self::render_label_value(
                        &current_layer, &font,
                        "計畫類型",
                        project_type,
                        y_position
                    );
                }

                // 計畫種類
                if let Some(project_category) = basic.get("project_category").and_then(|v| v.as_str()) {
                    y_position = Self::render_label_value(
                        &current_layer, &font,
                        "計畫種類",
                        project_category,
                        y_position
                    );
                }

                // 預計試驗時程
                if let (Some(start), Some(end)) = (
                    protocol.protocol.start_date,
                    protocol.protocol.end_date
                ) {
                    let date_range = format!("{} ~ {}", start, end);
                    y_position = Self::render_label_value(
                        &current_layer, &font,
                        "預計試驗時程",
                        &date_range,
                        y_position
                    );
                }

                // 計畫主持人
                if let Some(pi) = basic.get("pi") {
                    y_position -= SECTION_SPACING_MM;
                    y_position = Self::render_subsection_header(&current_layer, &font, "計畫主持人", y_position);
                    
                    if let Some(name) = pi.get("name").and_then(|v| v.as_str()) {
                        y_position = Self::render_label_value(&current_layer, &font, "姓名", name, y_position);
                    }
                    if let Some(phone) = pi.get("phone").and_then(|v| v.as_str()) {
                        y_position = Self::render_label_value(&current_layer, &font, "電話", phone, y_position);
                    }
                    if let Some(email) = pi.get("email").and_then(|v| v.as_str()) {
                        y_position = Self::render_label_value(&current_layer, &font, "Email", email, y_position);
                    }
                    if let Some(address) = pi.get("address").and_then(|v| v.as_str()) {
                        y_position = Self::render_label_value(&current_layer, &font, "地址", address, y_position);
                    }
                }

                // 委託單位
                if let Some(sponsor) = basic.get("sponsor") {
                    y_position -= SECTION_SPACING_MM;
                    y_position = Self::render_subsection_header(&current_layer, &font, "委託單位", y_position);
                    
                    if let Some(name) = sponsor.get("name").and_then(|v| v.as_str()) {
                        y_position = Self::render_label_value(&current_layer, &font, "單位名稱", name, y_position);
                    }
                    if let Some(contact_person) = sponsor.get("contact_person").and_then(|v| v.as_str()) {
                        y_position = Self::render_label_value(&current_layer, &font, "聯絡人", contact_person, y_position);
                    }
                    if let Some(phone) = sponsor.get("contact_phone").and_then(|v| v.as_str()) {
                        y_position = Self::render_label_value(&current_layer, &font, "聯絡電話", phone, y_position);
                    }
                    if let Some(email) = sponsor.get("contact_email").and_then(|v| v.as_str()) {
                        y_position = Self::render_label_value(&current_layer, &font, "聯絡 Email", email, y_position);
                    }
                }
            }

            y_position -= SECTION_SPACING_MM;

            // ========== 第2節：研究目的 ==========
            if let Some(purpose) = content.get("purpose") {
                y_position = Self::render_section_header(&current_layer, &font, "2. 研究目的", y_position);

                if let Some(significance) = purpose.get("significance").and_then(|v| v.as_str()) {
                    y_position = Self::render_subsection_header(&current_layer, &font, "2.1 研究之目的及重要性", y_position);
                    y_position = Self::render_paragraph(&current_layer, &font, significance, y_position);
                }

                if let Some(replacement) = purpose.get("replacement") {
                    if let Some(rationale) = replacement.get("rationale").and_then(|v| v.as_str()) {
                        y_position = Self::render_subsection_header(&current_layer, &font, "2.2 替代原則說明", y_position);
                        y_position = Self::render_paragraph(&current_layer, &font, rationale, y_position);
                    }
                }

                if let Some(reduction) = purpose.get("reduction") {
                    if let Some(design) = reduction.get("design").and_then(|v| v.as_str()) {
                        y_position = Self::render_subsection_header(&current_layer, &font, "2.3 減量原則", y_position);
                        y_position = Self::render_paragraph(&current_layer, &font, design, y_position);
                    }
                }

                y_position -= SECTION_SPACING_MM;
            }

            // ========== 第3節：試驗物質 ==========
            if let Some(items) = content.get("items") {
                let use_test_item = items.get("use_test_item").and_then(|v| v.as_bool()).unwrap_or(false);
                y_position = Self::render_section_header(&current_layer, &font, "3. 試驗物質與對照物質", y_position);

                if use_test_item {
                    if let Some(test_items) = items.get("test_items").and_then(|v| v.as_array()) {
                        for (i, item) in test_items.iter().enumerate() {
                            let label = format!("試驗物質 #{}", i + 1);
                            y_position = Self::render_subsection_header(&current_layer, &font, &label, y_position);
                            
                            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                                y_position = Self::render_label_value(&current_layer, &font, "物質名稱", name, y_position);
                            }
                            if let Some(form) = item.get("form").and_then(|v| v.as_str()) {
                                y_position = Self::render_label_value(&current_layer, &font, "劑型", form, y_position);
                            }
                            if let Some(purpose) = item.get("purpose").and_then(|v| v.as_str()) {
                                y_position = Self::render_label_value(&current_layer, &font, "用途", purpose, y_position);
                            }
                        }
                    }
                } else {
                    y_position = Self::render_paragraph(&current_layer, &font, "略", y_position);
                }

                y_position -= SECTION_SPACING_MM;
            }

            // ========== 第4節：研究設計 ==========
            if let Some(design) = content.get("design") {
                y_position = Self::render_section_header(&current_layer, &font, "4. 研究設計與方法", y_position);

                if let Some(procedures) = design.get("procedures").and_then(|v| v.as_str()) {
                    y_position = Self::render_subsection_header(&current_layer, &font, "動物試驗流程描述", y_position);
                    y_position = Self::render_paragraph(&current_layer, &font, procedures, y_position);
                }

                if let Some(anesthesia) = design.get("anesthesia") {
                    let is_under_anesthesia = anesthesia.get("is_under_anesthesia").and_then(|v| v.as_bool()).unwrap_or(false);
                    y_position = Self::render_label_value(
                        &current_layer, &font,
                        "是否於麻醉下進行試驗",
                        if is_under_anesthesia { "是" } else { "否" },
                        y_position
                    );
                }

                if let Some(pain) = design.get("pain") {
                    if let Some(category) = pain.get("category").and_then(|v| v.as_str()) {
                        y_position = Self::render_label_value(&current_layer, &font, "疼痛類別", category, y_position);
                    }
                }

                y_position -= SECTION_SPACING_MM;
            }

            // ========== 第7節：實驗動物資料 ==========
            if let Some(animals) = content.get("animals") {
                y_position = Self::render_section_header(&current_layer, &font, "7. 實驗動物資料", y_position);

                if let Some(animal_list) = animals.get("animals").and_then(|v| v.as_array()) {
                    for (i, animal) in animal_list.iter().enumerate() {
                        let label = format!("動物群組 #{}", i + 1);
                        y_position = Self::render_subsection_header(&current_layer, &font, &label, y_position);
                        
                        if let Some(species) = animal.get("species").and_then(|v| v.as_str()) {
                            y_position = Self::render_label_value(&current_layer, &font, "物種", species, y_position);
                        }
                        if let Some(strain) = animal.get("strain").and_then(|v| v.as_str()) {
                            y_position = Self::render_label_value(&current_layer, &font, "品系", strain, y_position);
                        }
                        if let Some(sex) = animal.get("sex").and_then(|v| v.as_str()) {
                            y_position = Self::render_label_value(&current_layer, &font, "性別", sex, y_position);
                        }
                        if let Some(number) = animal.get("number").and_then(|v| v.as_i64()) {
                            y_position = Self::render_label_value(&current_layer, &font, "數量", &number.to_string(), y_position);
                        }
                        // 月齡範圍
                        let age_unlimited = animal.get("age_unlimited").and_then(|v| v.as_bool()).unwrap_or(false);
                        if age_unlimited {
                            y_position = Self::render_label_value(&current_layer, &font, "月齡範圍", "不限", y_position);
                        } else {
                            let age_min = animal.get("age_min").and_then(|v| v.as_str()).unwrap_or("不限");
                            let age_max = animal.get("age_max").and_then(|v| v.as_str()).unwrap_or("不限");
                            y_position = Self::render_label_value(&current_layer, &font, "月齡範圍", &format!("{} ~ {}", age_min, age_max), y_position);
                        }
                        // 體重範圍
                        let weight_unlimited = animal.get("weight_unlimited").and_then(|v| v.as_bool()).unwrap_or(false);
                        if weight_unlimited {
                            y_position = Self::render_label_value(&current_layer, &font, "體重範圍", "不限", y_position);
                        } else {
                            let weight_min = animal.get("weight_min").and_then(|v| v.as_str()).unwrap_or("不限");
                            let weight_max = animal.get("weight_max").and_then(|v| v.as_str()).unwrap_or("不限");
                            y_position = Self::render_label_value(&current_layer, &font, "體重範圍", &format!("{}kg ~ {}kg", weight_min, weight_max), y_position);
                        }
                        if let Some(housing) = animal.get("housing_location").and_then(|v| v.as_str()) {
                            y_position = Self::render_label_value(&current_layer, &font, "飼養位置", housing, y_position);
                        }
                    }
                }

                if let Some(total) = animals.get("total_animals").and_then(|v| v.as_i64()) {
                    y_position = Self::render_label_value(&current_layer, &font, "總動物數", &total.to_string(), y_position);
                }
                y_position -= SECTION_SPACING_MM;
            }

            // ========== 第5節：相關規範及參考文獻 ==========
            if let Some(guide) = content.get("guidelines") {
                y_position = Self::render_section_header(&current_layer, &font, "5. 相關規範及參考文獻", y_position);

                if let Some(guideline_content) = guide.get("content").and_then(|v| v.as_str()) {
                    y_position = Self::render_subsection_header(&current_layer, &font, "相關規範說明", y_position);
                    y_position = Self::render_paragraph(&current_layer, &font, guideline_content, y_position);
                }

                if let Some(references) = guide.get("references").and_then(|v| v.as_array()) {
                    if !references.is_empty() {
                        y_position = Self::render_subsection_header(&current_layer, &font, "參考文獻", y_position);
                        for (i, reference) in references.iter().enumerate() {
                            if let Some(citation) = reference.get("citation").and_then(|v| v.as_str()) {
                                let ref_text = format!("{}. {}", i + 1, citation);
                                y_position = Self::render_label_value(&current_layer, &font, "", &ref_text, y_position);
                            }
                        }
                    }
                }
                y_position -= SECTION_SPACING_MM;
            }

            // ========== 第6節：手術計畫書 ==========
            if let Some(surg) = content.get("surgery") {
                // 檢查是否需要手術計畫書
                let design_data = content.get("design");
                let needs_surgery = design_data
                    .and_then(|d| d.get("anesthesia"))
                    .and_then(|a| {
                        let is_under = a.get("is_under_anesthesia").and_then(|v| v.as_bool()).unwrap_or(false);
                        let anesthesia_type = a.get("anesthesia_type").and_then(|v| v.as_str()).unwrap_or("");
                        if is_under && (anesthesia_type == "survival_surgery" || anesthesia_type == "non_survival_surgery") {
                            Some(true)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(false);

                y_position = Self::render_section_header(&current_layer, &font, "6. 手術計畫書", y_position);

                if needs_surgery {
                    if let Some(surgery_type) = surg.get("surgery_type").and_then(|v| v.as_str()) {
                        y_position = Self::render_label_value(&current_layer, &font, "手術類型", surgery_type, y_position);
                    }
                    if let Some(preop) = surg.get("preop_preparation").and_then(|v| v.as_str()) {
                        y_position = Self::render_subsection_header(&current_layer, &font, "術前準備", y_position);
                        y_position = Self::render_paragraph(&current_layer, &font, preop, y_position);
                    }
                    if let Some(desc) = surg.get("surgery_description").and_then(|v| v.as_str()) {
                        y_position = Self::render_subsection_header(&current_layer, &font, "手術描述", y_position);
                        y_position = Self::render_paragraph(&current_layer, &font, desc, y_position);
                    }
                    if let Some(monitoring) = surg.get("monitoring").and_then(|v| v.as_str()) {
                        y_position = Self::render_subsection_header(&current_layer, &font, "監控方式", y_position);
                        y_position = Self::render_paragraph(&current_layer, &font, monitoring, y_position);
                    }
                    if let Some(postop) = surg.get("postop_care").and_then(|v| v.as_str()) {
                        y_position = Self::render_subsection_header(&current_layer, &font, "術後照護", y_position);
                        y_position = Self::render_paragraph(&current_layer, &font, postop, y_position);
                    }
                    // 用藥計畫
                    if let Some(drugs) = surg.get("drugs").and_then(|v| v.as_array()) {
                        if !drugs.is_empty() {
                            y_position = Self::render_subsection_header(&current_layer, &font, "用藥計畫", y_position);
                            for drug in drugs.iter() {
                                let drug_name = drug.get("drug_name").and_then(|v| v.as_str()).unwrap_or("-");
                                let dose = drug.get("dose").and_then(|v| v.as_str()).unwrap_or("-");
                                let route = drug.get("route").and_then(|v| v.as_str()).unwrap_or("-");
                                let frequency = drug.get("frequency").and_then(|v| v.as_str()).unwrap_or("-");
                                let drug_text = format!("{}: 劑量{}, 途徑{}, 頻率{}", drug_name, dose, route, frequency);
                                y_position = Self::render_label_value(&current_layer, &font, "", &drug_text, y_position);
                            }
                        }
                    }
                } else {
                    y_position = Self::render_paragraph(&current_layer, &font, "略", y_position);
                }
                y_position -= SECTION_SPACING_MM;
            }

            // ========== 第8節：試驗人員資料 ==========
            if let Some(personnel) = content.get("personnel").and_then(|v| v.as_array()) {
                if !personnel.is_empty() {
                    y_position = Self::render_section_header(&current_layer, &font, "8. 試驗人員資料", y_position);

                    for (i, person) in personnel.iter().enumerate() {
                        let label = format!("人員 #{}", i + 1);
                        y_position = Self::render_subsection_header(&current_layer, &font, &label, y_position);
                        
                        if let Some(name) = person.get("name").and_then(|v| v.as_str()) {
                            y_position = Self::render_label_value(&current_layer, &font, "姓名", name, y_position);
                        }
                        if let Some(position) = person.get("position").and_then(|v| v.as_str()) {
                            y_position = Self::render_label_value(&current_layer, &font, "職位", position, y_position);
                        }
                        if let Some(years) = person.get("years_experience").and_then(|v| v.as_i64()) {
                            y_position = Self::render_label_value(&current_layer, &font, "參與動物試驗年數", &format!("{} 年", years), y_position);
                        }
                        // 工作內容
                        if let Some(roles) = person.get("roles").and_then(|v| v.as_array()) {
                            let roles_str: Vec<&str> = roles.iter().filter_map(|r| r.as_str()).collect();
                            if !roles_str.is_empty() {
                                y_position = Self::render_label_value(&current_layer, &font, "工作內容", &roles_str.join(", "), y_position);
                            }
                        }
                        // 訓練/資格
                        if let Some(trainings) = person.get("trainings").and_then(|v| v.as_array()) {
                            let trainings_str: Vec<&str> = trainings.iter().filter_map(|t| t.as_str()).collect();
                            if !trainings_str.is_empty() {
                                y_position = Self::render_label_value(&current_layer, &font, "訓練/資格", &trainings_str.join(", "), y_position);
                            }
                        }
                    }
                    y_position -= SECTION_SPACING_MM;
                }
            }

            // ========== 第9節：附件 ==========
            if let Some(attachments) = content.get("attachments").and_then(|v| v.as_array()) {
                if !attachments.is_empty() {
                    y_position = Self::render_section_header(&current_layer, &font, "9. 附件", y_position);

                    for (i, attachment) in attachments.iter().enumerate() {
                        if let Some(file_name) = attachment.get("file_name").and_then(|v| v.as_str()) {
                            let att_text = format!("{}. {}", i + 1, file_name);
                            y_position = Self::render_label_value(&current_layer, &font, "", &att_text, y_position);
                        }
                    }
                    y_position -= SECTION_SPACING_MM;
                }
            }

            // 標記 y_position 被使用（避免編譯器警告）
            let _ = y_position;
        }

        // ========== 頁尾 ==========
        let footer_y = MARGIN_MM;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        current_layer.use_text(
            &format!("生成日期: {}", today),
            8.0,
            Mm(MARGIN_MM),
            Mm(footer_y),
            &font
        );

        // 輸出 PDF 為 bytes
        let pdf_bytes = doc.save_to_bytes()
            .map_err(|e| AppError::Internal(format!("Failed to generate PDF: {}", e)))?;

        Ok(pdf_bytes)
    }

    /// 渲染 section 標題
    fn render_section_header(layer: &PdfLayerReference, font: &IndirectFontRef, text: &str, y: f32) -> f32 {
        layer.use_text(text, 14.0, Mm(MARGIN_MM), Mm(y), font);
        y - LINE_HEIGHT_MM * 1.5
    }

    /// 渲染 subsection 標題
    fn render_subsection_header(layer: &PdfLayerReference, font: &IndirectFontRef, text: &str, y: f32) -> f32 {
        layer.use_text(text, 11.0, Mm(MARGIN_MM), Mm(y), font);
        y - LINE_HEIGHT_MM
    }

    /// 渲染標籤:值對
    fn render_label_value(layer: &PdfLayerReference, font: &IndirectFontRef, label: &str, value: &str, y: f32) -> f32 {
        let text = format!("{}：{}", label, value);
        layer.use_text(&text, 10.0, Mm(MARGIN_MM + 5.0), Mm(y), font);
        y - LINE_HEIGHT_MM
    }

    /// 渲染段落文字（處理長文字換行）
    fn render_paragraph(layer: &PdfLayerReference, font: &IndirectFontRef, text: &str, mut y: f32) -> f32 {
        // 簡化處理：按字元數分行（每行約 50 個字）
        let chars: Vec<char> = text.chars().collect();
        let line_width = 45;
        
        for chunk in chars.chunks(line_width) {
            let line: String = chunk.iter().collect();
            layer.use_text(&line, 10.0, Mm(MARGIN_MM + 5.0), Mm(y), font);
            y -= LINE_HEIGHT_MM;
            
            // 防止超出頁面底部
            if y < MARGIN_MM + 20.0 {
                break;
            }
        }
        
        y - LINE_HEIGHT_MM * 0.5
    }
}
