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
const MIN_Y_BEFORE_PAGE_BREAK: f32 = 40.0; // Minimum Y position before forcing a new page

/// PDF rendering context for multi-page support
struct PdfContext {
    doc: PdfDocumentReference,
    font: IndirectFontRef,
    current_layer: PdfLayerReference,
    y_position: f32,
    page_number: i32,
}

impl PdfContext {
    fn new(doc: PdfDocumentReference, font: IndirectFontRef, layer: PdfLayerReference) -> Self {
        Self {
            doc,
            font,
            current_layer: layer,
            y_position: PAGE_HEIGHT_MM - MARGIN_MM,
            page_number: 1,
        }
    }

    fn check_page_break(&mut self, required_space: f32) {
        if self.y_position - required_space < MIN_Y_BEFORE_PAGE_BREAK {
            self.add_new_page();
        }
    }

    fn add_new_page(&mut self) {
        self.page_number += 1;
        let (page, layer) = self.doc.add_page(
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            format!("第{}頁", self.page_number)
        );
        self.current_layer = self.doc.get_page(page).get_layer(layer);
        self.y_position = PAGE_HEIGHT_MM - MARGIN_MM;
    }

    fn force_new_page(&mut self) {
        if self.y_position < PAGE_HEIGHT_MM - MARGIN_MM - 10.0 {
            self.add_new_page();
        }
    }

    fn render_section_header(&mut self, text: &str) {
        self.check_page_break(LINE_HEIGHT_MM * 3.0);
        self.current_layer.use_text(text, 14.0, Mm(MARGIN_MM), Mm(self.y_position), &self.font);
        self.y_position -= LINE_HEIGHT_MM * 1.5;
    }

    fn render_subsection_header(&mut self, text: &str) {
        self.check_page_break(LINE_HEIGHT_MM * 2.0);
        self.current_layer.use_text(text, 11.0, Mm(MARGIN_MM), Mm(self.y_position), &self.font);
        self.y_position -= LINE_HEIGHT_MM;
    }

    fn render_label_value(&mut self, label: &str, value: &str) {
        self.check_page_break(LINE_HEIGHT_MM);
        let text = if label.is_empty() { value.to_string() } else { format!("{}：{}", label, value) };
        self.current_layer.use_text(&text, 10.0, Mm(MARGIN_MM + 5.0), Mm(self.y_position), &self.font);
        self.y_position -= LINE_HEIGHT_MM;
    }

    fn render_paragraph(&mut self, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        let line_width = 45;
        
        for chunk in chars.chunks(line_width) {
            self.check_page_break(LINE_HEIGHT_MM);
            let line: String = chunk.iter().collect();
            self.current_layer.use_text(&line, 10.0, Mm(MARGIN_MM + 5.0), Mm(self.y_position), &self.font);
            self.y_position -= LINE_HEIGHT_MM;
        }
        self.y_position -= LINE_HEIGHT_MM * 0.5;
    }

    fn add_section_spacing(&mut self) {
        self.y_position -= SECTION_SPACING_MM;
    }
}

/// PDF 生成服務
pub struct PdfService;

impl PdfService {
    fn get_project_type_label(key: &str, other: Option<&str>) -> String {
        let label = match key {
            "1_basic_research" | "basic_research" => "1. 基礎研究",
            "2_applied_research" | "applied_research" => "2. 應用研究",
            "3_pre_market_testing" | "pre_market_testing" => "3. 產品上市前測試",
            "4_educational" | "educational" | "4_teaching_training" | "teaching_training" => "4. 教學訓練",
            "5_biologics_manufacturing" | "biologics_manufacturing" => "5. 製造生物製劑",
            "6_other" | "other" => "6. 其他",
            _ => return key.to_string(),
        };
        
        match other {
            Some(o) if !o.is_empty() => format!("{} ({})", label, o),
            _ => label.to_string(),
        }
    }

