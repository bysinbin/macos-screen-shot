mod annotation;
mod capture;
mod config;
mod export;
mod magnifier;
mod overlay;
mod pin;
mod settings;
mod tray;

use capture::CapturedScreen;
use config::AppConfig;
use export::{copy_to_clipboard, default_desktop_filename, save_to_file};
use overlay::OverlayApp;
use pin::PinApp;
use settings::SettingsApp;
use tray::create_camera_icon;

use eframe::egui::{self, ViewportCommand};
use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use objc2_app_kit::{NSApp, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tray_icon::{TrayIcon, TrayIconBuilder};

enum AppState {
    Idle,
    Overlay(OverlayApp),
    Settings(SettingsApp),
    Pin(PinApp),
}

struct MainApp {
    config: AppConfig,
    _tray: TrayIcon,
    hotkey_manager: GlobalHotKeyManager,
    area_hotkey: HotKey,
    fs_hotkey: HotKey,
    pin_hotkey: HotKey,
    item_area_id: muda::MenuId,
    item_fs_id: muda::MenuId,
    item_pin_id: muda::MenuId,
    item_settings_id: muda::MenuId,
    item_quit_id: muda::MenuId,
    state: AppState,
    settings_save_flag: Option<Arc<Mutex<bool>>>,
    should_quit: bool,
}

impl MainApp {
    fn reload_config(&mut self) {
        let _ = self.hotkey_manager.unregister(self.area_hotkey);
        let _ = self.hotkey_manager.unregister(self.fs_hotkey);
        let _ = self.hotkey_manager.unregister(self.pin_hotkey);

        self.config = AppConfig::load();
        self.area_hotkey = self.config.to_area_hotkey();
        self.fs_hotkey = self.config.to_fullscreen_hotkey();
        self.pin_hotkey = self.config.to_pin_hotkey();

        let _ = self.hotkey_manager.register(self.area_hotkey);
        let _ = self.hotkey_manager.register(self.fs_hotkey);
        let _ = self.hotkey_manager.register(self.pin_hotkey);
        println!("✓ Hotkeys updated from config!");
    }

    fn trigger_area_capture(&mut self, ctx: &egui::Context, from_menu: bool) {
        if from_menu {
            std::thread::sleep(Duration::from_millis(150));
        }
        println!("📸 Ekran alıntısı başlatılıyor...");
        match CapturedScreen::capture_primary() {
            Ok(captured) => {
                println!(
                    "✓ Ekran başarıyla yakalandı: {}x{} piksel",
                    captured.physical_width, captured.physical_height
                );

                let scale = ctx.pixels_per_point().max(1.0);
                let screen_w = captured.physical_width as f32 / scale;
                let screen_h = captured.physical_height as f32 / scale;

                self.state = AppState::Overlay(OverlayApp::new(captured));

                // Force macOS to bring the application to the foreground
                if let Some(mtm) = MainThreadMarker::new() {
                    let app = NSApp(mtm);
                    app.activate();
                }

                ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
                ctx.send_viewport_cmd(ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(egui::pos2(0.0, 0.0)));
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(screen_w, screen_h)));
                ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(ViewportCommand::Focus);
            }
            Err(e) => {
                eprintln!("❌ Ekran yakalama hatası: {e}");
            }
        }
    }

    fn trigger_fullscreen_capture(&self, from_menu: bool) {
        if from_menu {
            std::thread::sleep(Duration::from_millis(150));
        }
        println!("🖥️ Tam ekran yakalanıyor...");
        match CapturedScreen::capture_primary() {
            Ok(captured) => {
                if self.config.auto_copy_to_clipboard {
                    let _ = copy_to_clipboard(&captured.image);
                    println!("✓ Tam ekran panoya kopyalandı.");
                }
                if self.config.auto_save_to_folder {
                    let dir = PathBuf::from(&self.config.save_directory);
                    let now = chrono::Local::now();
                    let filename = format!("Tam Ekran {}.png", now.format("%Y-%m-%d %H.%M.%S"));
                    let path = dir.join(filename);
                    if let Ok(()) = save_to_file(&captured.image, &path) {
                        println!("✓ Kaydedildi: {}", path.display());
                    }
                } else {
                    let path = default_desktop_filename();
                    let _ = save_to_file(&captured.image, &path);
                    println!("✓ Masaüstüne kaydedildi: {}", path.display());
                }
            }
            Err(e) => {
                eprintln!("❌ Tam ekran hatası: {e}");
            }
        }
    }

    fn open_settings(&mut self, ctx: &egui::Context) {
        let flag = Arc::new(Mutex::new(false));
        self.settings_save_flag = Some(flag.clone());
        self.state = AppState::Settings(SettingsApp::new(self.config.clone(), Some(flag)));

        if let Some(mtm) = MainThreadMarker::new() {
            let app = NSApp(mtm);
            app.activate();
        }

        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(560.0, 520.0)));
        ctx.send_viewport_cmd(ViewportCommand::Resizable(false));
        ctx.send_viewport_cmd(ViewportCommand::Title("Ekran Alıntısı Ayarları".into()));
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }
}

