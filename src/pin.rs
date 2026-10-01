use eframe::egui::{self, Color32, Pos2, Rect, Stroke, TextureHandle, Vec2};
use image::RgbaImage;

pub struct PinApp {
    image: RgbaImage,
    texture: Option<TextureHandle>,
    zoom: f32,
    copied_notification: Option<std::time::Instant>,
    pub is_closed: bool,
}

impl PinApp {
    pub fn new(image: RgbaImage) -> Self {
        Self {
            image,
            texture: None,
            zoom: 1.0,
            copied_notification: None,
            is_closed: false,
        }
    }
}

impl eframe::App for PinApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Load texture once
        if self.texture.is_none() {
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [self.image.width() as usize, self.image.height() as usize],
                self.image.as_raw(),
            );
            self.texture = Some(ctx.load_texture("pinned_image", color_image, Default::default()));
        }

        // Handle Escape to close
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.is_closed = true;
            return;
        }

        // Handle Cmd+C to copy
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C)) {
            let _ = crate::export::copy_to_clipboard(&self.image);
            self.copied_notification = Some(std::time::Instant::now());
        }

        // Zoom with mouse wheel
        let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.0 {
            self.zoom = (self.zoom + scroll * 0.002).clamp(0.2, 5.0);
        }

        let panel_rect = ui.available_rect_before_wrap();
        let painter = ui.painter().clone();

        // Render pinned image
        if let Some(tex) = &self.texture {
            let orig_size = Vec2::new(self.image.width() as f32, self.image.height() as f32);
            let display_size = orig_size * self.zoom;
            let center = panel_rect.center();
            let draw_rect = Rect::from_center_size(center, display_size);

            // Draw subtle drop shadow
            painter.rect_filled(
                draw_rect.translate(Vec2::new(0.0, 4.0)),
                8.0,
                Color32::from_black_alpha(80),
            );

            // Draw texture
            let uv = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
            painter.image(tex.id(), draw_rect, uv, Color32::WHITE);

            // Thin border
            painter.rect_stroke(
                draw_rect,
                0.0,
                Stroke::new(1.0, Color32::from_white_alpha(50)),
                egui::StrokeKind::Middle,
            );

            // Double click to close
            let response = ui.allocate_rect(draw_rect, egui::Sense::click_and_drag());
            if response.double_clicked() {
                self.is_closed = true;
            }
            if response.dragged() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
        }

        // Notification overlay
        if let Some(t) = self.copied_notification {
            if t.elapsed().as_secs_f32() < 1.5 {
                let badge_rect = Rect::from_center_size(
                    panel_rect.center(),
                    Vec2::new(180.0, 36.0),
                );
                painter.rect_filled(badge_rect, 8.0, Color32::from_black_alpha(200));
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✓ Panoya Kopyalandı",
                    egui::FontId::new(14.0, egui::FontFamily::Proportional),
                    Color32::WHITE,
                );
                ctx.request_repaint();
            } else {
                self.copied_notification = None;
            }
        }
    }
}
