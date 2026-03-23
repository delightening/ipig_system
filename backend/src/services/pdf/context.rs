// PDF 渲染上下文 - 多頁面排版支援 (printpdf 0.9 Op-based API)

use printpdf::*;

/// PDF 頁面配置常數
pub(crate) const PAGE_WIDTH_MM: f32 = 210.0;  // A4 寬度
pub(crate) const PAGE_HEIGHT_MM: f32 = 297.0; // A4 高度
pub(crate) const MARGIN_MM: f32 = 20.0;
pub(crate) const LINE_HEIGHT_MM: f32 = 6.0;
pub(crate) const SECTION_SPACING_MM: f32 = 10.0;
pub(crate) const MIN_Y_BEFORE_PAGE_BREAK: f32 = 40.0;

/// PDF rendering context for multi-page support (printpdf 0.9)
///
/// 以 Op 操作列表取代舊版的 PdfLayerReference 直接操作
pub(crate) struct PdfContext {
    pub(crate) doc: PdfDocument,
    pub(crate) font: PdfFontHandle,
    completed_pages: Vec<Vec<Op>>,
    pub(crate) current_ops: Vec<Op>,
    pub(crate) y_position: f32,
    pub(crate) page_number: i32,
}

impl PdfContext {
    pub(crate) fn new(doc: PdfDocument, font: PdfFontHandle) -> Self {
        Self {
            doc,
            font,
            completed_pages: Vec::new(),
            current_ops: Vec::new(),
            y_position: PAGE_HEIGHT_MM - MARGIN_MM,
            page_number: 1,
        }
    }

    /// 將文字渲染操作推入當前頁面的 Op 列表
    pub(crate) fn push_text(&mut self, text: &str, size: f32, x_mm: f32, y_mm: f32) {
        let pos = Point {
            x: Mm(x_mm).into(),
            y: Mm(y_mm).into(),
        };
        self.current_ops.extend([
            Op::StartTextSection,
            Op::SetFont {
                font: self.font.clone(),
                size: Pt(size),
            },
            Op::SetTextCursor { pos },
            Op::ShowText {
                items: vec![TextItem::Text(text.to_string())],
            },
            Op::EndTextSection,
        ]);
    }

    pub(crate) fn check_page_break(&mut self, required_space: f32) {
        if self.y_position - required_space < MIN_Y_BEFORE_PAGE_BREAK {
            self.add_new_page();
        }
    }

    pub(crate) fn add_new_page(&mut self) {
        self.page_number += 1;
        let ops = std::mem::take(&mut self.current_ops);
        self.completed_pages.push(ops);
        self.y_position = PAGE_HEIGHT_MM - MARGIN_MM;
    }

    pub(crate) fn force_new_page(&mut self) {
        if self.y_position < PAGE_HEIGHT_MM - MARGIN_MM - 10.0 {
            self.add_new_page();
        }
    }

    pub(crate) fn render_section_header(&mut self, text: &str) {
        self.check_page_break(LINE_HEIGHT_MM * 3.0);
        self.push_text(text, 14.0, MARGIN_MM, self.y_position);
        self.y_position -= LINE_HEIGHT_MM * 1.5;
    }

    pub(crate) fn render_subsection_header(&mut self, text: &str) {
        self.check_page_break(LINE_HEIGHT_MM * 2.0);
        self.push_text(text, 11.0, MARGIN_MM, self.y_position);
        self.y_position -= LINE_HEIGHT_MM;
    }

    pub(crate) fn render_label_value(&mut self, label: &str, value: &str) {
        self.check_page_break(LINE_HEIGHT_MM);
        let text = if label.is_empty() {
            value.to_string()
        } else {
            format!("{}：{}", label, value)
        };
        self.push_text(&text, 10.0, MARGIN_MM + 5.0, self.y_position);
        self.y_position -= LINE_HEIGHT_MM;
    }

    pub(crate) fn render_paragraph(&mut self, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        let line_width = 45;

        for chunk in chars.chunks(line_width) {
            self.check_page_break(LINE_HEIGHT_MM);
            let line: String = chunk.iter().collect();
            self.push_text(&line, 10.0, MARGIN_MM + 5.0, self.y_position);
            self.y_position -= LINE_HEIGHT_MM;
        }
        self.y_position -= LINE_HEIGHT_MM * 0.5;
    }

    pub(crate) fn add_section_spacing(&mut self) {
        self.y_position -= SECTION_SPACING_MM;
    }

    /// 繪製填色矩形（座標為 mm，左下角為原點）
    pub(crate) fn draw_filled_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill_r: f32,
        fill_g: f32,
        fill_b: f32,
    ) {
        let points = vec![
            LinePoint { p: Point::new(Mm(x), Mm(y)), bezier: false },
            LinePoint { p: Point::new(Mm(x + w), Mm(y)), bezier: false },
            LinePoint { p: Point::new(Mm(x + w), Mm(y + h)), bezier: false },
            LinePoint { p: Point::new(Mm(x), Mm(y + h)), bezier: false },
        ];
        let polygon = Polygon {
            rings: vec![PolygonRing { points }],
            mode: PaintMode::FillStroke,
            winding_order: WindingOrder::NonZero,
        };
        self.current_ops.extend([
            Op::SetFillColor {
                col: Color::Rgb(Rgb::new(fill_r, fill_g, fill_b, None)),
            },
            Op::SetOutlineColor {
                col: Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)),
            },
            Op::SetOutlineThickness { pt: Pt(0.3) },
            Op::DrawPolygon { polygon },
        ]);
    }

    /// 重設填充顏色為黑色（用於文字渲染前）
    fn reset_text_color(&mut self) {
        self.current_ops.push(Op::SetFillColor {
            col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
        });
    }

    /// 渲染表格標題行（灰底黑字）
    pub(crate) fn render_table_header(&mut self, cols: &[(&str, f32)]) {
        self.check_page_break(LINE_HEIGHT_MM * 2.0);
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
        self.reset_text_color();
        let mut x = MARGIN_MM + 1.0;
        let y = self.y_position;
        for &(text, col_width) in cols {
            self.push_text(text, 9.0, x, y);
            x += col_width;
        }
        self.y_position -= LINE_HEIGHT_MM;
    }

    /// 渲染表格資料行（黑字）
    pub(crate) fn render_table_row(&mut self, cols: &[(&str, f32)]) {
        self.check_page_break(LINE_HEIGHT_MM);
        self.reset_text_color();
        let mut x = MARGIN_MM + 1.0;
        let y = self.y_position;
        for &(text, col_width) in cols {
            let max_chars = (col_width as usize) / 2;
            let display: String = if text.chars().count() > max_chars {
                text.chars().take(max_chars.saturating_sub(1)).collect::<String>() + "…"
            } else {
                text.to_string()
            };
            self.push_text(&display, 8.0, x, y);
            x += col_width;
        }
        self.y_position -= LINE_HEIGHT_MM;
    }

    /// 將所有頁面組裝為 PDF 並輸出 bytes
    pub(crate) fn save(mut self) -> Vec<u8> {
        self.completed_pages.push(self.current_ops);

        let pages: Vec<PdfPage> = self
            .completed_pages
            .into_iter()
            .map(|ops| PdfPage::new(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), ops))
            .collect();

        let mut warnings = Vec::new();
        self.doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut warnings)
    }
}
