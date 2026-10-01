use crate::capture::CapturedScreen;
use crate::config::AppConfig;
use crate::export::{copy_to_clipboard, default_desktop_filename, save_to_file};
use crate::overlay::OverlayApp;
use crate::pin::PinApp;
use crate::settings::SettingsApp;
use eframe::egui;
use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use objc2_app_kit::{NSApp, NSApplicationActivationPolicy, NSEventMask};
use objc2_foundation::{MainThreadMarker, NSDate, NSDefaultRunLoopMode};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

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

#[allow(dead_code)]
pub struct TrayRunner {
    pub config: AppConfig,
    tray: TrayIcon,
    menu: Menu,
    item_area_id: muda::MenuId,
    item_fs_id: muda::MenuId,
    item_pin_id: muda::MenuId,
    item_settings_id: muda::MenuId,
    item_quit_id: muda::MenuId,
    hotkey_manager: GlobalHotKeyManager,
    area_hotkey: HotKey,
    fs_hotkey: HotKey,
    pin_hotkey: HotKey,
}

impl TrayRunner {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = AppConfig::load();
        let icon = create_camera_icon();

        let menu = Menu::new();

        let area_label = format!("📸 Ekran Alıntısı ({})", AppConfig::hotkey_full_label(&config.area_hotkey_modifier, &config.area_hotkey_key));
        let fs_label = format!("🖥️ Tüm Ekranı Yakala ({})", AppConfig::hotkey_full_label(&config.fullscreen_hotkey_modifier, &config.fullscreen_hotkey_key));
        let pin_label = format!("📌 Görsel Sabitle ({})", AppConfig::hotkey_full_label(&config.pin_hotkey_modifier, &config.pin_hotkey_key));

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