impl eframe::App for MainApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Handle window close request
        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.should_quit {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                self.state = AppState::Idle;
                ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
            }
        }

        // 1. Process Tray Menu Events
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.item_area_id {
                self.trigger_area_capture(&ctx, true);
            } else if event.id == self.item_fs_id {
                self.trigger_fullscreen_capture(true);
            } else if event.id == self.item_pin_id {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                {
                    if let Ok(img) = image::open(&path) {
                        let rgba = img.to_rgba8();
                        let pw = (rgba.width() as f32 / 2.0).clamp(200.0, 1400.0);
                        let ph = (rgba.height() as f32 / 2.0).clamp(150.0, 1000.0);
                        self.state = AppState::Pin(PinApp::new(rgba));
                        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
                        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
                        ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(pw, ph)));
                        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                        ctx.send_viewport_cmd(ViewportCommand::Focus);
                    }
                }
            } else if event.id == self.item_settings_id {
                self.open_settings(&ctx);
            } else if event.id == self.item_quit_id {
                self.should_quit = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
                return;
            }
        }

        // 2. Process Global Hotkeys
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == HotKeyState::Pressed {
                if event.id == self.area_hotkey.id() {
                    self.trigger_area_capture(&ctx, false);
                } else if event.id == self.fs_hotkey.id() {
                    self.trigger_fullscreen_capture(false);
                } else if event.id == self.pin_hotkey.id() {
                    self.trigger_area_capture(&ctx, false);
                }
            }
        }

        // 3. Render state
        match &mut self.state {
            AppState::Idle => {
                // Window is invisible, nothing rendered
            }
            AppState::Overlay(overlay) => {
                overlay.ui(ui, frame);

                if overlay.is_finished {
                    println!("✓ Overlay kapandı.");
                    if let Some(pin_img) = overlay.pin_requested.take() {
                        let pw = (pin_img.width() as f32 / 2.0).clamp(200.0, 1400.0);
                        let ph = (pin_img.height() as f32 / 2.0).clamp(150.0, 1000.0);
                        self.state = AppState::Pin(PinApp::new(pin_img));
                        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
                        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
                        ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(pw, ph)));
                        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                        ctx.send_viewport_cmd(ViewportCommand::Focus);
                    } else {
                        self.state = AppState::Idle;
                        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
                        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                    }
                }
            }
            AppState::Settings(settings) => {
                settings.ui(ui, frame);

                if settings.is_closed {
                    if let Some(flag) = &self.settings_save_flag {
                        if *flag.lock().unwrap() {
                            self.reload_config();
                        }
                    }
                    self.state = AppState::Idle;
                    ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                }
            }
            AppState::Pin(pin) => {
                pin.ui(ui, frame);

                if pin.is_closed {
                    self.state = AppState::Idle;
                    ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                }
            }
        }

        // Keep loop alive for hotkeys & menu polling
        ctx.request_repaint_after(Duration::from_millis(40));
    }
}

