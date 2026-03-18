// PDF 渲染上下文 - 多頁面排版支援

use printpdf::path::{PaintMode, WindingOrder};
use printpdf::*;

/// PDF 頁面配置常數
pub(crate) const PAGE_WIDTH_MM: f32 = 210.0;  // A4 寬度
pub(crate) const PAGE_HEIGHT_MM: f32 = 297.0; // A4 高度
pub(crate) const MARGIN_MM: f32 = 20.0;
pub(crate) const LINE_HEIGHT_MM: f32 = 6.0;
pub(crate) const SECTION_SPACING_MM: f32 = 10.0;
pub(crate) const MIN_Y_BEFORE_PAGE_BREAK: f32 = 40.0;

/// PDF rendering context for multi-page support
pub(crate) struct PdfContext {
    pub(crate) doc: PdfDocumentReference,
    pub(crate) font: IndirectFontRef,
    pub(crate) current_layer: PdfLayerReference,
    pub(crate) y_position: f32,
    pub(crate) page_number: i32,
}

impl PdfContext {
    pub(crate) fn new(doc: PdfDocumentReference, font: IndirectFontRef, layer: PdfLayerReference) -> Self {
        Self {
            doc,
            font,
            current_layer: layer,
            y_position: PAGE_HEIGHT_MM - MARGIN_MM,
            page_number: 1,
        }
    }

    pub(crate) fn check_page_break(&mut self, required_space: f32) {
        if self.y_position - required_space < MIN_Y_BEFORE_PAGE_BREAK {
            self.add_new_page();
        }
    }

    pub(crate) fn add_new_page(&mut self) {
        self.page_number += 1;
        let (page, layer) = self.doc.add_page(
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            format!("Page {}", self.page_number)
        );
        self.current_layer = self.doc.get_page(page).get_layer(layer);
        self.y_position = PAGE_HEIGHT_MM - MARGIN_MM;
    }

    pub(crate) fn force_new_page(&mut self) {
        if self.y_position < PAGE_HEIGHT_MM - MARGIN_MM - 10.0 {
            self.add_new_page();
        }
    }

    pub(crate) fn render_section_header(&mut self, text: &str) {
        self.check_page_break(LINE_HEIGHT_MM * 3.0);
        self.current_layer.use_text(text, 14.0, Mm(MARGIN_MM), Mm(self.y_position), &self.font);
        self.y_position -= LINE_HEIGHT_MM * 1.5;
    }

    pub(crate) fn render_subsection_header(&mut self, text: &str) {
        self.check_page_break(LINE_HEIGHT_MM * 2.0);
        self.current_layer.use_text(text, 11.0, Mm(MARGIN_MM), Mm(self.y_position), &self.font);
        self.y_position -= LINE_HEIGHT_MM;
    }

    pub(crate) fn render_label_value(&mut self, label: &str, value: &str) {
        self.check_page_break(LINE_HEIGHT_MM);
        let text = if label.is_empty() { value.to_string() } else { format!("{}：{}", label, value) };
        self.current_layer.use_text(&text, 10.0, Mm(MARGIN_MM + 5.0), Mm(self.y_position), &self.font);
        self.y_position -= LINE_HEIGHT_MM;
    }

    pub(crate) fn render_paragraph(&mut self, text: &str) {
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

    pub(crate) fn add_section_spacing(&mut self) {
        self.y_position -= SECTION_SPACING_MM;
    }

    /// 繪製填色矩形（座標為 mm，左下角為原點）
    pub(crate) fn draw_filled_rect(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill_r: f32,
        fill_g: f32,
        fill_b: f32,
    ) {
        let points = vec![
            (Point::new(Mm(x), Mm(y)), false),
            (Point::new(Mm(x + w), Mm(y)), false),
            (Point::new(Mm(x + w), Mm(y + h)), false),
            (Point::new(Mm(x), Mm(y + h)), false),
        ];
        let polygon = Polygon {
            rings: vec![points],
            mode: PaintMode::FillStroke,
            winding_order: WindingOrder::NonZero,
        };
        self.current_layer
            .set_fill_color(Color::Rgb(Rgb::new(fill_r, fill_g, fill_b, None)));
        self.current_layer
            .set_outline_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
        self.current_layer
            .set_outline_thickness(0.3);
        self.current_layer.add_polygon(polygon);
    }

    /// 重設填充顏色為黑色（用於文字渲染前）
    fn reset_text_color(&self) {
        self.current_layer
            .set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    }

    /// 渲染表格標題行（灰底黑字）
    pub(crate) fn render_table_header(&mut self, cols: &[(&str, f32)]) {
        self.check_page_break(LINE_HEIGHT_MM * 2.0);
        // 繪製灰色背景
        let total_width: f32 = cols.iter().map(|(_, w)| w).sum();
        self.draw_filled_rect(
            MARGIN_MM,
            self.y_position - LINE_HEIGHT_MM + 1.0,
            total_width,
            LINE_HEIGHT_MM,
            0.85,
            0.85,
            0.85,
        );
        // 重設為黑色再渲染文字
        self.reset_text_color();
        let mut x = MARGIN_MM + 1.0;
        for &(text, col_width) in cols {
            self.current_layer
                .use_text(text, 9.0, Mm(x), Mm(self.y_position), &self.font);
            x += col_width;
        }
        self.y_position -= LINE_HEIGHT_MM;
    }

    /// 渲染表格資料行（黑字）
    pub(crate) fn render_table_row(&mut self, cols: &[(&str, f32)]) {
        self.check_page_break(LINE_HEIGHT_MM);
        self.reset_text_color();
        let mut x = MARGIN_MM + 1.0;
        for &(text, col_width) in cols {
            // 截斷超長文字
            let max_chars = (col_width as usize) / 2;
            let display: String = if text.chars().count() > max_chars {
                text.chars().take(max_chars.saturating_sub(1)).collect::<String>() + "…"
            } else {
                text.to_string()
            };
            self.current_layer
                .use_text(&display, 8.0, Mm(x), Mm(self.y_position), &self.font);
            x += col_width;
        }
        self.y_position -= LINE_HEIGHT_MM;
    }
}
