use std::sync::OnceLock;

use crate::models::{ProtocolResponse, WarehouseReportData};
use crate::time;
use crate::{AppError, Result};
use printpdf::*;

use super::context::*;

/// 字型快取：只讀一次磁碟，後續重複使用
static FONT_CACHE: OnceLock<Vec<u8>> = OnceLock::new();

/// 建築結構類型（佈局圖中不繪製庫存的元素）
const STRUCTURE_TYPES: &[&str] = &["wall", "door", "window"];

/// PDF 生成服務
pub struct PdfService;

impl PdfService {
    fn get_project_type_label(key: &str, other: Option<&str>) -> String {
        let label = match key {
            "1_basic_research" | "basic_research" => "1. 基礎研究",
            "2_applied_research" | "applied_research" => "2. 應用研究",
            "3_pre_market_testing" | "pre_market_testing" => "3. 產品上市前測試",
            "4_educational" | "educational" | "4_teaching_training" | "teaching_training" => {
                "4. 教學訓練"
            }
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
            "3_drugs_vaccines" | "drugs_vaccines" | "3_drug_herbal" | "drug_herbal" => {
                "3. 藥物及疫苗"
            }
            "4_supplements" | "supplements" | "4_health_food" | "health_food" => "4. 健康食品",
            "5_food" | "food" => "5. 食品",
            "6_toxics_chemicals" | "toxics_chemicals" | "6_toxic_chemical" | "toxic_chemical" => {
                "6. 毒、化學品"
            }
            "7_medical_materials" | "medical_materials" | "7_medical_device" | "medical_device" => {
                "7. 醫療器材"
            }
            "8_pesticide" | "pesticide" => "8. 農藥",
            "9_animal_drugs_vaccines" | "animal_drugs_vaccines" => "9. 動物用藥及疫苗",
            "10_animal_supplements_feed" | "animal_supplements_feed" => {
                "10. 動物保健品、飼料添加物"
            }
            "11_cosmetics" | "cosmetics" => "11. (含藥)化妝品",
            "12_other" | "other" => "12. 其他",
            _ => return key.to_string(),
        };

        match other {
            Some(o) if !o.is_empty() => format!("{} ({})", label, o),
            _ => label.to_string(),
        }
    }

    /// 初始化 PDF 文件並載入字型 (printpdf 0.9)
    /// 字型檔透過 OnceLock 快取，只在第一次呼叫時讀取磁碟
    fn init_pdf_context(title: &str) -> Result<PdfContext> {
        let mut doc = PdfDocument::new(title);

        let font_bytes = FONT_CACHE.get_or_init(|| {
            let font_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("resources/fonts/NotoSansSC-Regular.ttf");
            std::fs::read(&font_path)
                .unwrap_or_else(|e| panic!("Failed to read font file {}: {}", font_path.display(), e))
        });

        let mut warnings = Vec::new();
        let parsed_font = ParsedFont::from_bytes(font_bytes, 0, &mut warnings)
            .ok_or_else(|| AppError::Internal("Failed to parse font".to_string()))?;

        let font_id = doc.add_font(&parsed_font);
        let font_handle = PdfFontHandle::External(font_id);

        Ok(PdfContext::new(doc, font_handle))
    }

    /// 渲染 PDF 標題區塊
    fn render_protocol_title(ctx: &mut PdfContext, title: &str) {
        ctx.push_text("AUP 動物試驗計畫書", 24.0, PAGE_WIDTH_MM / 2.0 - 40.0, ctx.y_position);
        ctx.y_position -= 12.0;

        ctx.push_text(title, 14.0, MARGIN_MM, ctx.y_position);
        ctx.y_position -= SECTION_SPACING_MM * 2.0;
    }

