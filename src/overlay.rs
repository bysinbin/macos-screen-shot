use crate::annotation::{AnnotationItem, AnnotationState, Tool};
use crate::capture::CapturedScreen;
use crate::export::{bake_and_crop, copy_to_clipboard, default_desktop_filename, save_to_file};
use crate::magnifier::Magnifier;
use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontFamily, FontId, Key, Pos2, Rect, Stroke,
    TextureHandle, Vec2,
};
use image::RgbaImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandlePosition {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

pub struct OverlayApp {
    captured: CapturedScreen,
    texture: Option<TextureHandle>,
    selection: Option<Rect>,
    is_dragging_new: bool,
    drag_start: Option<Pos2>,
    active_handle: Option<HandlePosition>,
    is_moving_selection: bool,
    move_drag_start: Option<Pos2>,
    selection_start_rect: Option<Rect>,

    // Annotation state
    pub annotations: AnnotationState,
    drawing_start: Option<Pos2>,
    pen_points: Vec<Pos2>,

    // Notification toast
    toast_message: Option<(String, std::time::Instant)>,

    // Trigger pin window
    pub pin_requested: Option<RgbaImage>,
    pub is_finished: bool,
}

impl OverlayApp {
    pub fn new(captured: CapturedScreen) -> Self {
        Self {
            captured,
            texture: None,
            selection: None,
            is_dragging_new: false,
            drag_start: None,
            active_handle: None,
            is_moving_selection: false,
            move_drag_start: None,
            selection_start_rect: None,
            annotations: AnnotationState::default(),
            drawing_start: None,
            pen_points: Vec::new(),
            toast_message: None,
            pin_requested: None,
            is_finished: false,
        }
    }

    fn show_toast(&mut self, msg: &str) {
        self.toast_message = Some((msg.to_string(), std::time::Instant::now()));
    }

    fn normalized_selection(&self) -> Option<Rect> {
        self.selection.map(|r| {
            Rect::from_two_pos(
                Pos2::new(r.min.x.min(r.max.x), r.min.y.min(r.max.y)),
                Pos2::new(r.min.x.max(r.max.x), r.min.y.max(r.max.y)),
            )
        })
    }

    fn get_handle_rects(&self, rect: Rect) -> Vec<(HandlePosition, Rect)> {
        let size = 9.0;
        let c = rect.center();

        vec![
            (HandlePosition::TopLeft, Rect::from_center_size(rect.left_top(), Vec2::splat(size))),
            (HandlePosition::Top, Rect::from_center_size(Pos2::new(c.x, rect.top()), Vec2::splat(size))),
            (HandlePosition::TopRight, Rect::from_center_size(rect.right_top(), Vec2::splat(size))),
            (HandlePosition::Right, Rect::from_center_size(Pos2::new(rect.right(), c.y), Vec2::splat(size))),
            (HandlePosition::BottomRight, Rect::from_center_size(rect.right_bottom(), Vec2::splat(size))),
            (HandlePosition::Bottom, Rect::from_center_size(Pos2::new(c.x, rect.bottom()), Vec2::splat(size))),
            (HandlePosition::BottomLeft, Rect::from_center_size(rect.left_bottom(), Vec2::splat(size))),
            (HandlePosition::Left, Rect::from_center_size(Pos2::new(rect.left(), c.y), Vec2::splat(size))),
        ]
    }

    fn find_best_window_at(&self, mouse_pos: Pos2, scale_x: f32, scale_y: f32, screen_size: Vec2) -> Option<(Rect, String)> {
        let mut candidates: Vec<_> = self.captured.windows.iter().filter(|w| {
            let wx = w.x as f32 / scale_x;
            let wy = w.y as f32 / scale_y;
            let ww = w.width as f32 / scale_x;
            let wh = w.height as f32 / scale_y;
            let r = Rect::from_min_size(Pos2::new(wx, wy), Vec2::new(ww, wh));
            r.contains(mouse_pos)
        }).collect();

        if candidates.is_empty() {
            return None;
        }

        // Sort candidates:
        // 1. Prefer windows with real titles (main application windows)
        // 2. Prefer larger area (so we pick the full window, not a toolbar, tab, or sub-container)
        candidates.sort_by(|a, b| {
            let a_has_title = !a.title.is_empty();
            let b_has_title = !b.title.is_empty();
            if a_has_title != b_has_title {
                return b_has_title.cmp(&a_has_title);
            }
            let a_area = (a.width as u64) * (a.height as u64);
            let b_area = (b.width as u64) * (b.height as u64);
            b_area.cmp(&a_area)
        });

        let best = candidates.first()?;
        let wx = (best.x as f32 / scale_x).clamp(0.0, screen_size.x);
        let wy = (best.y as f32 / scale_y).clamp(0.0, screen_size.y);
        let ww = (best.width as f32 / scale_x).min(screen_size.x - wx);
        let wh = (best.height as f32 / scale_y).min(screen_size.y - wy);
        let rect = Rect::from_min_size(Pos2::new(wx, wy), Vec2::new(ww, wh));
        let title = if !best.title.is_empty() {
            best.title.clone()
        } else {
            best.app_name.clone()
        };
        Some((rect, title))
    }

    fn handle_cursor_icon(pos: HandlePosition) -> CursorIcon {
        match pos {
            HandlePosition::TopLeft | HandlePosition::BottomRight => CursorIcon::ResizeNwSe,
            HandlePosition::TopRight | HandlePosition::BottomLeft => CursorIcon::ResizeNeSw,
            HandlePosition::Top | HandlePosition::Bottom => CursorIcon::ResizeVertical,
            HandlePosition::Left | HandlePosition::Right => CursorIcon::ResizeHorizontal,
        }
    }
}

impl eframe::App for OverlayApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let total_rect = ui.max_rect();
        let screen_size = total_rect.size();
        if screen_size.x < 10.0 || screen_size.y < 10.0 {
            ctx.request_repaint();
            return;
        }

        // 1. Load screenshot texture once
        if self.texture.is_none() {
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [
                    self.captured.image.width() as usize,
                    self.captured.image.height() as usize,
                ],
                self.captured.image.as_raw(),
            );
            self.texture = Some(ctx.load_texture("fullscreen_shot", color_image, Default::default()));
        }

        let scale_x = self.captured.physical_width as f32 / screen_size.x;
        let scale_y = self.captured.physical_height as f32 / screen_size.y;

        let painter = ui.painter().clone();

        // 2. Draw background screenshot image 1:1
        if let Some(tex) = &self.texture {
            let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
            painter.image(tex.id(), total_rect, uv, Color32::WHITE);
        }

        // 3. Handle Keyboard Shortcuts & Mouse Cancel
        if ctx.input(|i| i.key_pressed(Key::Escape) || i.pointer.secondary_clicked()) {
            if self.selection.is_some() {
                self.selection = None;
                self.annotations.items.clear();
            } else {
                self.is_finished = true;
                return;
            }
        }

        // Quick Save (Cmd+Shift+S or Space)
        if ctx.input(|i| i.key_pressed(Key::Space)) && self.selection.is_some() {
            if let Some(sel) = self.normalized_selection() {
                let baked = bake_and_crop(&self.captured.image, sel, scale_x, scale_y, &self.annotations);
                let path = default_desktop_filename();
                if let Ok(()) = save_to_file(&baked, &path) {
                    self.show_toast(&format!("Masaüstüne kaydedildi: {}", path.file_name().unwrap_or_default().to_string_lossy()));
                }
            }
        }

        // Copy (Enter or Cmd+C)
        let copy_pressed = ctx.input(|i| i.key_pressed(Key::Enter) || (i.modifiers.command && i.key_pressed(Key::C)));
        if copy_pressed {
            if let Some(sel) = self.normalized_selection() {
                let baked = bake_and_crop(&self.captured.image, sel, scale_x, scale_y, &self.annotations);
                if let Ok(()) = copy_to_clipboard(&baked) {
                    self.show_toast("✓ Panoya Kopyalandı");
                    self.is_finished = true;
                    return;
                }
            }
        }

        // Undo (Cmd+Z)
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::Z)) {
            self.annotations.undo();
        }

        // Tool shortcuts
        if ctx.input(|i| i.key_pressed(Key::V)) { self.annotations.active_tool = Tool::Select; }
        if ctx.input(|i| i.key_pressed(Key::R)) { self.annotations.active_tool = Tool::Rectangle; }
        if ctx.input(|i| i.key_pressed(Key::O)) { self.annotations.active_tool = Tool::Ellipse; }
        if ctx.input(|i| i.key_pressed(Key::A)) { self.annotations.active_tool = Tool::Arrow; }
        if ctx.input(|i| i.key_pressed(Key::P)) { self.annotations.active_tool = Tool::Pen; }
        if ctx.input(|i| i.key_pressed(Key::N)) { self.annotations.active_tool = Tool::Step; }
        if ctx.input(|i| i.key_pressed(Key::T)) { self.annotations.active_tool = Tool::Text; }
        if ctx.input(|i| i.key_pressed(Key::B)) { self.annotations.active_tool = Tool::Mosaic; }
        if ctx.input(|i| i.key_pressed(Key::H)) { self.annotations.active_tool = Tool::Highlighter; }

        let mouse_pos = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO);
        let pointer_down = ctx.input(|i| i.pointer.primary_down());
        let pointer_clicked = ctx.input(|i| i.pointer.primary_clicked());
        let pointer_released = ctx.input(|i| i.pointer.primary_released());

        // 4. Dimmed Masking Outside Selection
        let norm_sel = self.normalized_selection();
        let mask_color = Color32::from_black_alpha(100);

        if let Some(sel) = norm_sel {
            // Top band
            painter.rect_filled(Rect::from_min_max(Pos2::new(total_rect.min.x, total_rect.min.y), Pos2::new(total_rect.max.x, sel.min.y)), 0.0, mask_color);
            // Bottom band
            painter.rect_filled(Rect::from_min_max(Pos2::new(total_rect.min.x, sel.max.y), Pos2::new(total_rect.max.x, total_rect.max.y)), 0.0, mask_color);
            // Left band
            painter.rect_filled(Rect::from_min_max(Pos2::new(total_rect.min.x, sel.min.y), Pos2::new(sel.min.x, sel.max.y)), 0.0, mask_color);
            // Right band
            painter.rect_filled(Rect::from_min_max(Pos2::new(sel.max.x, sel.min.y), Pos2::new(total_rect.max.x, sel.max.y)), 0.0, mask_color);

            // Selection Border (Crisp Apple Cyan/Blue)
            painter.rect_stroke(
                sel,
                0.0,
                Stroke::new(1.5, Color32::from_rgb(0, 122, 255)),
                egui::StrokeKind::Middle,
            );

            // 5. Draw Existing Annotations
            for item in &self.annotations.items {
                AnnotationState::paint_item(&painter, item);
            }

            // 6. Draw In-Progress Drawing
            if let Some(start) = self.drawing_start {
                match self.annotations.active_tool {
                    Tool::Rectangle => {
                        let cur_rect = Rect::from_two_pos(start, mouse_pos);
                        let item = AnnotationItem::Rectangle {
                            rect: cur_rect,
                            stroke_color: self.annotations.stroke_color,
                            stroke_width: self.annotations.stroke_width,
                            fill: self.annotations.fill_shape,
                        };
                        AnnotationState::paint_item(&painter, &item);
                    }
                    Tool::Ellipse => {
                        let cur_rect = Rect::from_two_pos(start, mouse_pos);
                        let item = AnnotationItem::Ellipse {
                            rect: cur_rect,
                            stroke_color: self.annotations.stroke_color,
                            stroke_width: self.annotations.stroke_width,
                            fill: self.annotations.fill_shape,
                        };
                        AnnotationState::paint_item(&painter, &item);
                    }
                    Tool::Arrow => {
                        let item = AnnotationItem::Arrow {
                            start,
                            end: mouse_pos,
                            color: self.annotations.stroke_color,
                            width: self.annotations.stroke_width,
                        };
                        AnnotationState::paint_item(&painter, &item);
                    }
                    Tool::Mosaic => {
                        let m_rect = Rect::from_two_pos(start, mouse_pos);
                        let blocks = AnnotationState::generate_mosaic_blocks(m_rect, &self.captured.image, scale_x, scale_y);
                        let item = AnnotationItem::Mosaic { rect: m_rect, blocks };
                        AnnotationState::paint_item(&painter, &item);
                    }
                    _ => {}
                }
            }

            if self.annotations.active_tool == Tool::Pen && !self.pen_points.is_empty() {
                let item = AnnotationItem::Pen {
                    points: self.pen_points.clone(),
                    color: self.annotations.stroke_color,
                    width: self.annotations.stroke_width,
                };
                AnnotationState::paint_item(&painter, &item);
            }

            if self.annotations.active_tool == Tool::Highlighter && !self.pen_points.is_empty() {
                let item = AnnotationItem::Highlighter {
                    points: self.pen_points.clone(),
                    color: self.annotations.stroke_color,
                    width: self.annotations.stroke_width,
                };
                AnnotationState::paint_item(&painter, &item);
            }

            // 7. Draw Resize Handles
            if self.annotations.active_tool == Tool::Select {
                for (_, h_rect) in self.get_handle_rects(sel) {
                    painter.rect_filled(h_rect, 2.0, Color32::WHITE);
                    painter.rect_stroke(
                        h_rect,
                        2.0,
                        Stroke::new(1.5, Color32::from_rgb(0, 122, 255)),
                        egui::StrokeKind::Middle,
                    );
                }
            }

            // 8. Dimension Pill Badge (e.g. 1280 x 720)
            let phys_w = (sel.width() * scale_x).round() as u32;
            let phys_h = (sel.height() * scale_y).round() as u32;
            let dim_text = format!("{} × {} px", phys_w, phys_h);

            let pill_pos = if sel.min.y > 28.0 {
                Pos2::new(sel.min.x, sel.min.y - 24.0)
            } else {
                Pos2::new(sel.min.x + 8.0, sel.min.y + 8.0)
            };
            let pill_w = (dim_text.len() as f32 * 7.5 + 16.0).max(95.0);
            let pill_rect = Rect::from_min_size(pill_pos, Vec2::new(pill_w, 22.0));
            painter.rect_filled(pill_rect, 4.0, Color32::from_black_alpha(190));
            painter.text(
                pill_rect.center(),
                Align2::CENTER_CENTER,
                &dim_text,
                FontId::new(11.0, FontFamily::Monospace),
                Color32::WHITE,
            );
        } else {
            // Full Dim Mask before any selection
            painter.rect_filled(total_rect, 0.0, Color32::from_black_alpha(60));

            // Smart Window Snapping
            if let Some((win_rect, title)) = self.find_best_window_at(mouse_pos, scale_x, scale_y, screen_size) {
                painter.rect_stroke(
                    win_rect,
                    8.0,
                    Stroke::new(2.5, Color32::from_rgb(0, 122, 255)),
                    egui::StrokeKind::Middle,
                );

                let badge_text = format!("Tıkla veya [Space]: {}", title);
                let badge_rect = Rect::from_center_size(
                    Pos2::new(win_rect.center().x, win_rect.min.y + 24.0),
                    Vec2::new((badge_text.len() as f32 * 7.5 + 24.0).min(400.0), 28.0),
                );
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(0, 122, 255));
                painter.text(
                    badge_rect.center(),
                    Align2::CENTER_CENTER,
                    &badge_text,
                    FontId::new(12.0, FontFamily::Proportional),
                    Color32::WHITE,
                );

                if ctx.input(|i| i.key_pressed(Key::Space)) {
                    self.selection = Some(win_rect);
                }
            }

            // Crosshair Guidelines across entire screen
            painter.line_segment(
                [Pos2::new(total_rect.min.x, mouse_pos.y), Pos2::new(total_rect.max.x, mouse_pos.y)],
                Stroke::new(0.5, Color32::from_white_alpha(70)),
            );
            painter.line_segment(
                [Pos2::new(mouse_pos.x, total_rect.min.y), Pos2::new(mouse_pos.x, total_rect.max.y)],
                Stroke::new(0.5, Color32::from_white_alpha(70)),
            );

            // Magnifier Loupe when not selecting
            if !self.is_dragging_new {
                if let Some(hex) = Magnifier::render(&painter, mouse_pos, &self.captured.image, scale_x, scale_y, screen_size) {
                    if ctx.input(|i| i.key_pressed(Key::C)) {
                        let mut clip = arboard::Clipboard::new().ok();
                        if let Some(c) = &mut clip {
                            let _ = c.set_text(&hex);
                            self.show_toast(&format!("{} Panoya Kopyalandı", hex));
                        }
                    }
                }
            }
        }

        // 9. Floating Toolbar (Rendered above or below selection)
        if let Some(sel) = norm_sel {
            let toolbar_width = 620.0;
            let toolbar_height = 44.0;

            let mut tb_x = sel.center().x - toolbar_width * 0.5;
            let mut tb_y = sel.max.y + 12.0;

            // Flip above if near bottom edge
            if tb_y + toolbar_height > screen_size.y - 10.0 {
                tb_y = sel.min.y - toolbar_height - 12.0;
            }
            tb_x = tb_x.clamp(10.0, screen_size.x - toolbar_width - 10.0);
            tb_y = tb_y.clamp(10.0, screen_size.y - toolbar_height - 10.0);

            let tb_rect = Rect::from_min_size(Pos2::new(tb_x, tb_y), Vec2::new(toolbar_width, toolbar_height));

            // Modern frosted-glass toolbar panel
            painter.rect_filled(tb_rect.translate(Vec2::new(0.0, 4.0)), 12.0, Color32::from_black_alpha(100));
            painter.rect_filled(tb_rect, 12.0, Color32::from_rgb(32, 32, 35));
            painter.rect_stroke(tb_rect, 12.0, Stroke::new(1.0, Color32::from_white_alpha(45)), egui::StrokeKind::Middle);

            // Render toolbar widgets using a sub-ui area
            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(tb_rect));
            child_ui.horizontal_centered(|ui| {
                ui.add_space(8.0);

                for (tool, tip) in [
                    (Tool::Select, "Seç / Taşı (V)"),
                    (Tool::Rectangle, "Dikdörtgen (R)"),
                    (Tool::Ellipse, "Daire (O)"),
                    (Tool::Arrow, "Ok (A)"),
                    (Tool::Pen, "Kalem (P)"),
                    (Tool::Text, "Metin (T)"),
                    (Tool::Step, "Numara (N)"),
                    (Tool::Mosaic, "Buzlama (B)"),
                    (Tool::Highlighter, "Vurgulayıcı (H)"),
                ] {
                    let is_active = self.annotations.active_tool == tool;
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(30.0, 26.0), egui::Sense::click());
                    let bg_color = if is_active {
                        Color32::from_rgb(0, 122, 255)
                    } else if resp.hovered() {
                        Color32::from_white_alpha(35)
                    } else {
                        Color32::TRANSPARENT
                    };
                    ui.painter().rect_filled(rect, 6.0, bg_color);
                    let icon_color = if is_active { Color32::WHITE } else { Color32::from_white_alpha(220) };
                    let c = rect.center();

                    match tool {
                        Tool::Select => {
                            let p1 = c + Vec2::new(-4.0, -5.0);
                            let p2 = c + Vec2::new(-4.0, 5.0);
                            let p3 = c + Vec2::new(-1.0, 2.0);
                            let p4 = c + Vec2::new(4.0, 5.0);
                            let p5 = c + Vec2::new(5.0, 3.0);
                            let p6 = c + Vec2::new(0.0, 0.0);
                            let p7 = c + Vec2::new(4.0, -1.0);
                            ui.painter().add(egui::Shape::convex_polygon(
                                vec![p1, p2, p3, p4, p5, p6, p7],
                                icon_color,
                                Stroke::NONE,
                            ));
                        }
                        Tool::Rectangle => {
                            let r = Rect::from_center_size(c, Vec2::new(14.0, 10.0));
                            ui.painter().rect_stroke(r, 2.0, Stroke::new(1.8, icon_color), egui::StrokeKind::Middle);
                        }
                        Tool::Ellipse => {
                            ui.painter().circle_stroke(c, 6.0, Stroke::new(1.8, icon_color));
                        }
                        Tool::Arrow => {
                            let start = c + Vec2::new(-5.0, 4.0);
                            let end = c + Vec2::new(5.0, -4.0);
                            ui.painter().line_segment([start, end], Stroke::new(2.0, icon_color));
                            ui.painter().line_segment([end, end + Vec2::new(-5.0, 0.0)], Stroke::new(2.0, icon_color));
                            ui.painter().line_segment([end, end + Vec2::new(0.0, 5.0)], Stroke::new(2.0, icon_color));
                        }
                        Tool::Pen => {
                            let start = c + Vec2::new(-5.0, 5.0);
                            let end = c + Vec2::new(5.0, -5.0);
                            ui.painter().line_segment([start, end], Stroke::new(2.2, icon_color));
                            ui.painter().circle_filled(start, 1.3, icon_color);
                        }
                        Tool::Text => {
                            ui.painter().text(
                                c,
                                Align2::CENTER_CENTER,
                                "T",
                                FontId::new(14.0, FontFamily::Proportional),
                                icon_color,
                            );
                        }
                        Tool::Step => {
                            ui.painter().circle_filled(c, 6.5, icon_color);
                            let num_color = if is_active { Color32::from_rgb(0, 122, 255) } else { Color32::from_rgb(32, 32, 35) };
                            ui.painter().text(c, Align2::CENTER_CENTER, "1", FontId::new(10.0, FontFamily::Proportional), num_color);
                        }
                        Tool::Mosaic => {
                            let r = Rect::from_center_size(c, Vec2::new(12.0, 12.0));
                            ui.painter().rect_stroke(r, 1.0, Stroke::new(1.0, icon_color), egui::StrokeKind::Middle);
                            let h = r.width() / 2.0;
                            ui.painter().rect_filled(Rect::from_min_size(r.min, Vec2::splat(h)), 0.0, icon_color);
                            ui.painter().rect_filled(Rect::from_min_size(c, Vec2::splat(h)), 0.0, icon_color);
                        }
                        Tool::Highlighter => {
                            let r = Rect::from_center_size(c, Vec2::new(14.0, 6.0));
                            ui.painter().rect_filled(r, 1.5, icon_color.gamma_multiply(0.65));
                            ui.painter().rect_stroke(r, 1.5, Stroke::new(1.0, icon_color), egui::StrokeKind::Middle);
                        }
                    }

                    if resp.on_hover_text(tip).clicked() {
                        self.annotations.active_tool = tool;
                    }
                }

                ui.separator();

                // Color choices
                let colors = [
                    Color32::from_rgb(255, 59, 48),  // Red
                    Color32::from_rgb(0, 122, 255),  // Blue
                    Color32::from_rgb(52, 199, 89),  // Green
                    Color32::from_rgb(255, 204, 0),  // Yellow
                    Color32::from_rgb(175, 82, 222), // Purple
                    Color32::WHITE,
                ];

                for color in colors {
                    let is_sel = self.annotations.stroke_color == color;
                    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::click());
                    ui.painter().circle_filled(rect.center(), 7.0, color);
                    if is_sel {
                        ui.painter().circle_stroke(rect.center(), 9.0, Stroke::new(2.0, Color32::WHITE));
                    }
                    if resp.clicked() {
                        self.annotations.stroke_color = color;
                    }
                }

                ui.separator();

                // Undo Button (↩)
                let (u_rect, u_resp) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), egui::Sense::click());
                if u_resp.hovered() {
                    ui.painter().rect_filled(u_rect, 6.0, Color32::from_white_alpha(35));
                }
                let uc = u_rect.center();
                let u_col = if u_resp.hovered() { Color32::WHITE } else { Color32::from_white_alpha(200) };
                ui.painter().line_segment([uc + Vec2::new(4.0, 4.0), uc + Vec2::new(4.0, -2.0)], Stroke::new(1.8, u_col));
                ui.painter().line_segment([uc + Vec2::new(4.0, -2.0), uc + Vec2::new(-4.0, -2.0)], Stroke::new(1.8, u_col));
                ui.painter().line_segment([uc + Vec2::new(-4.0, -2.0), uc + Vec2::new(-1.0, -5.0)], Stroke::new(1.8, u_col));
                ui.painter().line_segment([uc + Vec2::new(-4.0, -2.0), uc + Vec2::new(-1.0, 1.0)], Stroke::new(1.8, u_col));
                if u_resp.on_hover_text("Geri Al (Cmd+Z)").clicked() {
                    self.annotations.undo();
                }

                // Clear Button (🗑)
                let (d_rect, d_resp) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), egui::Sense::click());
                if d_resp.hovered() {
                    ui.painter().rect_filled(d_rect, 6.0, Color32::from_white_alpha(35));
                }
                let dc = d_rect.center();
                let d_col = if d_resp.hovered() { Color32::WHITE } else { Color32::from_white_alpha(200) };
                let bin_rect = Rect::from_center_size(dc + Vec2::new(0.0, 2.0), Vec2::new(10.0, 10.0));
                ui.painter().rect_stroke(bin_rect, 1.5, Stroke::new(1.5, d_col), egui::StrokeKind::Middle);
                ui.painter().line_segment([dc + Vec2::new(-6.0, -3.5), dc + Vec2::new(6.0, -3.5)], Stroke::new(1.5, d_col));
                if d_resp.on_hover_text("Çizimleri Temizle").clicked() {
                    self.annotations.clear();
                }

                ui.separator();

                // Pin Button (📌)
                let pin_btn = egui::Button::new(egui::RichText::new("📌 Sabitle").size(12.0).color(Color32::WHITE))
                    .fill(Color32::from_rgb(88, 86, 214))
                    .corner_radius(6.0);
                if ui.add(pin_btn).on_hover_text("Ekrana Sabitle (Yüzen Pencere)").clicked() {
                    let baked = bake_and_crop(&self.captured.image, sel, scale_x, scale_y, &self.annotations);
                    self.pin_requested = Some(baked);
                    self.is_finished = true;
                }

                // Save As Button (💾)
                let save_btn = egui::Button::new(egui::RichText::new("💾 Kaydet").size(12.0).color(Color32::WHITE))
                    .fill(Color32::from_rgb(50, 50, 54))
                    .corner_radius(6.0);
                if ui.add(save_btn).on_hover_text("Dosya Olarak Kaydet (Cmd+S)").clicked() {
                    let baked = bake_and_crop(&self.captured.image, sel, scale_x, scale_y, &self.annotations);
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("Ekran Resmi.png")
                        .add_filter("PNG Image", &["png"])
                        .add_filter("JPEG Image", &["jpg", "jpeg"])
                        .save_file()
                    {
                        if let Ok(()) = save_to_file(&baked, &path) {
                            self.show_toast("Başarıyla kaydedildi");
                        }
                    }
                }

                // Copy Button (📋)
                let copy_btn = egui::Button::new(egui::RichText::new("📋 Kopyala").size(12.5).strong().color(Color32::WHITE))
                    .fill(Color32::from_rgb(52, 199, 89))
                    .corner_radius(6.0);
                if ui.add(copy_btn).on_hover_text("Panoya Kopyala ve Çık (Enter)").clicked() {
                    let baked = bake_and_crop(&self.captured.image, sel, scale_x, scale_y, &self.annotations);
                    let _ = copy_to_clipboard(&baked);
                    self.is_finished = true;
                }

                // Cancel Button (✕)
                let (x_rect, x_resp) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), egui::Sense::click());
                if x_resp.hovered() {
                    ui.painter().rect_filled(x_rect, 6.0, Color32::from_white_alpha(35));
                }
                let xc = x_rect.center();
                let x_col = if x_resp.hovered() { Color32::WHITE } else { Color32::from_white_alpha(200) };
                ui.painter().line_segment([xc + Vec2::new(-4.5, -4.5), xc + Vec2::new(4.5, 4.5)], Stroke::new(2.0, x_col));
                ui.painter().line_segment([xc + Vec2::new(4.5, -4.5), xc + Vec2::new(-4.5, 4.5)], Stroke::new(2.0, x_col));
                if x_resp.on_hover_text("İptal (Esc)").clicked() {
                    self.is_finished = true;
                }
            });
        }

        // Dynamic Cursor feedback
        if let Some(sel) = norm_sel {
            if self.annotations.active_tool == Tool::Select {
                let mut handle_hovered = false;
                for (pos, h_rect) in self.get_handle_rects(sel) {
                    if h_rect.contains(mouse_pos) {
                        ctx.set_cursor_icon(Self::handle_cursor_icon(pos));
                        handle_hovered = true;
                        break;
                    }
                }
                if !handle_hovered {
                    if self.is_moving_selection {
                        ctx.set_cursor_icon(CursorIcon::Grabbing);
                    } else if sel.contains(mouse_pos) {
                        ctx.set_cursor_icon(CursorIcon::Grab);
                    } else {
                        ctx.set_cursor_icon(CursorIcon::Default);
                    }
                }
            } else {
                ctx.set_cursor_icon(CursorIcon::Crosshair);
            }
        } else {
            ctx.set_cursor_icon(CursorIcon::Crosshair);
        }

        // 10. Mouse Interaction Logic (Selecting, Moving, Resizing, Drawing)
        if pointer_down {
            if self.is_dragging_new {
                // Actively dragging out a new selection!
                if let Some(start) = self.drag_start {
                    self.selection = Some(Rect::from_two_pos(start, mouse_pos));
                }
            } else if self.selection.is_none() {
                // Initial selection drag start
                self.is_dragging_new = true;
                self.drag_start = Some(mouse_pos);
                self.selection = Some(Rect::from_two_pos(mouse_pos, mouse_pos));
            } else if let Some(sel) = norm_sel {
                // Check if clicking resize handles or inside selection
                if self.annotations.active_tool == Tool::Select {
                    if self.active_handle.is_none() && !self.is_moving_selection {
                        for (pos, h_rect) in self.get_handle_rects(sel) {
                            if h_rect.contains(mouse_pos) {
                                self.active_handle = Some(pos);
                                break;
                            }
                        }
                        if self.active_handle.is_none() && sel.contains(mouse_pos) && self.move_drag_start.is_none() {
                            self.is_moving_selection = true;
                            self.move_drag_start = Some(mouse_pos);
                            self.selection_start_rect = Some(sel);
                        } else if self.active_handle.is_none() && !sel.contains(mouse_pos) {
                            // Clicked outside existing selection: start dragging a brand new selection!
                            self.is_dragging_new = true;
                            self.drag_start = Some(mouse_pos);
                            self.selection = Some(Rect::from_two_pos(mouse_pos, mouse_pos));
                            self.annotations.items.clear();
                        }
                    }

                    // Apply handle resizing
                    if let Some(handle) = self.active_handle {
                        let mut new_sel = sel;
                        match handle {
                            HandlePosition::TopLeft => { new_sel.min = mouse_pos; }
                            HandlePosition::Top => { new_sel.min.y = mouse_pos.y; }
                            HandlePosition::TopRight => { new_sel.max.x = mouse_pos.x; new_sel.min.y = mouse_pos.y; }
                            HandlePosition::Right => { new_sel.max.x = mouse_pos.x; }
                            HandlePosition::BottomRight => { new_sel.max = mouse_pos; }
                            HandlePosition::Bottom => { new_sel.max.y = mouse_pos.y; }
                            HandlePosition::BottomLeft => { new_sel.min.x = mouse_pos.x; new_sel.max.y = mouse_pos.y; }
                            HandlePosition::Left => { new_sel.min.x = mouse_pos.x; }
                        }
                        self.selection = Some(new_sel);
                    } else if self.is_moving_selection {
                        if let (Some(start_m), Some(orig_sel)) = (self.move_drag_start, self.selection_start_rect) {
                            let delta = mouse_pos - start_m;
                            self.selection = Some(orig_sel.translate(delta));
                        }
                    }
                } else {
                    // Active annotation tool drawing
                    if self.drawing_start.is_none() {
                        self.drawing_start = Some(mouse_pos);
                        if self.annotations.active_tool == Tool::Pen || self.annotations.active_tool == Tool::Highlighter {
                            self.pen_points = vec![mouse_pos];
                        }
                    } else if self.annotations.active_tool == Tool::Pen || self.annotations.active_tool == Tool::Highlighter {
                        self.pen_points.push(mouse_pos);
                    }
                }
            }
        }

        // Pointer release: finalize actions
        if pointer_released {
            if self.is_dragging_new {
                if let Some(sel) = self.normalized_selection() {
                    // Check if it was just a tiny click (< 6px wide and tall)
                    if sel.width() < 6.0 && sel.height() < 6.0 {
                        // Check if a window was clicked
                        if let Some((win_rect, _)) = self.find_best_window_at(mouse_pos, scale_x, scale_y, screen_size) {
                            self.selection = Some(win_rect);
                        } else {
                            // Clicking on empty area clears selection
                            self.selection = None;
                        }
                    }
                }
            }
            self.is_dragging_new = false;
            self.drag_start = None;
            self.active_handle = None;
            self.is_moving_selection = false;
            self.move_drag_start = None;
            self.selection_start_rect = None;

            if let Some(start) = self.drawing_start {
                match self.annotations.active_tool {
                    Tool::Rectangle => {
                        let r = Rect::from_two_pos(start, mouse_pos);
                        if r.width() > 3.0 && r.height() > 3.0 {
                            self.annotations.items.push(AnnotationItem::Rectangle {
                                rect: r,
                                stroke_color: self.annotations.stroke_color,
                                stroke_width: self.annotations.stroke_width,
                                fill: self.annotations.fill_shape,
                            });
                        }
                    }
                    Tool::Ellipse => {
                        let r = Rect::from_two_pos(start, mouse_pos);
                        if r.width() > 3.0 && r.height() > 3.0 {
                            self.annotations.items.push(AnnotationItem::Ellipse {
                                rect: r,
                                stroke_color: self.annotations.stroke_color,
                                stroke_width: self.annotations.stroke_width,
                                fill: self.annotations.fill_shape,
                            });
                        }
                    }
                    Tool::Arrow => {
                        if (mouse_pos - start).length() > 5.0 {
                            self.annotations.items.push(AnnotationItem::Arrow {
                                start,
                                end: mouse_pos,
                                color: self.annotations.stroke_color,
                                width: self.annotations.stroke_width,
                            });
                        }
                    }
                    Tool::Pen => {
                        if self.pen_points.len() >= 2 {
                            self.annotations.items.push(AnnotationItem::Pen {
                                points: self.pen_points.clone(),
                                color: self.annotations.stroke_color,
                                width: self.annotations.stroke_width,
                            });
                        }
                        self.pen_points.clear();
                    }
                    Tool::Highlighter => {
                        if self.pen_points.len() >= 2 {
                            self.annotations.items.push(AnnotationItem::Highlighter {
                                points: self.pen_points.clone(),
                                color: self.annotations.stroke_color,
                                width: self.annotations.stroke_width,
                            });
                        }
                        self.pen_points.clear();
                    }
                    Tool::Mosaic => {
                        let r = Rect::from_two_pos(start, mouse_pos);
                        if r.width() > 5.0 && r.height() > 5.0 {
                            let blocks = AnnotationState::generate_mosaic_blocks(r, &self.captured.image, scale_x, scale_y);
                            self.annotations.items.push(AnnotationItem::Mosaic { rect: r, blocks });
                        }
                    }
                    _ => {}
                }
                self.drawing_start = None;
            }
        }

        // Click on Step Tool
        if pointer_clicked && self.annotations.active_tool == Tool::Step {
            if let Some(sel) = norm_sel {
                if sel.contains(mouse_pos) {
                    self.annotations.add_step(mouse_pos);
                }
            }
        }

        // Toast feedback banner
        if let Some((msg, time)) = &self.toast_message {
            if time.elapsed().as_secs_f32() < 2.0 {
                let toast_rect = Rect::from_center_size(
                    Pos2::new(screen_size.x * 0.5, 60.0),
                    Vec2::new(260.0, 38.0),
                );
                painter.rect_filled(toast_rect, 8.0, Color32::from_black_alpha(220));
                painter.rect_stroke(toast_rect, 8.0, Stroke::new(1.0, Color32::from_white_alpha(50)), egui::StrokeKind::Middle);
                painter.text(
                    toast_rect.center(),
                    Align2::CENTER_CENTER,
                    msg,
                    FontId::new(13.0, FontFamily::Proportional),
                    Color32::WHITE,
                );
                ctx.request_repaint();
            } else {
                self.toast_message = None;
            }
        }
    }
}
