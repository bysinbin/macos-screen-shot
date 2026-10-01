mod annotation;
mod capture;
mod config;
mod export;
mod magnifier;
mod overlay;
mod pin;
mod settings;
mod tray;

use config::AppConfig;
use settings::SettingsApp;
use std::env;
use std::path::PathBuf;
use tray::TrayRunner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // 1. Direct Pin Mode: --pin <file>
    if args.len() >= 3 && args[1] == "--pin" {
        let path = PathBuf::from(&args[2]);
        let img = image::open(&path)?.to_rgba8();
        TrayRunner::spawn_pin_window(img);
        return Ok(());
    }

    // 2. Direct Settings Mode: --settings
    if args.len() >= 2 && args[1] == "--settings" {
        let config = AppConfig::load();
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([560.0, 520.0])
                .with_resizable(false)
                .with_title("Ekran Alıntısı Ayarları"),
            ..Default::default()
        };
        eframe::run_native(
            "Ekran Alıntısı Ayarları",
            options,
            Box::new(move |_cc| Ok(Box::new(SettingsApp::new(config, None)))),
        )?;
        return Ok(());
    }

    // 3. Direct Capture Mode: --capture
    if args.len() >= 2 && args[1] == "--capture" {
        let mut runner = TrayRunner::new()?;
        runner.trigger_area_capture();
        return Ok(());
    }

    // 4. Default: Launch macOS Menubar Status Item & Global Hotkey Daemon
    println!("══════════════════════════════════════════════════");
    println!("  📸 macOS Screen Shot - Menubar Servisi Başladı");
    println!("══════════════════════════════════════════════════");

    let mut runner = TrayRunner::new()?;
    runner.run_loop();

    Ok(())
}