    fn get_project_category_label(key: &str, other: Option<&str>) -> String {
        let label = match key {
            "1_medical" | "medical" => "1. 醫學研究",
            "2_agricultural" | "agricultural" => "2. 農業研究",
            "3_drugs_vaccines" | "drugs_vaccines" | "3_drug_herbal" | "drug_herbal" => "3. 藥物及疫苗",
            "4_supplements" | "supplements" | "4_health_food" | "health_food" => "4. 健康食品",
            "5_food" | "food" => "5. 食品",
            "6_toxics_chemicals" | "toxics_chemicals" | "6_toxic_chemical" | "toxic_chemical" => "6. 毒、化學品",
            "7_medical_materials" | "medical_materials" | "7_medical_device" | "medical_device" => "7. 醫療器材",
            "8_pesticide" | "pesticide" => "8. 農藥",
            "9_animal_drugs_vaccines" | "animal_drugs_vaccines" => "9. 動物用藥及疫苗",
            "10_animal_supplements_feed" | "animal_supplements_feed" => "10. 動物保健品、飼料添加物",
            "11_cosmetics" | "cosmetics" => "11. (含藥)化妝品",
            "12_other" | "other" => "12. 其他",
            _ => return key.to_string(),
        };
        
        match other {
            Some(o) if !o.is_empty() => format!("{} ({})", label, o),
            _ => label.to_string(),
        }
    }

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

        let initial_layer = doc.get_page(page1).get_layer(layer1);
        let mut ctx = PdfContext::new(doc, font, initial_layer);

        // ========== 標題 ==========
        ctx.current_layer.use_text(
            "AUP 動物試驗計畫書",
            24.0,
            Mm(PAGE_WIDTH_MM / 2.0 - 40.0),
            Mm(ctx.y_position),
            &ctx.font
        );
        ctx.y_position -= 12.0;

        // 計畫標題
        ctx.current_layer.use_text(
            &protocol.protocol.title,
            14.0,
            Mm(MARGIN_MM),
            Mm(ctx.y_position),
            &ctx.font
        );
        ctx.y_position -= SECTION_SPACING_MM * 2.0;

        // ========== 第1節：研究資料 ==========
        ctx.force_new_page();
        ctx.render_section_header("1. 研究資料");
        
