use eframe::egui::{Color32, FontFamily, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use image::RgbaImage;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Rectangle,
    Ellipse,
    Arrow,
    Pen,
    Step,
    Text,
    Mosaic,
    Highlighter,
}

#[allow(dead_code)]
impl Tool {
    pub fn name_tr(&self) -> &'static str {
        match self {
            Tool::Select => "Seç / Taşı",
            Tool::Rectangle => "Dikdörtgen",
            Tool::Ellipse => "Daire",
            Tool::Arrow => "Ok",
            Tool::Pen => "Kalem",
            Tool::Step => "Adım (1,2,3)",
            Tool::Text => "Metin",
            Tool::Mosaic => "Buzlama / Mozaik",
            Tool::Highlighter => "Vurgulayıcı",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Tool::Select => "↖",
            Tool::Rectangle => "▭",
            Tool::Ellipse => "◯",
            Tool::Arrow => "➔",
            Tool::Pen => "✎",
            Tool::Step => "①",
            Tool::Text => "T",
            Tool::Mosaic => "▦",
            Tool::Highlighter => "▨",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            Tool::Select => "V",
            Tool::Rectangle => "R",
            Tool::Ellipse => "O",
            Tool::Arrow => "A",
            Tool::Pen => "P",
            Tool::Step => "N",
            Tool::Text => "T",
            Tool::Mosaic => "B",
            Tool::Highlighter => "H",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AnnotationItem {
    Rectangle {
        rect: Rect,
        stroke_color: Color32,
        stroke_width: f32,
        fill: bool,
    },
    Ellipse {
        rect: Rect,
        stroke_color: Color32,
        stroke_width: f32,
        fill: bool,
    },
    Arrow {
        start: Pos2,
        end: Pos2,
        color: Color32,
        width: f32,
    },
    Pen {
        points: Vec<Pos2>,
        color: Color32,
        width: f32,
    },
    StepBadge {
        center: Pos2,
        number: usize,
        bg_color: Color32,
        text_color: Color32,
        radius: f32,
    },
    Text {
        pos: Pos2,
        content: String,
        color: Color32,
        font_size: f32,
    },
    Mosaic {
        rect: Rect,
        blocks: Vec<(Rect, Color32)>,
    },
    Highlighter {
        points: Vec<Pos2>,
        color: Color32,
        width: f32,
    },
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct AnnotationState {
    pub active_tool: Tool,
    pub stroke_color: Color32,
    pub stroke_width: f32,
    pub fill_shape: bool,
    pub items: Vec<AnnotationItem>,
    pub next_step: usize,
    pub active_text_pos: Option<Pos2>,
    pub active_text_buffer: String,
}

impl Default for AnnotationState {
    fn default() -> Self {
        Self {
            active_tool: Tool::Select,
            stroke_color: Color32::from_rgb(255, 59, 48), // Apple Red
            stroke_width: 3.5,
            fill_shape: false,
            items: Vec::new(),
            next_step: 1,
            active_text_pos: None,
            active_text_buffer: String::new(),
        }
    }
}

impl AnnotationState {
    pub fn undo(&mut self) {
        if let Some(removed) = self.items.pop() {
            if let AnnotationItem::StepBadge { .. } = removed {
                if self.next_step > 1 {
                    self.next_step -= 1;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.next_step = 1;
        self.active_text_pos = None;
        self.active_text_buffer.clear();
    }

    pub fn add_step(&mut self, center: Pos2) {
        let num = self.next_step;
        self.next_step += 1;
        self.items.push(AnnotationItem::StepBadge {
            center,
            number: num,
            bg_color: self.stroke_color,
            text_color: Color32::WHITE,
            radius: 14.0,
        });
    }

    pub fn generate_mosaic_blocks(
        rect: Rect,
        bg_image: &RgbaImage,
        scale_x: f32,
        scale_y: f32,
    ) -> Vec<(Rect, Color32)> {
        let block_size_pt = 12.0;
        let mut blocks = Vec::new();

        let norm_rect = Rect::from_two_pos(
            Pos2::new(rect.min.x.min(rect.max.x), rect.min.y.min(rect.max.y)),
            Pos2::new(rect.min.x.max(rect.max.x), rect.min.y.max(rect.max.y)),
        );

        let img_w = bg_image.width() as i32;
        let img_h = bg_image.height() as i32;

        let mut y = norm_rect.min.y;
        while y < norm_rect.max.y {
            let next_y = (y + block_size_pt).min(norm_rect.max.y);
            let mut x = norm_rect.min.x;
            while x < norm_rect.max.x {
                let next_x = (x + block_size_pt).min(norm_rect.max.x);
                let block_rect = Rect::from_min_max(Pos2::new(x, y), Pos2::new(next_x, next_y));

                // Sample center pixel from background image
                let mid_x = ((x + next_x) * 0.5 * scale_x) as i32;
                let mid_y = ((y + next_y) * 0.5 * scale_y) as i32;

                let px = mid_x.clamp(0, img_w - 1) as u32;
                let py = mid_y.clamp(0, img_h - 1) as u32;
                let rgba = bg_image.get_pixel(px, py);

                let color = Color32::from_rgb(rgba[0], rgba[1], rgba[2]);
                blocks.push((block_rect, color));

                x = next_x;
            }
            y = next_y;
        }

        blocks
    }

    pub fn paint_item(painter: &Painter, item: &AnnotationItem) {
        match item {
            AnnotationItem::Rectangle {
                rect,
                stroke_color,
                stroke_width,
                fill,
            } => {
                let stroke = Stroke::new(*stroke_width, *stroke_color);
                let fill_c = if *fill {
                    stroke_color.gamma_multiply(0.25)
                } else {
                    Color32::TRANSPARENT
                };
                painter.rect(*rect, 4.0, fill_c, stroke, egui::StrokeKind::Middle);
            }
            AnnotationItem::Ellipse {
                rect,
                stroke_color,
                stroke_width,
                fill,
            } => {
                let stroke = Stroke::new(*stroke_width, *stroke_color);
                let fill_c = if *fill {
                    stroke_color.gamma_multiply(0.25)
                } else {
                    Color32::TRANSPARENT
                };
                // Draw smooth approximated ellipse
                let center = rect.center();
                let rx = rect.width() * 0.5;
                let ry = rect.height() * 0.5;
                let segments = 48;
                let mut points = Vec::with_capacity(segments);
                for i in 0..segments {
                    let theta = std::f32::consts::TAU * (i as f32) / (segments as f32);
                    points.push(Pos2::new(
                        center.x + rx * theta.cos(),
                        center.y + ry * theta.sin(),
                    ));
                }
                if *fill {
                    painter.add(egui::Shape::convex_polygon(points.clone(), fill_c, Stroke::NONE));
                }
                painter.add(egui::Shape::closed_line(points, stroke));
            }
            AnnotationItem::Arrow {
                start,
                end,
                color,
                width,
            } => {
                let vec = *end - *start;
                let len = vec.length();
                if len < 3.0 {
                    return;
                }
                let dir = vec / len;
                let normal = Vec2::new(-dir.y, dir.x);

                let head_len = (width * 4.5).clamp(14.0, 32.0).min(len * 0.7);
                let head_width = head_len * 0.6;

                let line_end = *end - dir * (head_len * 0.5);

                // Main line
                painter.line_segment([*start, line_end], Stroke::new(*width, *color));

                // Arrow head triangle
                let tip = *end;
                let left = *end - dir * head_len + normal * head_width;
                let right = *end - dir * head_len - normal * head_width;

                painter.add(egui::Shape::convex_polygon(
                    vec![tip, left, right],
                    *color,
                    Stroke::NONE,
                ));
            }
            AnnotationItem::Pen {
                points,
                color,
                width,
            } => {
                if points.len() >= 2 {
                    painter.add(egui::Shape::line(points.clone(), Stroke::new(*width, *color)));
                }
            }
            AnnotationItem::Highlighter {
                points,
                color,
                width,
            } => {
                if points.len() >= 2 {
                    let high_color = color.gamma_multiply(0.4);
                    painter.add(egui::Shape::line(
                        points.clone(),
                        Stroke::new(*width * 3.5, high_color),
                    ));
                }
            }
            AnnotationItem::StepBadge {
                center,
                number,
                bg_color,
                text_color,
                radius,
            } => {
                // Drop shadow
                painter.circle_filled(*center + Vec2::new(0.0, 1.5), *radius, Color32::from_black_alpha(80));
                // Outer ring
                painter.circle_stroke(*center, *radius + 1.5, Stroke::new(2.0, Color32::WHITE));
                // Fill circle
                painter.circle_filled(*center, *radius, *bg_color);

                // Centered text
                let font_id = FontId::new(*radius * 1.15, FontFamily::Proportional);
                let num_str = number.to_string();
                painter.text(
                    *center,
                    egui::Align2::CENTER_CENTER,
                    num_str,
                    font_id,
                    *text_color,
                );
            }
            AnnotationItem::Text {
                pos,
                content,
                color,
                font_size,
            } => {
                let font_id = FontId::new(*font_size, FontFamily::Proportional);
                // Subtle shadow for legibility
                painter.text(
                    *pos + Vec2::new(1.0, 1.0),
                    egui::Align2::LEFT_TOP,
                    content,
                    font_id.clone(),
                    Color32::from_black_alpha(150),
                );
                painter.text(*pos, egui::Align2::LEFT_TOP, content, font_id, *color);
            }
            AnnotationItem::Mosaic { blocks, .. } => {
                for (rect, color) in blocks {
                    painter.rect_filled(*rect, 0.0, *color);
                    painter.rect_stroke(
                        *rect,
                        0.0,
                        Stroke::new(0.5, Color32::from_white_alpha(20)),
                        egui::StrokeKind::Middle,
                    );
                }
            }
        }
    }
}
