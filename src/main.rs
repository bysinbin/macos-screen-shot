mod annotation;
mod capture;
mod export;
mod magnifier;
mod overlay;
mod pin;

use capture::CapturedScreen;
use eframe::egui;
use overlay::OverlayApp;
use pin::PinApp;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Check if user requested pinning an existing file: --pin <path>
    if args.len() >= 3 && args[1] == "--pin" {
        let path = PathBuf::from(&args[2]);
        let img = image::open(&path)?.to_rgba8();
        let w = img.width() as f32;
        let h = img.height() as f32;
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([w.min(1200.0), h.min(900.0)])
                .with_decorations(false)
                .with_always_on_top()
                .with_title("Pinned Image"),
            ..Default::default()
        };
        eframe::run_native(
            "Pinned Image",
            options,
            Box::new(move |_cc| Ok(Box::new(PinApp::new(img)))),
        )?;
        return Ok(());
    }

    // 1. Capture the screen first
    println!("📸 Ekran yakalanıyor...");
    let captured = match CapturedScreen::capture_primary() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Ekran yakalama hatası: {e}");
            eprintln!("💡 İpucu: macOS Sistem Ayarları > Gizlilik ve Güvenlik > Ekran Kaydı iznini kontrol edin.");
            std::process::exit(1);
        }
    };

    println!(
        "✓ Ekran yakalandı: {} ({}x{} px)",
        captured.monitor_name, captured.physical_width, captured.physical_height
    );

    let pin_img_holder: Arc<Mutex<Option<image::RgbaImage>>> = Arc::new(Mutex::new(None));
    let holder_clone = pin_img_holder.clone();

    // 2. Launch fullscreen overlay
    let overlay_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_active(true)
            .with_title("macOS Screenshot Overlay"),
        ..Default::default()
    };

    eframe::run_native(
        "macOS Screenshot",
        overlay_options,
        Box::new(move |_cc| {
            let app = OverlayApp::new(captured);
            Ok(Box::new(AppWrapper {
                app,
                pin_holder: holder_clone,
            }))
        }),
    )?;

    // 3. If user clicked "Pin" inside the overlay, spawn pinned window
    let maybe_pin_img = pin_img_holder.lock().unwrap().take();
    if let Some(pin_img) = maybe_pin_img {
        let pw = (pin_img.width() as f32 / 2.0).clamp(200.0, 1400.0);
        let ph = (pin_img.height() as f32 / 2.0).clamp(150.0, 1000.0);

        let pin_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([pw, ph])
                .with_decorations(false)
                .with_always_on_top()
                .with_title("Pinned Screenshot"),
            ..Default::default()
        };

        eframe::run_native(
            "Pinned Screenshot",
            pin_options,
            Box::new(move |_cc| Ok(Box::new(PinApp::new(pin_img)))),
        )?;
    }

    Ok(())
}

struct AppWrapper {
    app: OverlayApp,
    pin_holder: Arc<Mutex<Option<image::RgbaImage>>>,
}

impl eframe::App for AppWrapper {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
        if let Some(img) = self.app.pin_requested.take() {
            *self.pin_holder.lock().unwrap() = Some(img);
        }
    }
}