        if let Some(ref content) = protocol.protocol.working_content {
            if let Some(basic) = content.get("basic") {
                let is_glp = basic.get("is_glp").and_then(|v| v.as_bool()).unwrap_or(false);
                ctx.render_label_value("GLP 屬性", if is_glp { "符合 GLP 規範" } else { "不符合 GLP 規範" });

                if let Some(project_type) = basic.get("project_type").and_then(|v| v.as_str()) {
                    let other = basic.get("project_type_other").and_then(|v| v.as_str());
                    ctx.render_label_value("計畫類型", &Self::get_project_type_label(project_type, other));
                }
                if let Some(project_category) = basic.get("project_category").and_then(|v| v.as_str()) {
                    let other = basic.get("project_category_other").and_then(|v| v.as_str());
                    ctx.render_label_value("計畫種類", &Self::get_project_category_label(project_category, other));
                }
                if let (Some(start), Some(end)) = (protocol.protocol.start_date, protocol.protocol.end_date) {
                    ctx.render_label_value("預計試驗時程", &format!("{} ~ {}", start, end));
                }

                if let Some(funding_sources) = basic.get("funding_sources").and_then(|v| v.as_array()) {
                    let mut labels = Vec::new();
                    for source in funding_sources {
                        if let Some(s) = source.as_str() {
                            let label = match s {
                                "moa" => "農業部 Ministry of Agriculture",
                                "mohw" => "衛生福利部 Ministry of Health and Welfare",
                                "nstc" => "國家科學及技術委員會 NSTC",
                                "moe" => "教育部 Ministry of Education",
                                "env" => "環境部 Ministry of Environment",
                                "other" => "其他 Other",
                                _ => s,
                            };
                            labels.push(label);
                        }
                    }
                    if !labels.is_empty() {
                        let mut value = labels.join(", ");
                        if funding_sources.iter().any(|s| s.as_str() == Some("other")) {
                            if let Some(other) = basic.get("funding_other").and_then(|v| v.as_str()) {
                                if !other.is_empty() {
                                    value = format!("{} ({})", value, other);
                                }
                            }
                        }
                        ctx.render_label_value("資金來源", &value);
                    }
                }

                // 計畫主持人
                if let Some(pi) = basic.get("pi") {
                    ctx.add_section_spacing();
                    ctx.render_subsection_header("計畫主持人");
                    if let Some(name) = pi.get("name").and_then(|v| v.as_str()) { ctx.render_label_value("姓名", name); }
                    if let Some(phone) = pi.get("phone").and_then(|v| v.as_str()) { ctx.render_label_value("電話", phone); }
                    if let Some(email) = pi.get("email").and_then(|v| v.as_str()) { ctx.render_label_value("Email", email); }
                    if let Some(address) = pi.get("address").and_then(|v| v.as_str()) { ctx.render_label_value("地址", address); }
                }

                // 委託單位
                if let Some(sponsor) = basic.get("sponsor") {
                    ctx.add_section_spacing();
                    ctx.render_subsection_header("委託單位");
                    if let Some(name) = sponsor.get("name").and_then(|v| v.as_str()) { ctx.render_label_value("單位名稱", name); }
                    if let Some(contact_person) = sponsor.get("contact_person").and_then(|v| v.as_str()) { ctx.render_label_value("聯絡人", contact_person); }
                    if let Some(phone) = sponsor.get("contact_phone").and_then(|v| v.as_str()) { ctx.render_label_value("聯絡電話", phone); }
                    if let Some(email) = sponsor.get("contact_email").and_then(|v| v.as_str()) { ctx.render_label_value("聯絡 Email", email); }
                }

                // 試驗機構與設施
                if let Some(facility) = basic.get("facility") {
                    ctx.add_section_spacing();
                    ctx.render_subsection_header("試驗機構與設施");
                    if let Some(title) = facility.get("title").and_then(|v| v.as_str()) { ctx.render_label_value("機構名稱", title); }
                }
                if let Some(loc) = basic.get("housing_location").and_then(|v| v.as_str()) { ctx.render_label_value("位置", loc); }
            }

            ctx.add_section_spacing();

            // ========== 第2節：研究目的 ==========
            if let Some(purpose) = content.get("purpose") {
                ctx.force_new_page();
                ctx.render_section_header("2. 研究目的");
                if let Some(significance) = purpose.get("significance").and_then(|v| v.as_str()) {
                    ctx.render_subsection_header("2.1 研究之目的及重要性");
                    ctx.render_paragraph(significance);
                }
                if let Some(replacement) = purpose.get("replacement") {
                    if let Some(rationale) = replacement.get("rationale").and_then(|v| v.as_str()) {
                        ctx.render_subsection_header("2.2 3Rs之替代原則");
                        ctx.render_paragraph(rationale);
                    }
                    if let Some(alt_search) = replacement.get("alt_search") {
                        ctx.render_subsection_header("2.2.2 確曾非動物性替代方案");
                        if let Some(platforms) = alt_search.get("platforms").and_then(|v| v.as_array()) {
                            let platforms_str: Vec<&str> = platforms.iter().filter_map(|p| p.as_str()).collect();
                            if !platforms_str.is_empty() { ctx.render_label_value("查詢平台", &platforms_str.join(", ")); }
                        }
                        if let Some(keywords) = alt_search.get("keywords").and_then(|v| v.as_str()) { ctx.render_label_value("關鍵字", keywords); }
                        if let Some(conclusion) = alt_search.get("conclusion").and_then(|v| v.as_str()) { ctx.render_paragraph(conclusion); }
                    }
                }
                if let Some(duplicate) = purpose.get("duplicate") {
                    ctx.render_subsection_header("2.2.3 是否為重複他人試驗");
                    let is_dup = duplicate.get("experiment").and_then(|v| v.as_bool()).unwrap_or(false);
                    ctx.render_label_value("", if is_dup { "是" } else { "否" });
                    if is_dup { if let Some(just) = duplicate.get("justification").and_then(|v| v.as_str()) { ctx.render_paragraph(just); } }
                }
                if let Some(reduction) = purpose.get("reduction") {
                    if let Some(design) = reduction.get("design").and_then(|v| v.as_str()) {
                        ctx.render_subsection_header("2.3 減量原則");
                        ctx.render_paragraph(design);
                    }
                }
                ctx.add_section_spacing();
            }

            // ========== 第3節：試驗物質 ==========
            if let Some(items) = content.get("items") {
                let use_test_item = items.get("use_test_item").and_then(|v| v.as_bool()).unwrap_or(false);
                ctx.force_new_page();
                ctx.render_section_header("3. 試驗物質與對照物質");
                if use_test_item {
                    if let Some(test_items) = items.get("test_items").and_then(|v| v.as_array()) {
                        for (i, item) in test_items.iter().enumerate() {
                            ctx.render_subsection_header(&format!("試驗物質 #{}", i + 1));
                            if let Some(name) = item.get("name").and_then(|v| v.as_str()) { ctx.render_label_value("物質名稱", name); }
                            if let Some(form) = item.get("form").and_then(|v| v.as_str()) { ctx.render_label_value("劑型", form); }
                            if let Some(purpose) = item.get("purpose").and_then(|v| v.as_str()) { ctx.render_label_value("用途", purpose); }
                            if let Some(storage) = item.get("storage_conditions").and_then(|v| v.as_str()) { ctx.render_label_value("儲存條件", storage); }
                            let is_sterile = item.get("is_sterile").and_then(|v| v.as_bool()).unwrap_or(true);
                            ctx.render_label_value("無菌", if is_sterile { "是" } else { "否" });
                        }
                    }
                    if let Some(ctrl_items) = items.get("control_items").and_then(|v| v.as_array()) {
                        for (i, item) in ctrl_items.iter().enumerate() {
                            ctx.render_subsection_header(&format!("對照物質 #{}", i + 1));
                            if let Some(name) = item.get("name").and_then(|v| v.as_str()) { ctx.render_label_value("物質名稱", name); }
                            if let Some(purpose) = item.get("purpose").and_then(|v| v.as_str()) { ctx.render_label_value("用途", purpose); }
                        }
                    }
                } else {
                    ctx.render_paragraph("略");
                }
                ctx.add_section_spacing();
            }

            // ========== 第4節：研究設計 ==========
            if let Some(design) = content.get("design") {
                ctx.force_new_page();
                ctx.render_section_header("4. 研究設計與方法");
                if let Some(procedures) = design.get("procedures").and_then(|v| v.as_str()) {
                    ctx.render_subsection_header("動物試驗流程描述");
                    ctx.render_paragraph(procedures);
                }
                if let Some(anesthesia) = design.get("anesthesia") {
                    let is_under = anesthesia.get("is_under_anesthesia").and_then(|v| v.as_bool()).unwrap_or(false);
                    ctx.render_label_value("是否於麻醉下進行試驗", if is_under { "是" } else { "否" });
                    if let Some(a_type) = anesthesia.get("anesthesia_type").and_then(|v| v.as_str()) { ctx.render_label_value("麻醉類型", a_type); }
                }
                if let Some(pain) = design.get("pain") {
                    if let Some(category) = pain.get("category").and_then(|v| v.as_str()) { ctx.render_label_value("疼痛類別", category); }
                    if let Some(mgmt) = pain.get("management_plan").and_then(|v| v.as_str()) { ctx.render_subsection_header("疼痛管理方案"); ctx.render_paragraph(mgmt); }
                }
                if let Some(endpoints) = design.get("endpoints") {
                    if let Some(exp_ep) = endpoints.get("experimental_endpoint").and_then(|v| v.as_str()) { ctx.render_subsection_header("試驗終點"); ctx.render_paragraph(exp_ep); }
                    if let Some(hum_ep) = endpoints.get("humane_endpoint").and_then(|v| v.as_str()) { ctx.render_subsection_header("人道終點"); ctx.render_paragraph(hum_ep); }
                }
                ctx.add_section_spacing();
            }

            // ========== 第5節：相關規範及參考文獻 ==========
            if let Some(guide) = content.get("guidelines") {
                ctx.force_new_page();
                ctx.render_section_header("5. 相關規範及參考文獻");
                if let Some(g_content) = guide.get("content").and_then(|v| v.as_str()) {
                    ctx.render_subsection_header("相關規範說明");
                    ctx.render_paragraph(g_content);
                }
                if let Some(refs) = guide.get("references").and_then(|v| v.as_array()) {
                    if !refs.is_empty() {
                        ctx.render_subsection_header("參考文獻");
                        for (i, r) in refs.iter().enumerate() {
                            if let Some(citation) = r.get("citation").and_then(|v| v.as_str()) {
                                ctx.render_label_value("", &format!("{}. {}", i + 1, citation));
                            }
                        }
                    }
                }
                ctx.add_section_spacing();
            }

            // ========== 第6節：手術計畫書 ==========
            if let Some(surg) = content.get("surgery") {
                let design_data = content.get("design");
                let needs_surgery = design_data.and_then(|d| d.get("anesthesia")).and_then(|a| {
                    let is_under = a.get("is_under_anesthesia").and_then(|v| v.as_bool()).unwrap_or(false);
                    let a_type = a.get("anesthesia_type").and_then(|v| v.as_str()).unwrap_or("");
                    if is_under && (a_type == "survival_surgery" || a_type == "non_survival_surgery") { Some(true) } else { None }
                }).unwrap_or(false);
                ctx.force_new_page();
                ctx.render_section_header("6. 手術計畫書");
                if needs_surgery {
                    if let Some(st) = surg.get("surgery_type").and_then(|v| v.as_str()) { ctx.render_label_value("手術類型", st); }
                    if let Some(preop) = surg.get("preop_preparation").and_then(|v| v.as_str()) { ctx.render_subsection_header("術前準備"); ctx.render_paragraph(preop); }
                    if let Some(desc) = surg.get("surgery_description").and_then(|v| v.as_str()) { ctx.render_subsection_header("手術描述"); ctx.render_paragraph(desc); }
                    if let Some(mon) = surg.get("monitoring").and_then(|v| v.as_str()) { ctx.render_subsection_header("監控方式"); ctx.render_paragraph(mon); }
                    if let Some(postop) = surg.get("postop_care").and_then(|v| v.as_str()) { ctx.render_subsection_header("術後照護"); ctx.render_paragraph(postop); }
                    if let Some(drugs) = surg.get("drugs").and_then(|v| v.as_array()) {
                        if !drugs.is_empty() {
                            ctx.render_subsection_header("用藥計畫");
                            for drug in drugs.iter() {
                                let dn = drug.get("drug_name").and_then(|v| v.as_str()).unwrap_or("-");
                                let dose = drug.get("dose").and_then(|v| v.as_str()).unwrap_or("-");
                                let route = drug.get("route").and_then(|v| v.as_str()).unwrap_or("-");
                                let freq = drug.get("frequency").and_then(|v| v.as_str()).unwrap_or("-");
                                ctx.render_label_value("", &format!("{}: 劑量{}, 途徑{}, 頻率{}", dn, dose, route, freq));
                            }
                        }
                    }
                } else {
                    ctx.render_paragraph("略");
                }
                ctx.add_section_spacing();
            }

            // ========== 第7節：實驗動物資料 ==========
            if let Some(animals) = content.get("animals") {
                ctx.force_new_page();
                ctx.render_section_header("7. 實驗動物資料");
                if let Some(animal_list) = animals.get("animals").and_then(|v| v.as_array()) {
                    for (i, animal) in animal_list.iter().enumerate() {
                        ctx.render_subsection_header(&format!("動物群組 #{}", i + 1));
                        if let Some(sp) = animal.get("species").and_then(|v| v.as_str()) { ctx.render_label_value("物種", sp); }
                        if let Some(st) = animal.get("strain").and_then(|v| v.as_str()) { ctx.render_label_value("品系", st); }
                        if let Some(sx) = animal.get("sex").and_then(|v| v.as_str()) { ctx.render_label_value("性別", sx); }
                        if let Some(num) = animal.get("number").and_then(|v| v.as_i64()) { ctx.render_label_value("數量", &num.to_string()); }
                        let age_unlim = animal.get("age_unlimited").and_then(|v| v.as_bool()).unwrap_or(false);
                        if age_unlim { ctx.render_label_value("月齡範圍", "不限"); }
                        else {
                            let amin = animal.get("age_min").and_then(|v| v.as_str()).unwrap_or("不限");
                            let amax = animal.get("age_max").and_then(|v| v.as_str()).unwrap_or("不限");
                            ctx.render_label_value("月齡範圍", &format!("{} ~ {}", amin, amax));
                        }
                        let wt_unlim = animal.get("weight_unlimited").and_then(|v| v.as_bool()).unwrap_or(false);
                        if wt_unlim { ctx.render_label_value("體重範圍", "不限"); }
                        else {
                            let wmin = animal.get("weight_min").and_then(|v| v.as_str()).unwrap_or("不限");
                            let wmax = animal.get("weight_max").and_then(|v| v.as_str()).unwrap_or("不限");
                            ctx.render_label_value("體重範圍", &format!("{}kg ~ {}kg", wmin, wmax));
                        }
                        if let Some(loc) = animal.get("housing_location").and_then(|v| v.as_str()) { ctx.render_label_value("飼養位置", loc); }
                    }
                }
                if let Some(total) = animals.get("total_animals").and_then(|v| v.as_i64()) { ctx.render_label_value("總動物數", &total.to_string()); }
                ctx.add_section_spacing();
            }

            // ========== 第8節：試驗人員資料 ==========
            if let Some(personnel) = content.get("personnel").and_then(|v| v.as_array()) {
                if !personnel.is_empty() {
                    ctx.force_new_page();
                    ctx.render_section_header("8. 試驗人員資料");
                    for (i, person) in personnel.iter().enumerate() {
                        ctx.render_subsection_header(&format!("人員 #{}", i + 1));
                        if let Some(name) = person.get("name").and_then(|v| v.as_str()) { ctx.render_label_value("姓名", name); }
                        if let Some(pos) = person.get("position").and_then(|v| v.as_str()) { ctx.render_label_value("職位", pos); }
                        if let Some(yrs) = person.get("years_experience").and_then(|v| v.as_i64()) { ctx.render_label_value("參與動物試驗年數", &format!("{} 年", yrs)); }
                        if let Some(roles) = person.get("roles").and_then(|v| v.as_array()) {
                            let rs: Vec<&str> = roles.iter().filter_map(|r| r.as_str()).collect();
                            if !rs.is_empty() { ctx.render_label_value("工作內容", &rs.join(", ")); }
                        }
                        if let Some(trainings) = person.get("trainings").and_then(|v| v.as_array()) {
                            let ts: Vec<&str> = trainings.iter().filter_map(|t| t.as_str()).collect();
                            if !ts.is_empty() { ctx.render_label_value("訓練/資格", &ts.join(", ")); }
                        }
                    }
                    ctx.add_section_spacing();
                }
            }

            // ========== 第9節：附件 ==========
            if let Some(attachments) = content.get("attachments").and_then(|v| v.as_array()) {
                if !attachments.is_empty() {
                    ctx.force_new_page();
                    ctx.render_section_header("9. 附件");
                    for (i, att) in attachments.iter().enumerate() {
                        if let Some(fname) = att.get("file_name").and_then(|v| v.as_str()) {
                            ctx.render_label_value("", &format!("{}. {}", i + 1, fname));
                        }
                    }
                    ctx.add_section_spacing();
                }
            }
        }

