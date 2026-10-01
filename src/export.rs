use crate::annotation::{AnnotationItem, AnnotationState};
use eframe::egui::{Color32, Pos2, Rect, Vec2};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{
    draw_antialiased_line_segment_mut, draw_filled_circle_mut, draw_polygon_mut,
    draw_filled_rect_mut, draw_hollow_circle_mut, draw_hollow_rect_mut,
};
use imageproc::point::Point;
use imageproc::rect::Rect as ImgRect;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

fn to_rgba(c: Color32) -> Rgba<u8> {
    Rgba([c.r(), c.g(), c.b(), c.a()])
}

pub fn bake_and_crop(
    src_image: &RgbaImage,
    crop_rect_pt: Rect,
    scale_x: f32,
    scale_y: f32,
    annotations: &AnnotationState,
) -> RgbaImage {
    let img_w = src_image.width() as i32;
    let img_h = src_image.height() as i32;

    let x0 = ((crop_rect_pt.min.x * scale_x).round() as i32).clamp(0, img_w);
    let y0 = ((crop_rect_pt.min.y * scale_y).round() as i32).clamp(0, img_h);
    let x1 = ((crop_rect_pt.max.x * scale_x).round() as i32).clamp(0, img_w);
    let y1 = ((crop_rect_pt.max.y * scale_y).round() as i32).clamp(0, img_h);

    let crop_x = x0.min(x1) as u32;
    let crop_y = y0.min(y1) as u32;
    let crop_w = (x1 - x0).abs().max(1) as u32;
    let crop_h = (y1 - y0).abs().max(1) as u32;

    // Crop base image
    let mut cropped = image::imageops::crop_imm(src_image, crop_x, crop_y, crop_w, crop_h).to_image();

    // Coordinate conversion from logical points to cropped image pixels
    let pt_to_crop = |pt: Pos2| -> (i32, i32) {
        let px = (pt.x * scale_x).round() as i32 - crop_x as i32;
        let py = (pt.y * scale_y).round() as i32 - crop_y as i32;
        (px, py)
    };

    // Bake annotations
    for item in &annotations.items {
        match item {
            AnnotationItem::Mosaic { blocks, .. } => {
                for (b_rect, color) in blocks {
                    let (bx0, by0) = pt_to_crop(b_rect.min);
                    let (bx1, by1) = pt_to_crop(b_rect.max);
                    let rx = bx0.min(bx1);
                    let ry = by0.min(by1);
                    let rw = (bx1 - bx0).abs() as u32;
                    let rh = (by1 - by0).abs() as u32;

                    if rw > 0 && rh > 0 {
                        let rect_area = ImgRect::at(rx, ry).of_size(rw, rh);
                        draw_filled_rect_mut(&mut cropped, rect_area, to_rgba(*color));
                    }
                }
            }
            AnnotationItem::Rectangle {
                rect,
                stroke_color,
                stroke_width,
                fill,
            } => {
                let (rx0, ry0) = pt_to_crop(rect.min);
                let (rx1, ry1) = pt_to_crop(rect.max);
                let x = rx0.min(rx1);
                let y = ry0.min(ry1);
                let w = (rx1 - rx0).abs() as u32;
                let h = (ry1 - ry0).abs() as u32;

                if w > 0 && h > 0 {
                    let rect_area = ImgRect::at(x, y).of_size(w, h);
                    if *fill {
                        let fill_c = stroke_color.gamma_multiply(0.25);
                        draw_filled_rect_mut(&mut cropped, rect_area, to_rgba(fill_c));
                    }
                    let stroke_px = (*stroke_width * scale_x).round().max(1.0) as i32;
                    for s in 0..stroke_px {
                        let r = ImgRect::at(x + s, y + s).of_size(
                            w.saturating_sub((s * 2) as u32).max(1),
                            h.saturating_sub((s * 2) as u32).max(1),
                        );
                        draw_hollow_rect_mut(&mut cropped, r, to_rgba(*stroke_color));
                    }
                }
            }
            AnnotationItem::Ellipse {
                rect,
                stroke_color,
                stroke_width,
                fill,
            } => {
                let (rx0, ry0) = pt_to_crop(rect.min);
                let (rx1, ry1) = pt_to_crop(rect.max);
                let cx = (rx0 + rx1) / 2;
                let cy = (ry0 + ry1) / 2;
                let radius = ((rx1 - rx0).abs().min((ry1 - ry0).abs()) / 2).max(1);

                if *fill {
                    let fill_c = stroke_color.gamma_multiply(0.25);
                    draw_filled_circle_mut(&mut cropped, (cx, cy), radius, to_rgba(fill_c));
                }
                let stroke_px = (*stroke_width * scale_x).round().max(1.0) as i32;
                for s in 0..stroke_px {
                    let r = (radius - s).max(1);
                    draw_hollow_circle_mut(&mut cropped, (cx, cy), r, to_rgba(*stroke_color));
                }
            }
            AnnotationItem::Arrow {
                start,
                end,
                color,
                width,
            } => {
                let (sx, sy) = pt_to_crop(*start);
                let (ex, ey) = pt_to_crop(*end);

                let start_pt = Pos2::new(sx as f32, sy as f32);
                let end_pt = Pos2::new(ex as f32, ey as f32);

                let vec = end_pt - start_pt;
                let len = vec.length();
                if len < 3.0 {
                    continue;
                }
                let dir = vec / len;
                let normal = Vec2::new(-dir.y, dir.x);

                let stroke_px = (*width * scale_x).max(2.0);
                let head_len = (stroke_px * 4.5).clamp(20.0, 50.0).min(len * 0.7);
                let head_width = head_len * 0.6;

                let line_end = end_pt - dir * (head_len * 0.5);

                // Draw thick line by drawing multiple parallel antialiased lines
                let line_steps = stroke_px.round() as i32;
                let half_steps = line_steps / 2;
                for step in -half_steps..=half_steps {
                    let offset = normal * (step as f32 * 0.8);
                    let p1 = start_pt + offset;
                    let p2 = line_end + offset;
                    draw_antialiased_line_segment_mut(
                        &mut cropped,
                        (p1.x as i32, p1.y as i32),
                        (p2.x as i32, p2.y as i32),
                        to_rgba(*color),
                        imageproc::pixelops::interpolate,
                    );
                }

                // Arrow head polygon
                let tip = end_pt;
                let left = end_pt - dir * head_len + normal * head_width;
                let right = end_pt - dir * head_len - normal * head_width;

                let poly = [
                    Point::new(tip.x as i32, tip.y as i32),
                    Point::new(left.x as i32, left.y as i32),
                    Point::new(right.x as i32, right.y as i32),
                ];
                draw_polygon_mut(&mut cropped, &poly, to_rgba(*color));
            }
            AnnotationItem::Pen {
                points,
                color,
                width,
            } => {
                let stroke_px = (*width * scale_x).max(1.0);
                let steps = stroke_px.round() as i32;
                let half = steps / 2;

                for window in points.windows(2) {
                    let (x1, y1) = pt_to_crop(window[0]);
                    let (x2, y2) = pt_to_crop(window[1]);

                    for dx in -half..=half {
                        for dy in -half..=half {
                            draw_antialiased_line_segment_mut(
                                &mut cropped,
                                (x1 + dx, y1 + dy),
                                (x2 + dx, y2 + dy),
                                to_rgba(*color),
                                imageproc::pixelops::interpolate,
                            );
                        }
                    }
                }
            }
            AnnotationItem::Highlighter {
                points,
                color,
                width,
            } => {
                let high_c = color.gamma_multiply(0.4);
                let stroke_px = (*width * 3.5 * scale_x).max(4.0);
                let half = (stroke_px / 2.0).round() as i32;

                for window in points.windows(2) {
                    let (x1, y1) = pt_to_crop(window[0]);
                    let (x2, y2) = pt_to_crop(window[1]);

                    for step in -half..=half {
                        draw_antialiased_line_segment_mut(
                            &mut cropped,
                            (x1, y1 + step),
                            (x2, y2 + step),
                            to_rgba(high_c),
                            imageproc::pixelops::interpolate,
                        );
                    }
                }
            }
            AnnotationItem::StepBadge {
                center,
                number,
                bg_color,
                text_color: _,
                radius,
            } => {
                let (cx, cy) = pt_to_crop(*center);
                let r_px = (*radius * scale_x).round() as i32;

                // Outer border (white ring)
                draw_filled_circle_mut(&mut cropped, (cx, cy), r_px + 3, Rgba([255, 255, 255, 255]));
                // Circle body
                draw_filled_circle_mut(&mut cropped, (cx, cy), r_px, to_rgba(*bg_color));

                // Simple high-contrast number indicator: draw dot / cross pattern or digit bars
                let digit_color = Rgba([255, 255, 255, 255]);
                let num = *number;
                if num <= 9 {
                    // Draw a mini clean center marker or badge indicator
                    draw_filled_circle_mut(&mut cropped, (cx, cy), (r_px / 3).max(2), digit_color);
                }
            }
            AnnotationItem::Text { .. } => {
                // Text rendering can also be composited
            }
        }
    }

    cropped
}

pub fn copy_to_clipboard(image: &RgbaImage) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("Clipboard error: {e}"))?;
    let img_data = arboard::ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: Cow::Borrowed(image.as_raw()),
    };
    clipboard
        .set_image(img_data)
        .map_err(|e| format!("Failed to set clipboard image: {e}"))?;
    Ok(())
}

pub fn save_to_file(image: &RgbaImage, path: &Path) -> Result<(), String> {
    image
        .save(path)
        .map_err(|e| format!("Failed to save image to {}: {e}", path.display()))
}

pub fn default_desktop_filename() -> PathBuf {
    let desktop = dirs::desktop_dir().unwrap_or_else(|| PathBuf::from("."));
    let now = chrono::Local::now();
    let filename = format!("Ekran Resmi {}.png", now.format("%Y-%m-%d %H.%M.%S"));
    desktop.join(filename)
}
