use tray_icon::Icon;

pub fn create_camera_icon() -> Icon {
    let width = 32;
    let height = 32;
    let mut rgba = vec![0u8; width * height * 4];

    // Draw camera icon matching macOS template style (white on transparent)
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            // Camera body: rounded rect between x: 4..27, y: 10..25
            let in_body = x >= 4 && x <= 27 && y >= 10 && y <= 25;
            // Top bump (viewfinder/shutter): x: 11..20, y: 7..9
            let in_bump = x >= 11 && x <= 20 && y >= 7 && y <= 9;

            // Lens circle centered at (16, 17)
            let dx = (x as i32) - 16;
            let dy = (y as i32) - 17;
            let dist_sq = dx * dx + dy * dy;
            let in_lens_hole = dist_sq <= 25;
            let in_lens_center = dist_sq <= 7;

            // Small flash dot at (8, 13)
            let flash_dist_sq = ((x as i32) - 8).pow(2) + ((y as i32) - 13).pow(2);
            let in_flash = flash_dist_sq <= 2;

            if (in_body || in_bump) && (!in_lens_hole || in_lens_center) || in_flash {
                rgba[idx] = 255;
                rgba[idx + 1] = 255;
                rgba[idx + 2] = 255;
                rgba[idx + 3] = 255;
            }
        }
    }

    Icon::from_rgba(rgba, width as u32, height as u32).expect("Failed to create tray icon")
}