        menu.append(&item_area)?;
        menu.append(&item_fs)?;
        menu.append(&item_pin)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&item_settings)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&item_quit)?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("macOS Screenshot")
            .with_icon(icon)
            .build()?;

        let hotkey_manager = GlobalHotKeyManager::new().map_err(|e| format!("Hotkey init error: {e}"))?;

        let area_hotkey = config.to_area_hotkey();
        let fs_hotkey = config.to_fullscreen_hotkey();
        let pin_hotkey = config.to_pin_hotkey();

        let _ = hotkey_manager.register(area_hotkey);
        let _ = hotkey_manager.register(fs_hotkey);
        let _ = hotkey_manager.register(pin_hotkey);

        // Tell macOS LaunchServices that the application is ready and responsive
        if let Some(mtm) = MainThreadMarker::new() {
            let app = NSApp(mtm);
            let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
            app.finishLaunching();
        }

        Ok(Self {
            config,
            tray,
            menu,
            item_area_id,
            item_fs_id,
            item_pin_id,
            item_settings_id,
            item_quit_id,
            hotkey_manager,
            area_hotkey,
            fs_hotkey,
            pin_hotkey,
        })
    }

    pub fn reload_hotkeys(&mut self) {
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

        println!(
            "✓ Kısayollar güncellendi: Bölge: {}, Tam Ekran: {}, Sabitle: {}",
            AppConfig::hotkey_full_label(&self.config.area_hotkey_modifier, &self.config.area_hotkey_key),
            AppConfig::hotkey_full_label(&self.config.fullscreen_hotkey_modifier, &self.config.fullscreen_hotkey_key),
            AppConfig::hotkey_full_label(&self.config.pin_hotkey_modifier, &self.config.pin_hotkey_key),
        );
    }

    pub fn trigger_area_capture(&mut self) {
        println!("📸 Ekran alıntısı başlatılıyor...");
        match CapturedScreen::capture_primary() {
            Ok(captured) => {
                let pin_holder: Arc<Mutex<Option<image::RgbaImage>>> = Arc::new(Mutex::new(None));
                let pin_holder_clone = pin_holder.clone();

                let overlay_options = eframe::NativeOptions {
                    viewport: egui::ViewportBuilder::default()
                        .with_fullscreen(true)
                        .with_decorations(false)
                        .with_always_on_top()
                        .with_active(true)
                        .with_title("macOS Screenshot Overlay"),
                    ..Default::default()
                };

                let _ = eframe::run_native(
                    "macOS Screenshot",
                    overlay_options,
                    Box::new(move |_cc| {
                        let app = OverlayApp::new(captured);
                        Ok(Box::new(OverlayWrapper {
                            app,
                            pin_holder: pin_holder_clone,
                        }))
                    }),
                );

                // If user pinned a region inside overlay
                if let Some(pin_img) = pin_holder.lock().unwrap().take() {
                    Self::spawn_pin_window(pin_img);
                }
            }
            Err(e) => {
                eprintln!("❌ Ekran yakalama hatası: {e}");
            }
        }
    }

    pub fn trigger_fullscreen_capture(&self) {
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
                eprintln!("❌ Hata: {e}");
            }
        }
    }

    pub fn open_settings(&mut self) {
        println!("⚙️ Ayarlar penceresi açılıyor...");
        let save_flag = Arc::new(Mutex::new(false));
        let flag_clone = save_flag.clone();
        let cfg = self.config.clone();

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([560.0, 520.0])
                .with_resizable(false)
                .with_title("Ekran Alıntısı Ayarları"),
            ..Default::default()
        };

        let _ = eframe::run_native(
            "Ekran Alıntısı Ayarları",
            options,
            Box::new(move |_cc| Ok(Box::new(SettingsApp::new(cfg, Some(flag_clone))))),
        );

        if *save_flag.lock().unwrap() {
            self.reload_hotkeys();
        }
    }

    pub fn spawn_pin_window(image: image::RgbaImage) {
        let pw = (image.width() as f32 / 2.0).clamp(200.0, 1400.0);
        let ph = (image.height() as f32 / 2.0).clamp(150.0, 1000.0);

        let pin_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([pw, ph])
                .with_decorations(false)
                .with_always_on_top()
                .with_title("Pinned Screenshot"),
            ..Default::default()
        };

        let _ = eframe::run_native(
            "Pinned Screenshot",
            pin_options,
            Box::new(move |_cc| Ok(Box::new(PinApp::new(image)))),
        );
    }

    pub fn run_loop(&mut self) {
        println!("🚀 macOS Menubar arka plan servisi aktif!");
        println!(
            "💡 Kısayol: {} tuşlarına basarak her an ekran alıntısı alabilirsiniz.",
            AppConfig::hotkey_full_label(&self.config.area_hotkey_modifier, &self.config.area_hotkey_key)
        );

        let mtm = MainThreadMarker::new();

        loop {
            // Pump macOS NSApplication event loop so LaunchServices and WindowServer know the app is alive & responsive!
            if let Some(m) = mtm {
                let app = NSApp(m);
                let until = NSDate::dateWithTimeIntervalSinceNow(0.02);
                while let Some(event) = app.nextEventMatchingMask_untilDate_inMode_dequeue(
                    NSEventMask::Any,
                    Some(&until),
                    unsafe { NSDefaultRunLoopMode },
                    true,
                ) {
                    app.sendEvent(&event);
                }
                app.updateWindows();
            }

            // Process tray menu clicks
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == self.item_area_id {
                    self.trigger_area_capture();
                } else if event.id == self.item_fs_id {
                    self.trigger_fullscreen_capture();
                } else if event.id == self.item_pin_id {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                    {
                        if let Ok(img) = image::open(&path) {
                            Self::spawn_pin_window(img.to_rgba8());
                        }
                    }
                } else if event.id == self.item_settings_id {
                    self.open_settings();
                } else if event.id == self.item_quit_id {
                    println!("👋 Çıkış yapılıyor...");
                    break;
                }
            }

            // Process global hotkeys
            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.state == HotKeyState::Pressed {
                    if event.id == self.area_hotkey.id() {
                        self.trigger_area_capture();
                    } else if event.id == self.fs_hotkey.id() {
                        self.trigger_fullscreen_capture();
                    } else if event.id == self.pin_hotkey.id() {
                        self.trigger_area_capture();
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(15));
        }
    }
}

struct OverlayWrapper {
    app: OverlayApp,
    pin_holder: Arc<Mutex<Option<image::RgbaImage>>>,
}

impl eframe::App for OverlayWrapper {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
        if let Some(img) = self.app.pin_requested.take() {
            *self.pin_holder.lock().unwrap() = Some(img);
        }
    }
}