        // ========== 頁尾 ==========
        let footer_y = MARGIN_MM;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        ctx.current_layer.use_text(
            &format!("生成日期: {} | 頁 {} ", today, ctx.page_number),
            8.0,
            Mm(MARGIN_MM),
            Mm(footer_y),
            &ctx.font
        );

        // 輸出 PDF 為 bytes
        let pdf_bytes = ctx.doc.save_to_bytes()
            .map_err(|e| AppError::Internal(format!("Failed to generate PDF: {}", e)))?;

        Ok(pdf_bytes)
    }

    /// 生成豬隻病歷 PDF
    pub fn generate_medical_pdf(data: &serde_json::Value) -> Result<Vec<u8>> {
        // 建立 PDF 文件
        let (doc, page1, layer1) = PdfDocument::new(
            "豬隻病歷紀錄",
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            "第1頁"
        );

        // 載入中文字型
        let font_path = std::path::Path::new("resources/fonts/NotoSansSC-Regular.ttf");
        let font_bytes = std::fs::read(font_path)
            .map_err(|e| AppError::Internal(format!("Failed to read font file: {}", e)))?;
        let font = doc.add_external_font(&*font_bytes)
            .map_err(|e| AppError::Internal(format!("Failed to load font: {}", e)))?;

        let initial_layer = doc.get_page(page1).get_layer(layer1);
        let mut ctx = PdfContext::new(doc, font, initial_layer);

        // ========== 標題 ==========
        ctx.current_layer.use_text(
            "豬隻病歷紀錄總表",
            20.0,
            Mm(PAGE_WIDTH_MM / 2.0 - 40.0),
            Mm(ctx.y_position),
            &ctx.font
        );
        ctx.y_position -= SECTION_SPACING_MM * 2.0;

        // 使用共用渲染方法
        Self::render_pig_medical_data(&mut ctx, data);

        // ========== 頁尾 ==========
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        ctx.current_layer.use_text(
            &format!("生成日期: {} | 頁 {} ", today, ctx.page_number),
            8.0,
            Mm(MARGIN_MM),
            Mm(MARGIN_MM),
            &ctx.font
        );

        let pdf_bytes = ctx.doc.save_to_bytes()
            .map_err(|e| AppError::Internal(format!("Failed to generate PDF: {}", e)))?;

        Ok(pdf_bytes)
    }


    /// 共用：渲染豬隻完整病歷資料（基本資料、體重、疫苗、觀察、手術）
    /// 用於單隻匯出與批次匯出，統一 session-per-page 分頁邏輯
    fn render_pig_medical_data(ctx: &mut PdfContext, data: &serde_json::Value) {
        // ========== 1. 豬隻基本資料 ==========
        if let Some(pig) = data.get("pig") {
            ctx.render_section_header("1. 豬隻基本資料");
            let ear_tag = pig.get("ear_tag").and_then(|v| v.as_str()).unwrap_or("-");
            let breed = pig.get("breed").and_then(|v| v.as_str()).unwrap_or("-");
            let gender = pig.get("gender").and_then(|v| v.as_str()).unwrap_or("-");
            let iacuc_no = pig.get("iacuc_no").and_then(|v| v.as_str()).unwrap_or("未分配");

            ctx.render_label_value("耳號", ear_tag);
            ctx.render_label_value("計畫編號", iacuc_no);
            ctx.render_label_value("品種", breed);
            ctx.render_label_value("性別", if gender == "male" { "公" } else { "母" });

            if let Some(entry_date) = pig.get("entry_date").and_then(|v| v.as_str()) {
                ctx.render_label_value("進場日期", entry_date);
            }
            if let Some(birth_date) = pig.get("birth_date").and_then(|v| v.as_str()) {
                ctx.render_label_value("出生日期", birth_date);
            }
            if let Some(loc) = pig.get("pen_location").and_then(|v| v.as_str()) {
                ctx.render_label_value("欄位編號", loc);
            }
            ctx.add_section_spacing();
        }

        // ========== 2. 體重紀錄 ==========
        if let Some(weights) = data.get("weights").and_then(|v| v.as_array()) {
            if !weights.is_empty() {
                ctx.render_section_header("2. 體重紀錄");
                for w in weights {
                    let date = w.get("measure_date").and_then(|v| v.as_str()).unwrap_or("-");
                    let weight = w.get("weight").map(|v| v.to_string()).unwrap_or("-".to_string());
                    ctx.render_label_value(&format!("日期: {}", date), &format!("{} kg", weight));
                }
                ctx.add_section_spacing();
            }
        }

        // ========== 3. 疫苗/驅蟲紀錄 ==========
        if let Some(vaccinations) = data.get("vaccinations").and_then(|v| v.as_array()) {
            if !vaccinations.is_empty() {
                ctx.render_section_header("3. 疫苗/驅蟲紀錄");
                for v in vaccinations {
                    let date = v.get("administered_date").and_then(|v| v.as_str()).unwrap_or("-");
                    let vaccine = v.get("vaccine").and_then(|v| v.as_str()).unwrap_or("-");
                    let dose = v.get("deworming_dose").and_then(|v| v.as_str()).unwrap_or("-");
                    ctx.render_label_value(&format!("日期: {}", date), &format!("項目: {}, 劑量: {}", vaccine, dose));
                }
                ctx.add_section_spacing();
            }
        }

        // ========== 4. 觀察試驗紀錄 (Session per page) ==========
        if let Some(observations) = data.get("observations").and_then(|v| v.as_array()) {
            if !observations.is_empty() {
                ctx.render_section_header("4. 觀察試驗紀錄");
                for obs in observations {
                    ctx.force_new_page();
                    let date = obs.get("event_date").and_then(|v| v.as_str()).unwrap_or("-");
                    let rtype = obs.get("record_type").and_then(|v| v.as_str()).unwrap_or("-");
                    let content = obs.get("content").and_then(|v| v.as_str()).unwrap_or("-");

                    ctx.render_subsection_header(&format!("觀察紀錄 - {}", date));
                    ctx.render_label_value("紀錄類型", rtype);
                    ctx.render_label_value("內容", "");
                    ctx.render_paragraph(content);

                    if let Some(treatments) = obs.get("treatments").and_then(|v| v.as_array()) {
                        if !treatments.is_empty() {
                            ctx.render_label_value("治療方式", "");
                            for t in treatments {
                                let drug = t.get("drug").and_then(|v| v.as_str()).unwrap_or("-");
                                let dose = t.get("dosage").and_then(|v| v.as_str()).unwrap_or("-");
                                ctx.render_paragraph(&format!("藥物: {}, 劑量: {}", drug, dose));
                            }
                        }
                    }
                }
            }
        }

        // ========== 5. 手術紀錄 (Session per page) ==========
        if let Some(surgeries) = data.get("surgeries").and_then(|v| v.as_array()) {
            if !surgeries.is_empty() {
                ctx.render_section_header("5. 手術紀錄");
                for surg in surgeries {
                    ctx.force_new_page();
                    let date = surg.get("surgery_date").and_then(|v| v.as_str()).unwrap_or("-");
                    let site = surg.get("surgery_site").and_then(|v| v.as_str()).unwrap_or("-");

                    ctx.render_subsection_header(&format!("手術紀錄 - {}", date));
                    ctx.render_label_value("手術部位", site);

                    if let Some(remark) = surg.get("remark").and_then(|v| v.as_str()) {
                        ctx.render_label_value("備註", "");
                        ctx.render_paragraph(remark);
                    }
                }
            }
        }
    }

    /// 生成計畫下所有豬隻病歷 PDF
    pub fn generate_project_medical_pdf(iacuc_no: &str, pigs_data: &serde_json::Value) -> Result<Vec<u8>> {
        // 建立 PDF 文件
        let (doc, page1, layer1) = PdfDocument::new(
            format!("計畫病歷匯出 - {}", iacuc_no),
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            "第1頁"
        );

        // 載入中文字型
        let font_path = std::path::Path::new("resources/fonts/NotoSansSC-Regular.ttf");
        let font_bytes = std::fs::read(font_path)
            .map_err(|e| AppError::Internal(format!("Failed to read font file: {}", e)))?;
        let font = doc.add_external_font(&*font_bytes)
            .map_err(|e| AppError::Internal(format!("Failed to load font: {}", e)))?;

        let initial_layer = doc.get_page(page1).get_layer(layer1);
        let mut ctx = PdfContext::new(doc, font, initial_layer);

        // ========== 封面標題 ==========
        ctx.current_layer.use_text(
            &format!("計畫病歷匯出總表 - {}", iacuc_no),
            20.0,
            Mm(MARGIN_MM),
            Mm(ctx.y_position),
            &ctx.font
        );
        ctx.y_position -= SECTION_SPACING_MM * 2.0;

        // 顯示匯出日期
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        ctx.render_label_value("匯出日期", &today);

        if let Some(pigs) = pigs_data.get("pigs").and_then(|v| v.as_array()) {
            ctx.render_label_value("豬隻總數", &pigs.len().to_string());
            ctx.add_section_spacing();

            // 封面：豬隻清單摘要
            ctx.render_subsection_header("豬隻清單");
            for (i, pig_data) in pigs.iter().enumerate() {
                if let Some(pig) = pig_data.get("pig") {
                    let ear_tag = pig.get("ear_tag").and_then(|v| v.as_str()).unwrap_or("-");
                    let breed = pig.get("breed").and_then(|v| v.as_str()).unwrap_or("-");
                    let gender = pig.get("gender").and_then(|v| v.as_str()).unwrap_or("-");
                    let gender_label = if gender == "male" { "公" } else { "母" };
                    ctx.render_label_value("", &format!("{}. {} ({}, {})", i + 1, ear_tag, breed, gender_label));
                }
            }

            // 每隻豬隻獨立分頁渲染完整病歷
            for pig_data in pigs.iter() {
                ctx.add_new_page();
                Self::render_pig_medical_data(&mut ctx, pig_data);
            }
        }

        // ========== 頁尾 ==========
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        ctx.current_layer.use_text(
            &format!("生成日期: {} | 頁 {} ", today, ctx.page_number),
            8.0,
            Mm(MARGIN_MM),
            Mm(MARGIN_MM),
            &ctx.font
        );

        let pdf_bytes = ctx.doc.save_to_bytes()
            .map_err(|e| AppError::Internal(format!("Failed to generate PDF: {}", e)))?;

        Ok(pdf_bytes)
    }
}

