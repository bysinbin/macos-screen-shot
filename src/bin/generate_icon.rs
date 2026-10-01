use image::{Rgba, RgbaImage};

fn main() {
    let size: u32 = 1024;
    let mut img = RgbaImage::new(size, size);

    let center = size as f32 * 0.5;
    let radius = size as f32 * 0.44; // rounded squircle radius

    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32 - center).abs();
            let dy = (y as f32 - center).abs();

            // Superellipse / Squircle distance: (|x|^4 + |y|^4)^(1/4)
            let squircle_dist = (dx.powi(4) + dy.powi(4)).powf(0.25);

            if squircle_dist <= radius {
                // Smooth anti-aliased edge
                let edge_alpha = ((radius - squircle_dist) / 2.0).clamp(0.0, 1.0);

                // Background gradient: Dark Indigo to Vibrant Violet / Cyan
                let norm_y = y as f32 / size as f32;
                let norm_x = x as f32 / size as f32;

                let r_base = (28.0 + 40.0 * norm_y) as u8;
                let g_base = (25.0 + 70.0 * norm_x + 30.0 * norm_y) as u8;
                let b_base = (85.0 + 140.0 * norm_y) as u8;

                let mut r = r_base;
                let mut g = g_base;
                let mut b = b_base;

                // Subtle inner border (glass highlight)
                if squircle_dist >= radius - 8.0 {
                    let border_factor = ((squircle_dist - (radius - 8.0)) / 8.0).clamp(0.0, 1.0);
                    r = (r as f32 * (1.0 - border_factor * 0.4) + 255.0 * border_factor * 0.4) as u8;
                    g = (g as f32 * (1.0 - border_factor * 0.4) + 255.0 * border_factor * 0.4) as u8;
                    b = (b as f32 * (1.0 - border_factor * 0.4) + 255.0 * border_factor * 0.4) as u8;
                }

                // Camera Lens Body (outer metallic circle around center)
                let cdx = x as f32 - center;
                let cdy = y as f32 - (center + 15.0);
                let lens_dist = (cdx * cdx + cdy * cdy).sqrt();

                // Viewfinder / Crop Marks on corners
                let in_crop_tl = (x >= 240 && x <= 340 && y >= 240 && y <= 270)
                    || (x >= 240 && x <= 270 && y >= 240 && y <= 340);
                let in_crop_tr = (x >= 684 && x <= 784 && y >= 240 && y <= 270)
                    || (x >= 754 && x <= 784 && y >= 240 && y <= 340);
                let in_crop_bl = (x >= 240 && x <= 340 && y >= 754 && y <= 784)
                    || (x >= 240 && x <= 270 && y >= 684 && y <= 784);
                let in_crop_br = (x >= 684 && x <= 784 && y >= 754 && y <= 784)
                    || (x >= 754 && x <= 784 && y >= 684 && y <= 784);

                if in_crop_tl || in_crop_tr || in_crop_bl || in_crop_br {
                    // Electric Cyan crop marks
                    r = 0;
                    g = 190;
                    b = 255;
                } else if lens_dist <= 220.0 {
                    if lens_dist >= 195.0 {
                        // Outer metal ring
                        let shade = 180 + (40.0 * (1.0 - (cdx / 200.0))) as u8;
                        r = shade;
                        g = shade;
                        b = shade.min(240);
                    } else if lens_dist >= 175.0 {
                        // Dark inset
                        r = 20;
                        g = 22;
                        b = 30;
                    } else if lens_dist >= 150.0 {
                        // Cyan aperture ring
                        r = 0;
                        g = 122;
                        b = 255;
                    } else if lens_dist >= 60.0 {
                        // Deep glass lens with reflection
                        let refl = if cdx + cdy < -40.0 && cdx + cdy > -160.0 { 90 } else { 15 };
                        r = 12 + refl;
                        g = 15 + refl;
                        b = 35 + refl;
                    } else {
                        // Center reflection dot
                        let mini_refl = if (cdx + 25.0).powi(2) + (cdy + 25.0).powi(2) < 250.0 { 240 } else { 20 };
                        r = mini_refl;
                        g = mini_refl;
                        b = (mini_refl as f32 * 1.05).min(255.0) as u8;
                    }
                }

                // Camera Flash Pill in top right
                let flash_dx = (x as f32 - 700.0).abs();
                let flash_dy = (y as f32 - 340.0).abs();
                if flash_dx <= 24.0 && flash_dy <= 24.0 && (flash_dx * flash_dx + flash_dy * flash_dy) <= 576.0 {
                    r = 255;
                    g = 204;
                    b = 0; // Amber / gold flash
                }

                let alpha = (255.0 * edge_alpha) as u8;
                img.put_pixel(x, y, Rgba([r, g, b, alpha]));
            }
        }
    }

    img.save("app_icon_1024.png").expect("Failed to save master icon");
    println!("✓ app_icon_1024.png başarıyla oluşturuldu.");
}