    /// 渲染資金來源
    fn render_funding_sources(
        ctx: &mut PdfContext,
        basic: &serde_json::Value,
    ) {
        let funding_sources = match basic.get("funding_sources").and_then(|v| v.as_array()) {
            Some(fs) => fs,
            None => return,
        };

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
        if labels.is_empty() {
            return;
        }

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

    /// 渲染計畫主持人資訊
    fn render_pi_info(ctx: &mut PdfContext, pi: &serde_json::Value) {
        ctx.add_section_spacing();
        ctx.render_subsection_header("計畫主持人");
        if let Some(name) = pi.get("name").and_then(|v| v.as_str()) {
            ctx.render_label_value("姓名", name);
        }
        if let Some(phone) = pi.get("phone").and_then(|v| v.as_str()) {
            let phone_val = match pi.get("phone_ext").and_then(|v| v.as_str()) {
                Some(ext) if !ext.is_empty() => format!("{} #{}", phone, ext),
                _ => phone.to_string(),
            };
            ctx.render_label_value("電話", &phone_val);
        }
        if let Some(email) = pi.get("email").and_then(|v| v.as_str()) {
            ctx.render_label_value("Email", email);
        }
        if let Some(address) = pi.get("address").and_then(|v| v.as_str()) {
            ctx.render_label_value("地址", address);
        }
    }

    /// 渲染委託單位資訊
    fn render_sponsor_info(ctx: &mut PdfContext, sponsor: &serde_json::Value) {
        ctx.add_section_spacing();
        ctx.render_subsection_header("委託單位");
        if let Some(name) = sponsor.get("name").and_then(|v| v.as_str()) {
            ctx.render_label_value("單位名稱", name);
        }
        if let Some(contact_person) = sponsor.get("contact_person").and_then(|v| v.as_str()) {
            ctx.render_label_value("聯絡人", contact_person);
        }
        if let Some(phone) = sponsor.get("contact_phone").and_then(|v| v.as_str()) {
            let phone_val = match sponsor.get("contact_phone_ext").and_then(|v| v.as_str()) {
                Some(ext) if !ext.is_empty() => format!("{} #{}", phone, ext),
                _ => phone.to_string(),
            };
            ctx.render_label_value("聯絡電話", &phone_val);
        }
        if let Some(email) = sponsor.get("contact_email").and_then(|v| v.as_str()) {
            ctx.render_label_value("聯絡 Email", email);
        }
    }

    /// 渲染第1節：研究資料
    fn render_section_1(ctx: &mut PdfContext, protocol: &ProtocolResponse, content: &serde_json::Value) {
        let basic = match content.get("basic") {
            Some(b) => b,
            None => return,
        };

        ctx.force_new_page();
        ctx.render_section_header("1. 研究資料");

        let is_glp = basic
            .get("is_glp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        ctx.render_label_value(
            "GLP 符合性",
            if is_glp { "GLP" } else { "非GLP" },
        );

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

        Self::render_funding_sources(ctx, basic);

        if let Some(pi) = basic.get("pi") {
            Self::render_pi_info(ctx, pi);
        }
        if let Some(sponsor) = basic.get("sponsor") {
            Self::render_sponsor_info(ctx, sponsor);
        }

        // 試驗機構與設施
        if let Some(facility) = basic.get("facility") {
            ctx.add_section_spacing();
            ctx.render_subsection_header("試驗機構與設施");
            if let Some(title) = facility.get("title").and_then(|v| v.as_str()) {
                ctx.render_label_value("機構名稱", title);
            }
        }
        if let Some(loc) = basic.get("housing_location").and_then(|v| v.as_str()) {
            ctx.render_label_value("位置", loc);
        }

        ctx.add_section_spacing();
    }

    /// 渲染替代方案搜尋資訊
    fn render_alt_search(ctx: &mut PdfContext, alt_search: &serde_json::Value) {
        ctx.render_subsection_header("2.2.2 確曾非動物性替代方案");
        if let Some(platforms) = alt_search.get("platforms").and_then(|v| v.as_array()) {
            let platforms_str: Vec<&str> = platforms.iter().filter_map(|p| p.as_str()).collect();
            if !platforms_str.is_empty() {
                ctx.render_label_value("查詢平台", &platforms_str.join(", "));
            }
        }
        if let Some(keywords) = alt_search.get("keywords").and_then(|v| v.as_str()) {
            ctx.render_label_value("關鍵字", keywords);
        }
        if let Some(conclusion) = alt_search.get("conclusion").and_then(|v| v.as_str()) {
            ctx.render_paragraph(conclusion);
        }
    }

    /// 渲染第2節：研究目的
    fn render_section_2(ctx: &mut PdfContext, purpose: &serde_json::Value) {
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
                Self::render_alt_search(ctx, alt_search);
            }
        }
        if let Some(duplicate) = purpose.get("duplicate") {
            ctx.render_subsection_header("2.2.3 是否為重複他人試驗");
            let is_dup = duplicate
                .get("experiment")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            ctx.render_label_value("", if is_dup { "是" } else { "否" });
            if is_dup {
                if let Some(just) = duplicate.get("justification").and_then(|v| v.as_str()) {
                    ctx.render_paragraph(just);
                }
            }
        }
        if let Some(reduction) = purpose.get("reduction") {
            if let Some(design) = reduction.get("design").and_then(|v| v.as_str()) {
                ctx.render_subsection_header("2.3 減量原則");
                ctx.render_paragraph(design);
            }
        }
        ctx.add_section_spacing();
    }

    /// 渲染第3節：試驗物質與對照物質
    fn render_section_3(ctx: &mut PdfContext, items: &serde_json::Value) {
        let use_test_item = items
            .get("use_test_item")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        ctx.force_new_page();
        ctx.render_section_header("3. 試驗物質與對照物質");

        if !use_test_item {
            ctx.render_paragraph("略");
            ctx.add_section_spacing();
            return;
        }

        if let Some(test_items) = items.get("test_items").and_then(|v| v.as_array()) {
            for (i, item) in test_items.iter().enumerate() {
                Self::render_test_item(ctx, i, item);
            }
        }
        if let Some(ctrl_items) = items.get("control_items").and_then(|v| v.as_array()) {
            for (i, item) in ctrl_items.iter().enumerate() {
                ctx.render_subsection_header(&format!("對照物質 #{}", i + 1));
                if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                    ctx.render_label_value("物質名稱", name);
                }
                if let Some(purpose) = item.get("purpose").and_then(|v| v.as_str()) {
                    ctx.render_label_value("用途", purpose);
                }
            }
        }
        ctx.add_section_spacing();
    }

    /// 渲染單一試驗物質
    fn render_test_item(ctx: &mut PdfContext, index: usize, item: &serde_json::Value) {
        ctx.render_subsection_header(&format!("試驗物質 #{}", index + 1));
        if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
            ctx.render_label_value("物質名稱", name);
        }
        if let Some(form) = item.get("form").and_then(|v| v.as_str()) {
            ctx.render_label_value("劑型", form);
        }
        if let Some(purpose) = item.get("purpose").and_then(|v| v.as_str()) {
            ctx.render_label_value("用途", purpose);
        }
        if let Some(storage) = item.get("storage_conditions").and_then(|v| v.as_str()) {
            ctx.render_label_value("儲存條件", storage);
        }
        let is_sterile = item
            .get("is_sterile")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        ctx.render_label_value("無菌", if is_sterile { "是" } else { "否" });
    }

    /// 渲染第4節：研究設計與方法
    fn render_section_4(ctx: &mut PdfContext, design: &serde_json::Value) {
        ctx.force_new_page();
        ctx.render_section_header("4. 研究設計與方法");

        if let Some(procedures) = design.get("procedures").and_then(|v| v.as_str()) {
            ctx.render_subsection_header("動物試驗流程描述");
            ctx.render_paragraph(procedures);
        }
        if let Some(anesthesia) = design.get("anesthesia") {
            let is_under = anesthesia
                .get("is_under_anesthesia")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            ctx.render_label_value("是否於麻醉下進行試驗", if is_under { "是" } else { "否" });
            if let Some(a_type) = anesthesia.get("anesthesia_type").and_then(|v| v.as_str()) {
                ctx.render_label_value("麻醉類型", a_type);
            }
        }
        if let Some(pain) = design.get("pain") {
            if let Some(category) = pain.get("category").and_then(|v| v.as_str()) {
                ctx.render_label_value("疼痛類別", category);
            }
            if let Some(mgmt) = pain.get("management_plan").and_then(|v| v.as_str()) {
                ctx.render_subsection_header("疼痛管理方案");
                ctx.render_paragraph(mgmt);
            }
        }
        if let Some(endpoints) = design.get("endpoints") {
            if let Some(exp_ep) = endpoints.get("experimental_endpoint").and_then(|v| v.as_str()) {
                ctx.render_subsection_header("試驗終點");
                ctx.render_paragraph(exp_ep);
            }
            if let Some(hum_ep) = endpoints.get("humane_endpoint").and_then(|v| v.as_str()) {
                ctx.render_subsection_header("人道終點");
                ctx.render_paragraph(hum_ep);
            }
        }
        ctx.add_section_spacing();
    }

    /// 渲染第5節：相關規範及參考文獻
    fn render_section_5(ctx: &mut PdfContext, guide: &serde_json::Value) {
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

    /// 判斷是否需要手術計畫
    fn needs_surgery(content: &serde_json::Value) -> bool {
        content
            .get("design")
            .and_then(|d| d.get("anesthesia"))
            .and_then(|a| {
                let is_under = a
                    .get("is_under_anesthesia")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let a_type = a
                    .get("anesthesia_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if is_under && (a_type == "survival_surgery" || a_type == "non_survival_surgery") {
                    Some(true)
                } else {
                    None
                }
            })
            .unwrap_or(false)
    }

    /// 渲染手術用藥計畫
    fn render_surgery_drugs(ctx: &mut PdfContext, drugs: &[serde_json::Value]) {
        ctx.render_subsection_header("用藥計畫");
        for drug in drugs.iter() {
            let dn = drug.get("drug_name").and_then(|v| v.as_str()).unwrap_or("-");
            let dose = drug.get("dose").and_then(|v| v.as_str()).unwrap_or("-");
            let route = drug.get("route").and_then(|v| v.as_str()).unwrap_or("-");
            let freq = drug.get("frequency").and_then(|v| v.as_str()).unwrap_or("-");
            ctx.render_label_value(
                "",
                &format!("{}: 劑量{}, 途徑{}, 頻率{}", dn, dose, route, freq),
            );
        }
    }

    /// 渲染第6節：手術計畫書
    fn render_section_6(ctx: &mut PdfContext, surg: &serde_json::Value, content: &serde_json::Value) {
        let has_surgery = Self::needs_surgery(content);
        ctx.force_new_page();
        ctx.render_section_header("6. 手術計畫書");

        if !has_surgery {
            ctx.render_paragraph("略");
            ctx.add_section_spacing();
            return;
        }

        if let Some(st) = surg.get("surgery_type").and_then(|v| v.as_str()) {
            ctx.render_label_value("手術類型", st);
        }
        if let Some(preop) = surg.get("preop_preparation").and_then(|v| v.as_str()) {
            ctx.render_subsection_header("術前準備");
            ctx.render_paragraph(preop);
        }
        if let Some(desc) = surg.get("surgery_description").and_then(|v| v.as_str()) {
            ctx.render_subsection_header("手術描述");
            ctx.render_paragraph(desc);
        }
        if let Some(mon) = surg.get("monitoring").and_then(|v| v.as_str()) {
            ctx.render_subsection_header("監控方式");
            ctx.render_paragraph(mon);
        }
        if let Some(postop) = surg.get("postop_care").and_then(|v| v.as_str()) {
            ctx.render_subsection_header("術後照護");
            ctx.render_paragraph(postop);
        }
        if let Some(drugs) = surg.get("drugs").and_then(|v| v.as_array()) {
            if !drugs.is_empty() {
                Self::render_surgery_drugs(ctx, drugs);
            }
        }
        ctx.add_section_spacing();
    }

    /// 渲染單一動物群組資料
    fn render_animal_group(ctx: &mut PdfContext, index: usize, animal: &serde_json::Value) {
        ctx.render_subsection_header(&format!("動物群組 #{}", index + 1));
        if let Some(sp) = animal.get("species").and_then(|v| v.as_str()) {
            ctx.render_label_value("物種", sp);
        }
        if let Some(st) = animal.get("strain").and_then(|v| v.as_str()) {
            ctx.render_label_value("品系", st);
        }
        if let Some(sx) = animal.get("sex").and_then(|v| v.as_str()) {
            ctx.render_label_value("性別", sx);
        }
        if let Some(num) = animal.get("number").and_then(|v| v.as_i64()) {
            ctx.render_label_value("數量", &num.to_string());
        }

        Self::render_animal_age_weight(ctx, animal);

        if let Some(loc) = animal.get("housing_location").and_then(|v| v.as_str()) {
            ctx.render_label_value("飼養位置", loc);
        }
    }

    /// 渲染動物月齡與體重範圍
    fn render_animal_age_weight(ctx: &mut PdfContext, animal: &serde_json::Value) {
        let age_unlim = animal
            .get("age_unlimited")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if age_unlim {
            ctx.render_label_value("月齡範圍", "不限");
        } else {
            let amin = animal.get("age_min").and_then(|v| v.as_str()).unwrap_or("不限");
            let amax = animal.get("age_max").and_then(|v| v.as_str()).unwrap_or("不限");
            ctx.render_label_value("月齡範圍", &format!("{} ~ {}", amin, amax));
        }

        let wt_unlim = animal
            .get("weight_unlimited")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if wt_unlim {
            ctx.render_label_value("體重範圍", "不限");
        } else {
            let wmin = animal.get("weight_min").and_then(|v| v.as_str()).unwrap_or("不限");
            let wmax = animal.get("weight_max").and_then(|v| v.as_str()).unwrap_or("不限");
            ctx.render_label_value("體重範圍", &format!("{}kg ~ {}kg", wmin, wmax));
        }
    }

    /// 渲染第7節：實驗動物資料
    fn render_section_7(ctx: &mut PdfContext, animals: &serde_json::Value) {
        ctx.force_new_page();
        ctx.render_section_header("7. 實驗動物資料");

        if let Some(animal_list) = animals.get("animals").and_then(|v| v.as_array()) {
            for (i, animal) in animal_list.iter().enumerate() {
                Self::render_animal_group(ctx, i, animal);
            }
        }
        if let Some(total) = animals.get("total_animals").and_then(|v| v.as_i64()) {
            ctx.render_label_value("總動物數", &total.to_string());
        }
        ctx.add_section_spacing();
    }

    /// 渲染第8節：試驗人員資料（表格格式）
    fn render_section_8(ctx: &mut PdfContext, personnel: &[serde_json::Value]) {
        if personnel.is_empty() {
            return;
        }
        ctx.force_new_page();
        ctx.render_section_header("8. 試驗人員資料");

        // 表頭：姓名 | 職位 | 年資 | 工作內容 | 訓練/資格
        let col_defs: &[(&str, f32)] = &[
            ("姓名", 25.0),
            ("職位", 25.0),
            ("年資", 15.0),
            ("工作內容", 40.0),
            ("訓練/資格", 65.0),
        ];
        ctx.render_table_header(col_defs);

        for person in personnel {
            let name = person.get("name").and_then(|v| v.as_str()).unwrap_or("-");
            let position = person.get("position").and_then(|v| v.as_str()).unwrap_or("-");
            let years = person
                .get("years_experience")
                .and_then(|v| v.as_i64())
                .map(|y| format!("{} 年", y))
                .unwrap_or_else(|| "-".to_string());
            let roles_text = Self::format_roles(person);
            let trainings_text = Self::format_trainings_detail(person);

            let row: Vec<(&str, f32)> = vec![
                (name, 25.0),
                (position, 25.0),
                (&years, 15.0),
                (&roles_text, 40.0),
                (&trainings_text, 65.0),
            ];
            ctx.render_table_row(&row);
        }
        ctx.add_section_spacing();
    }

    /// 格式化工作內容代碼為可讀文字
    fn format_roles(person: &serde_json::Value) -> String {
        let roles = match person.get("roles").and_then(|v| v.as_array()) {
            Some(r) => r,
            None => return "-".to_string(),
        };
        let labels: Vec<String> = roles
            .iter()
            .filter_map(|r| r.as_str())
            .map(|code| match code {
                "a" => "a.計畫督導".to_string(),
                "b" => "b.飼養照顧".to_string(),
                "c" => "c.保定".to_string(),
                "d" => "d.麻醉止痛".to_string(),
                "e" => "e.手術".to_string(),
                "f" => "f.手術支援".to_string(),
                "g" => "g.觀察監測".to_string(),
                "h" => "h.安樂死".to_string(),
                "i" => {
                    let other = person
                        .get("roles_other_text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if other.is_empty() {
                        "i.其他".to_string()
                    } else {
                        format!("i.其他({})", other)
                    }
                }
                other => other.to_string(),
            })
            .collect();
        if labels.is_empty() { "-".to_string() } else { labels.join(", ") }
    }

    /// 格式化訓練/資格為詳細文字（含證書編號）
    fn format_trainings_detail(person: &serde_json::Value) -> String {
        let trainings = match person.get("trainings").and_then(|v| v.as_array()) {
            Some(t) => t,
            None => return "-".to_string(),
        };
        let certs = person
            .get("training_certificates")
            .and_then(|v| v.as_array());

        let labels: Vec<String> = trainings
            .iter()
            .filter_map(|t| t.as_str())
            .map(|code| {
                let label = match code {
                    "A" => "A.IACUC訓練班",
                    "B" => "B.IACUC研討會",
                    "C" => "C.輻射安全訓練班",
                    "D" => "D.生醫產業研習會",
                    "E" => "E.動物法規管理班",
                    "F" => {
                        let other = person
                            .get("trainings_other_text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        return if other.is_empty() {
                            "F.其他".to_string()
                        } else {
                            format!("F.其他({})", other)
                        };
                    }
                    other => return other.to_string(),
                };
                // 附加證書編號
                if let Some(cert_arr) = certs {
                    let cert_nos: Vec<&str> = cert_arr
                        .iter()
                        .filter(|c| {
                            c.get("training_code")
                                .and_then(|v| v.as_str())
                                == Some(code)
                        })
                        .filter_map(|c| c.get("certificate_no").and_then(|v| v.as_str()))
                        .filter(|s| !s.is_empty())
                        .collect();
                    if cert_nos.is_empty() {
                        label.to_string()
                    } else {
                        format!("{}({})", label, cert_nos.join("; "))
                    }
                } else {
                    label.to_string()
                }
            })
            .collect();
        if labels.is_empty() { "-".to_string() } else { labels.join(", ") }
    }

    /// 渲染第9節：附件
    fn render_section_9(ctx: &mut PdfContext, attachments: &[serde_json::Value]) {
        if attachments.is_empty() {
            return;
        }
        ctx.force_new_page();
        ctx.render_section_header("9. 附件");
        for (i, att) in attachments.iter().enumerate() {
            if let Some(fname) = att.get("file_name").and_then(|v| v.as_str()) {
                ctx.render_label_value("", &format!("{}. {}", i + 1, fname));
            }
        }
        ctx.add_section_spacing();
    }

    /// 渲染頁尾並輸出 PDF bytes (printpdf 0.9)
    fn render_footer_and_save(mut ctx: PdfContext) -> Result<Vec<u8>> {
        let today = time::now_taiwan().format("%Y-%m-%d").to_string();
        let footer = format!("生成日期: {} | 頁 {} ", today, ctx.page_number);
        ctx.push_text(&footer, 8.0, MARGIN_MM, MARGIN_MM);

        Ok(ctx.save())
    }

    /// 渲染 protocol 各章節內容
    fn render_protocol_sections(ctx: &mut PdfContext, protocol: &ProtocolResponse, content: &serde_json::Value) {
        Self::render_section_1(ctx, protocol, content);

        if let Some(purpose) = content.get("purpose") {
            Self::render_section_2(ctx, purpose);
        }
        if let Some(items) = content.get("items") {
            Self::render_section_3(ctx, items);
        }
        if let Some(design) = content.get("design") {
            Self::render_section_4(ctx, design);
        }
        if let Some(guide) = content.get("guidelines") {
            Self::render_section_5(ctx, guide);
        }
        if let Some(surg) = content.get("surgery") {
            Self::render_section_6(ctx, surg, content);
        }
        if let Some(animals) = content.get("animals") {
            Self::render_section_7(ctx, animals);
        }
        if let Some(personnel) = content.get("personnel").and_then(|v| v.as_array()) {
            Self::render_section_8(ctx, personnel);
        }
        if let Some(attachments) = content.get("attachments").and_then(|v| v.as_array()) {
            Self::render_section_9(ctx, attachments);
        }
    }

    /// 生成 AUP 計畫書 PDF
    pub fn generate_protocol_pdf(protocol: &ProtocolResponse) -> Result<Vec<u8>> {
        let mut ctx = Self::init_pdf_context("AUP Protocol")?;

        Self::render_protocol_title(&mut ctx, &protocol.protocol.title);

        if let Some(ref content) = protocol.protocol.working_content {
            Self::render_protocol_sections(&mut ctx, protocol, content);
        }

        Self::render_footer_and_save(ctx)
    }

    /// 生成動物病歷 PDF
    pub fn generate_medical_pdf(data: &serde_json::Value) -> Result<Vec<u8>> {
        let mut ctx = Self::init_pdf_context("Animal Medical Record")?;

        // ========== 標題 ==========
        ctx.push_text("動物病歷紀錄總表", 20.0, PAGE_WIDTH_MM / 2.0 - 40.0, ctx.y_position);
        ctx.y_position -= SECTION_SPACING_MM * 2.0;

        // 使用共用渲染方法
        Self::render_animal_medical_data(&mut ctx, data);

        Self::render_footer_and_save(ctx)
    }

    /// 共用：渲染動物完整病歷資料（基本資料、體重、疫苗、觀察、手術）
    /// 用於單隻匯出與批次匯出，統一 session-per-page 分頁邏輯
    fn render_animal_medical_data(ctx: &mut PdfContext, data: &serde_json::Value) {
        // ========== 1. 動物基本資料 ==========
        if let Some(animal) = data.get("animal") {
            ctx.render_section_header("1. 動物基本資料");
            let ear_tag = animal
                .get("ear_tag")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let breed = animal.get("breed").and_then(|v| v.as_str()).unwrap_or("-");
            let gender = animal.get("gender").and_then(|v| v.as_str()).unwrap_or("-");
            let iacuc_no = animal
                .get("iacuc_no")
                .and_then(|v| v.as_str())
                .unwrap_or("未分配");

            ctx.render_label_value("耳號", ear_tag);
            ctx.render_label_value("計畫編號", iacuc_no);
            ctx.render_label_value("品種", breed);
            ctx.render_label_value("性別", if gender == "male" { "公" } else { "母" });

            if let Some(entry_date) = animal.get("entry_date").and_then(|v| v.as_str()) {
                ctx.render_label_value("進場日期", entry_date);
            }
            if let Some(birth_date) = animal.get("birth_date").and_then(|v| v.as_str()) {
                ctx.render_label_value("出生日期", birth_date);
            }
            if let Some(loc) = animal.get("pen_location").and_then(|v| v.as_str()) {
                ctx.render_label_value("欄位編號", loc);
            }
            ctx.add_section_spacing();
        }

        // ========== 2. 體重紀錄 ==========
        if let Some(weights) = data.get("weights").and_then(|v| v.as_array()) {
            if !weights.is_empty() {
                ctx.render_section_header("2. 體重紀錄");
                for w in weights {
                    let date = w
                        .get("measure_date")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    let weight = w
                        .get("weight")
                        .map(|v| v.to_string())
                        .unwrap_or("-".to_string());
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
                    let date = v
                        .get("administered_date")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    let vaccine = v.get("vaccine").and_then(|v| v.as_str()).unwrap_or("-");
                    let dose = v
                        .get("deworming_dose")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    ctx.render_label_value(
                        &format!("日期: {}", date),
                        &format!("項目: {}, 劑量: {}", vaccine, dose),
                    );
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
                    let date = obs
                        .get("event_date")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    let rtype = obs
                        .get("record_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
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
                    let date = surg
                        .get("surgery_date")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    let site = surg
                        .get("surgery_site")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");

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

    /// 生成計畫下所有動物病歷 PDF
    pub fn generate_project_medical_pdf(
        iacuc_no: &str,
        animals_data: &serde_json::Value,
    ) -> Result<Vec<u8>> {
        let mut ctx = Self::init_pdf_context(&format!("Project Medical Export - {}", iacuc_no))?;

        // ========== 封面標題 ==========
        let title = format!("計畫病歷匯出總表 - {}", iacuc_no);
        ctx.push_text(&title, 20.0, MARGIN_MM, ctx.y_position);
        ctx.y_position -= SECTION_SPACING_MM * 2.0;

        // 顯示匯出日期
        let today = time::now_taiwan().format("%Y-%m-%d").to_string();
        ctx.render_label_value("匯出日期", &today);

        if let Some(animals) = animals_data.get("animals").and_then(|v| v.as_array()) {
            ctx.render_label_value("動物總數", &animals.len().to_string());
            ctx.add_section_spacing();

            // 封面：動物清單摘要
            ctx.render_subsection_header("動物清單");
            for (i, animal_data) in animals.iter().enumerate() {
                if let Some(animal) = animal_data.get("animal") {
                    let ear_tag = animal
                        .get("ear_tag")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-");
                    let breed = animal.get("breed").and_then(|v| v.as_str()).unwrap_or("-");
                    let gender = animal.get("gender").and_then(|v| v.as_str()).unwrap_or("-");
                    let gender_label = if gender == "male" { "公" } else { "母" };
                    ctx.render_label_value(
                        "",
                        &format!("{}. {} ({}, {})", i + 1, ear_tag, breed, gender_label),
                    );
                }
            }

            // 每隻動物獨立分頁渲染完整病歷
            for animal_data in animals.iter() {
                ctx.add_new_page();
                Self::render_animal_medical_data(&mut ctx, animal_data);
            }
        }

        Self::render_footer_and_save(ctx)
    }

    // ============================================
    // 倉庫現況報表 PDF
    // ============================================

    /// 生成倉庫現況報表 PDF
    pub fn generate_warehouse_report(data: &WarehouseReportData) -> Result<Vec<u8>> {
        let mut ctx = Self::init_pdf_context("Warehouse Report")?;

        Self::render_warehouse_title(&mut ctx, data);
        Self::render_warehouse_summary(&mut ctx, data);
        Self::render_warehouse_layout(&mut ctx, data);
        Self::render_warehouse_inventory(&mut ctx, data);

        Self::render_footer_and_save(ctx)
    }

    /// 渲染倉庫報表標題
    fn render_warehouse_title(ctx: &mut PdfContext, data: &WarehouseReportData) {
        ctx.push_text("倉庫現況報表", 22.0, PAGE_WIDTH_MM / 2.0 - 30.0, ctx.y_position);
        ctx.y_position -= SECTION_SPACING_MM * 2.0;

        let generated = data
            .generated_at
            .format("%Y-%m-%d %H:%M")
            .to_string();
        let info_line = format!(
            "倉庫代碼：{}　｜　倉庫名稱：{}　｜　報表產出時間：{}",
            data.warehouse.code, data.warehouse.name, generated
        );
        ctx.push_text(&info_line, 9.0, MARGIN_MM, ctx.y_position);
        ctx.y_position -= LINE_HEIGHT_MM;
        ctx.add_section_spacing();
    }

    /// 渲染倉庫摘要統計
    fn render_warehouse_summary(ctx: &mut PdfContext, data: &WarehouseReportData) {
        ctx.render_section_header("一、摘要統計");
        let summary_line = format!(
            "儲位總數：{}　｜　使用中儲位：{}　｜　庫存品項總數：{}",
            data.summary.total_locations,
            data.summary.active_locations,
            data.summary.total_inventory_items
        );
        ctx.push_text(&summary_line, 10.0, MARGIN_MM + 5.0, ctx.y_position);
        ctx.y_position -= LINE_HEIGHT_MM;
        ctx.add_section_spacing();
    }

    /// 渲染倉庫佈局圖
    fn render_warehouse_layout(ctx: &mut PdfContext, data: &WarehouseReportData) {
        if data.locations.is_empty() {
            return;
        }

        ctx.render_section_header("二、儲位分佈圖");

        let max_col = data
            .locations
            .iter()
            .map(|l| l.col_index + l.width)
            .max()
            .unwrap_or(12) as f32;
        let max_row = data
            .locations
            .iter()
            .map(|l| l.row_index + l.height)
            .max()
            .unwrap_or(6) as f32;

        let available_width = PAGE_WIDTH_MM - 2.0 * MARGIN_MM;
        let layout_height = (available_width * max_row / max_col).min(120.0);
        let cell_w = available_width / max_col;
        let cell_h = layout_height / max_row;

        // 佈局圖頂部 Y 座標
        let layout_top = ctx.y_position;

        for loc in &data.locations {
            let x = MARGIN_MM + loc.col_index as f32 * cell_w;
            // printpdf Y 軸從下到上，row_index 從上到下
            let y = layout_top - (loc.row_index as f32 + loc.height as f32) * cell_h;
            let w = loc.width as f32 * cell_w;
            let h = loc.height as f32 * cell_h;

            let (r, g, b) = Self::parse_hex_color(
                loc.color.as_deref().unwrap_or("#3b82f6"),
                loc.location_type.as_str(),
            );
            ctx.draw_filled_rect(x, y, w, h, r, g, b);

            // 在方塊內繪製名稱（優先）或代碼
            let display_text = loc.name.as_deref().unwrap_or(&loc.code);
            let text_y = y + h / 2.0 - 1.5;
            let text_x = x + 1.0;
            ctx.current_ops.push(Op::SetFillColor {
                col: Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)),
            });
            ctx.push_text(display_text, 7.0, text_x, text_y);
        }

        // 重設填充顏色為黑色（文字用）
        ctx.current_ops.push(Op::SetFillColor {
            col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
        });
        ctx.y_position = layout_top - layout_height - SECTION_SPACING_MM;
    }

    /// 渲染倉庫庫存明細
    fn render_warehouse_inventory(ctx: &mut PdfContext, data: &WarehouseReportData) {
        ctx.force_new_page();
        ctx.render_section_header("三、各儲位庫存明細");

        // 欄寬定義
        let col_defs: &[(&str, f32)] = &[
            ("產品名稱", 50.0),
            ("SKU", 30.0),
            ("數量", 20.0),
            ("單位", 15.0),
            ("批號", 25.0),
            ("效期", 30.0),
        ];

        for loc in &data.locations {
            // 跳過建築結構
            if STRUCTURE_TYPES.contains(&loc.location_type.as_str()) {
                continue;
            }

            ctx.check_page_break(LINE_HEIGHT_MM * 4.0);

            let loc_title = match &loc.name {
                Some(name) => format!("【{}】{}", loc.code, name),
                None => format!("【{}】", loc.code),
            };
            let capacity_info = match loc.capacity {
                Some(cap) if cap > 0 => {
                    format!("（容量: {}/{}）", loc.current_count, cap)
                }
                _ => format!("（目前: {}）", loc.current_count),
            };
            ctx.render_subsection_header(&format!("{} {}", loc_title, capacity_info));

            if loc.inventory.is_empty() {
                ctx.render_label_value("", "（無庫存）");
                ctx.y_position -= LINE_HEIGHT_MM * 0.5;
                continue;
            }

            ctx.render_table_header(col_defs);

            for item in &loc.inventory {
                let qty_str = item.on_hand_qty.trunc().to_string();
                let batch = item.batch_no.as_deref().unwrap_or("-");
                let expiry = item
                    .expiry_date
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "-".to_string());

                let row: Vec<(&str, f32)> = vec![
                    (&item.product_name, 50.0),
                    (&item.product_sku, 30.0),
                    (&qty_str, 20.0),
                    (&item.base_uom, 15.0),
                    (batch, 25.0),
                    (&expiry, 30.0),
                ];
                ctx.render_table_row(&row);
            }
            ctx.y_position -= LINE_HEIGHT_MM * 0.5;
        }
    }

    /// 解析 hex 色碼為 RGB (0.0~1.0)
    fn parse_hex_color(hex: &str, location_type: &str) -> (f32, f32, f32) {
        // 建築結構用特定顏色
        match location_type {
            "wall" => return (0.6, 0.6, 0.6),
            "door" => return (0.55, 0.35, 0.17),
            "window" => return (0.7, 0.85, 0.95),
            _ => {}
        }

        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            }
        }
        // fallback 藍色
        (0.23, 0.51, 0.96)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        StorageLocationInventoryItem, StorageLocationWithInventory, Warehouse,
        WarehouseReportData, WarehouseReportSummary,
    };
    use chrono::Utc;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[test]
    fn test_generate_warehouse_report_pdf() {
        let data = WarehouseReportData {
            warehouse: Warehouse {
                id: Uuid::new_v4(),
                code: "WH01".to_string(),
                name: "準備室".to_string(),
                address: Some("台中市".to_string()),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            summary: WarehouseReportSummary {
                total_locations: 3,
                active_locations: 3,
                total_capacity: 10,
                total_current_count: 5,
                total_inventory_items: 2,
            },
            locations: vec![
                StorageLocationWithInventory {
                    id: Uuid::new_v4(),
                    code: "A01".to_string(),
                    name: Some("儲物架".to_string()),
                    location_type: "shelf".to_string(),
                    row_index: 0,
                    col_index: 0,
                    width: 2,
                    height: 1,
                    capacity: Some(5),
                    current_count: 3,
                    color: Some("#3b82f6".to_string()),
                    is_active: true,
                    inventory: vec![StorageLocationInventoryItem {
                        id: Uuid::new_v4(),
                        storage_location_id: Uuid::new_v4(),
                        product_id: Uuid::new_v4(),
                        product_sku: "SKU001".to_string(),
                        product_name: "飼料A".to_string(),
                        on_hand_qty: Decimal::new(10, 0),
                        base_uom: "kg".to_string(),
                        batch_no: Some("B001".to_string()),
                        expiry_date: None,
                        updated_at: Utc::now(),
                    }],
                },
                StorageLocationWithInventory {
                    id: Uuid::new_v4(),
                    code: "A02".to_string(),
                    name: None,
                    location_type: "shelf".to_string(),
                    row_index: 0,
                    col_index: 2,
                    width: 1,
                    height: 1,
                    capacity: None,
                    current_count: 0,
                    color: None,
                    is_active: true,
                    inventory: vec![],
                },
            ],
            generated_at: Utc::now(),
        };

        let result = PdfService::generate_warehouse_report(&data);
        assert!(result.is_ok(), "PDF generation should succeed");
    }
}