fn main() -> eframe::Result {
    let args: Vec<String> = env::args().collect();

    let start_mode_capture = args.len() >= 2 && args[1] == "--capture";
    let start_mode_settings = args.len() >= 2 && args[1] == "--settings";

    let (init_w, init_h) = if let Ok(monitors) = xcap::Monitor::all() {
        if let Some(m) = monitors.first() {
            let w = m.width().unwrap_or(2560) as f32 / 2.0;
            let h = m.height().unwrap_or(1600) as f32 / 2.0;
            (w, h)
        } else {
            (1280.0, 800.0)
        }
    } else {
        (1280.0, 800.0)
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_visible(start_mode_capture || start_mode_settings)
            .with_decorations(start_mode_settings)
            .with_inner_size(if start_mode_settings { [560.0, 520.0] } else { [init_w, init_h] })
            .with_position([0.0, 0.0])
            .with_always_on_top()
            .with_title("ScreenShot"),
        ..Default::default()
    };

    eframe::run_native(
        "ScreenShot",
        native_options,
        Box::new(move |cc| {
            if let Some(mtm) = MainThreadMarker::new() {
                let app = NSApp(mtm);
                let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
                app.finishLaunching();
            }

            let config = AppConfig::load();
            let icon = create_camera_icon();

            let menu = Menu::new();
            let area_label = format!(
                "📸 Ekran Alıntısı ({})",
                AppConfig::hotkey_full_label(&config.area_hotkey_modifier, &config.area_hotkey_key)
            );
            let fs_label = format!(
                "🖥️ Tüm Ekranı Yakala ({})",
                AppConfig::hotkey_full_label(&config.fullscreen_hotkey_modifier, &config.fullscreen_hotkey_key)
            );
            let pin_label = format!(
                "📌 Görsel Sabitle ({})",
                AppConfig::hotkey_full_label(&config.pin_hotkey_modifier, &config.pin_hotkey_key)
            );

            let item_area = MenuItem::new(area_label, true, None);
            let item_fs = MenuItem::new(fs_label, true, None);
            let item_pin = MenuItem::new(pin_label, true, None);
            let item_settings = MenuItem::new("⚙️ Ayarlar & Kısayollar...", true, None);
            let item_quit = MenuItem::new("❌ Çıkış", true, None);

            let item_area_id = item_area.id().clone();
            let item_fs_id = item_fs.id().clone();
            let item_pin_id = item_pin.id().clone();
            let item_settings_id = item_settings.id().clone();
            let item_quit_id = item_quit.id().clone();

            menu.append(&item_area).unwrap();
            menu.append(&item_fs).unwrap();
            menu.append(&item_pin).unwrap();
            menu.append(&PredefinedMenuItem::separator()).unwrap();
            menu.append(&item_settings).unwrap();
            menu.append(&PredefinedMenuItem::separator()).unwrap();
            menu.append(&item_quit).unwrap();

            let tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("ScreenShot")
                .with_icon(icon)
                .build()
                .expect("Failed to build tray icon");

            let hotkeys = GlobalHotKeyManager::new().expect("Failed to init hotkeys");
            let area_hotkey = config.to_area_hotkey();
            let fs_hotkey = config.to_fullscreen_hotkey();
            let pin_hotkey = config.to_pin_hotkey();

            if let Err(e) = hotkeys.register(area_hotkey) {
                eprintln!("❌ Area hotkey register error: {e}");
            } else {
                println!("✓ Registered area_hotkey: {:?}", area_hotkey);
            }
            let _ = hotkeys.register(fs_hotkey);
            let _ = hotkeys.register(pin_hotkey);

            let mut initial_state = AppState::Idle;
            if start_mode_capture {
                match CapturedScreen::capture_primary() {
                    Ok(captured) => {
                        println!(
                            "✓ CLI --capture: Ekran yakalandı ({}x{})",
                            captured.physical_width, captured.physical_height
                        );
                        let w = captured.physical_width as f32 / 2.0;
                        let h = captured.physical_height as f32 / 2.0;
                        initial_state = AppState::Overlay(OverlayApp::new(captured));

                        if let Some(mtm) = MainThreadMarker::new() {
                            let app = NSApp(mtm);
                            app.activate();
                        }

                        cc.egui_ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
                        cc.egui_ctx.send_viewport_cmd(ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                        cc.egui_ctx.send_viewport_cmd(ViewportCommand::OuterPosition(egui::pos2(0.0, 0.0)));
                        cc.egui_ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(w, h)));
                        cc.egui_ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                        cc.egui_ctx.send_viewport_cmd(ViewportCommand::Focus);
                    }
                    Err(e) => {
                        eprintln!("❌ CLI --capture hatası: {e}");
                    }
                }
            } else if start_mode_settings {
                let flag = Arc::new(Mutex::new(false));
                initial_state = AppState::Settings(SettingsApp::new(config.clone(), Some(flag)));
                if let Some(mtm) = MainThreadMarker::new() {
                    let app = NSApp(mtm);
                    app.activate();
                }
                cc.egui_ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(560.0, 520.0)));
                cc.egui_ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                cc.egui_ctx.send_viewport_cmd(ViewportCommand::Focus);
            }

            println!("══════════════════════════════════════════════════");
            println!("  📸 macOS Screen Shot - Menubar Servisi Aktif!");
            println!("══════════════════════════════════════════════════");

            Ok(Box::new(MainApp {
                config,
                _tray: tray,
                hotkey_manager: hotkeys,
                area_hotkey,
                fs_hotkey,
                pin_hotkey,
                item_area_id,
                item_fs_id,
                item_pin_id,
                item_settings_id,
                item_quit_id,
                state: initial_state,
                settings_save_flag: None,
                should_quit: false,
            }))
        }),
    )
}
