use eframe::egui::{
    Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Stroke, Vec2,
};
use image::RgbaImage;

pub struct Magnifier;

impl Magnifier {
    pub fn render(
        painter: &Painter,
        cursor_pos: Pos2,
        bg_image: &RgbaImage,
        scale_x: f32,
        scale_y: f32,
        screen_size: Vec2,
    ) -> Option<String> {
        let img_w = bg_image.width() as i32;
        let img_h = bg_image.height() as i32;

        let center_px = (cursor_pos.x * scale_x) as i32;
        let center_py = (cursor_pos.y * scale_y) as i32;

        if center_px < 0 || center_px >= img_w || center_py < 0 || center_py >= img_h {
            return None;
        }

        let current_rgba = bg_image.get_pixel(center_px as u32, center_py as u32);
        let hex_code = format!(
            "#{:02X}{:02X}{:02X}",
            current_rgba[0], current_rgba[1], current_rgba[2]
        );
        let rgb_text = format!(
            "{}, {}, {}",
            current_rgba[0], current_rgba[1], current_rgba[2]
        );
        let pos_text = format!("X: {}  Y: {}", center_px, center_py);

        // Grid config
        let grid_size: i32 = 11; // 11x11 pixel window
        let half_grid = grid_size / 2;
        let cell_size: f32 = 9.0;
        let grid_view_size = (grid_size as f32) * cell_size;

        // Container size
        let card_w = grid_view_size + 24.0;
        let card_h = grid_view_size + 70.0;

        // Place card offset from cursor, flipping if hitting screen bounds
        let mut card_x = cursor_pos.x + 24.0;
        let mut card_y = cursor_pos.y + 24.0;

        if card_x + card_w > screen_size.x {
            card_x = cursor_pos.x - card_w - 24.0;
        }
        if card_y + card_h > screen_size.y {
            card_y = cursor_pos.y - card_h - 24.0;
        }

        let card_rect = Rect::from_min_size(Pos2::new(card_x, card_y), Vec2::new(card_w, card_h));

        // Card shadow & background
        painter.rect_filled(
            card_rect.translate(Vec2::new(0.0, 4.0)),
            10.0,
            Color32::from_black_alpha(120),
        );
        painter.rect_filled(card_rect, 10.0, Color32::from_rgb(28, 28, 30));
        painter.rect_stroke(
            card_rect,
            10.0,
            Stroke::new(1.0, Color32::from_white_alpha(40)),
            egui::StrokeKind::Middle,
        );

        // Render pixel grid
        let grid_start = Pos2::new(card_x + 12.0, card_y + 12.0);

        for dy in -half_grid..=half_grid {
            for dx in -half_grid..=half_grid {
                let sx = (center_px + dx).clamp(0, img_w - 1) as u32;
                let sy = (center_py + dy).clamp(0, img_h - 1) as u32;
                let rgba = bg_image.get_pixel(sx, sy);
                let color = Color32::from_rgb(rgba[0], rgba[1], rgba[2]);

                let cell_rect = Rect::from_min_size(
                    Pos2::new(
                        grid_start.x + ((dx + half_grid) as f32) * cell_size,
                        grid_start.y + ((dy + half_grid) as f32) * cell_size,
                    ),
                    Vec2::splat(cell_size),
                );

                painter.rect_filled(cell_rect, 0.0, color);
                painter.rect_stroke(
                    cell_rect,
                    0.0,
                    Stroke::new(0.5, Color32::from_white_alpha(30)),
                    egui::StrokeKind::Middle,
                );
            }
        }

        // Center crosshair cell highlight
        let center_cell_rect = Rect::from_min_size(
            Pos2::new(
                grid_start.x + (half_grid as f32) * cell_size,
                grid_start.y + (half_grid as f32) * cell_size,
            ),
            Vec2::splat(cell_size),
        );
        painter.rect_stroke(
            center_cell_rect.expand(1.0),
            1.0,
            Stroke::new(2.0, Color32::from_rgb(255, 59, 48)),
            egui::StrokeKind::Middle,
        );

        // Information text section
        let text_y = grid_start.y + grid_view_size + 8.0;

        // Color sample pill + HEX text
        let pill_rect = Rect::from_min_size(
            Pos2::new(grid_start.x, text_y + 2.0),
            Vec2::new(14.0, 14.0),
        );
        painter.rect_filled(
            pill_rect,
            4.0,
            Color32::from_rgb(current_rgba[0], current_rgba[1], current_rgba[2]),
        );
        painter.rect_stroke(
            pill_rect,
            4.0,
            Stroke::new(1.0, Color32::from_white_alpha(100)),
            egui::StrokeKind::Middle,
        );

        painter.text(
            Pos2::new(grid_start.x + 20.0, text_y),
            Align2::LEFT_TOP,
            &hex_code,
            FontId::new(12.0, FontFamily::Monospace),
            Color32::WHITE,
        );

        // Coordinates & RGB
        painter.text(
            Pos2::new(grid_start.x, text_y + 20.0),
            Align2::LEFT_TOP,
            &pos_text,
            FontId::new(10.5, FontFamily::Proportional),
            Color32::from_white_alpha(180),
        );

        painter.text(
            Pos2::new(grid_start.x, text_y + 34.0),
            Align2::LEFT_TOP,
            &format!("RGB: {}", rgb_text),
            FontId::new(10.0, FontFamily::Monospace),
            Color32::from_white_alpha(140),
        );

        Some(hex_code)
    }
}
