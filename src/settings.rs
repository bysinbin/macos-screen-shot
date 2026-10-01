use crate::config::AppConfig;
use eframe::egui::{self, Color32, RichText};
use std::sync::{Arc, Mutex};

pub struct SettingsApp {
    pub config: AppConfig,
    saved_notification: Option<std::time::Instant>,
    pub on_save_callback: Option<Arc<Mutex<bool>>>,
}

impl SettingsApp {
    pub fn new(config: AppConfig, on_save_flag: Option<Arc<Mutex<bool>>>) -> Self {
        Self {
            config,
            saved_notification: None,
            on_save_callback: on_save_flag,
        }
    }
}

impl eframe::App for SettingsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ui.add_space(8.0);

            // Title Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚙️").size(24.0));
                ui.vertical(|ui| {
                    ui.label(RichText::new("Ekran Alıntısı Ayarları").size(18.0).strong());
                    ui.label(RichText::new("Kısayolları ve genel tercihleri buradan yapılandırabilirsiniz.").size(12.0).color(Color32::from_white_alpha(150)));
                });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Section 1: Keyboard Shortcuts
                ui.label(RichText::new("⌨️ Genel Kısayollar (Global Hotkeys)").size(15.0).strong().color(Color32::from_rgb(0, 122, 255)));
                ui.label(RichText::new("Uygulama arka planda Menubar'da çalışırken bu tuşlara basarak anında tetikleyebilirsiniz.").size(11.5).color(Color32::from_white_alpha(140)));
                ui.add_space(8.0);

                let modifier_options = [
                    ("ALT", "⌥ (Option)"),
                    ("CMD_SHIFT", "⌘ + ⇧ (Cmd + Shift)"),
                    ("CTRL_ALT", "⌃ + ⌥ (Ctrl + Option)"),
                    ("CMD_ALT", "⌘ + ⌥ (Cmd + Option)"),
                ];

                let key_options = [
                    ("KeyA", "A"),
                    ("KeyS", "S"),
                    ("KeyX", "X"),
                    ("KeyP", "P"),
                    ("KeyR", "R"),
                    ("KeyC", "C"),
                    ("Digit1", "1"),
                    ("Digit2", "2"),
                    ("Digit3", "3"),
                    ("Digit4", "4"),
                ];

                // Area capture shortcut row
                egui::Frame::group(ui.style()).corner_radius(8.0).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📸 Bölge Seçimi Alıntısı:").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let preview = AppConfig::hotkey_full_label(&self.config.area_hotkey_modifier, &self.config.area_hotkey_key);
                            ui.label(RichText::new(format!("  {}  ", preview)).strong().size(13.0).color(Color32::WHITE).background_color(Color32::from_rgb(0, 122, 255)));

                            egui::ComboBox::from_id_salt("area_key")
                                .selected_text(AppConfig::key_display(&self.config.area_hotkey_key))
                                .show_ui(ui, |ui| {
                                    for (val, label) in key_options {
                                        ui.selectable_value(&mut self.config.area_hotkey_key, val.to_string(), label);
                                    }
                                });

                            egui::ComboBox::from_id_salt("area_mod")
                                .selected_text(AppConfig::modifier_display(&self.config.area_hotkey_modifier))
                                .show_ui(ui, |ui| {
                                    for (val, label) in modifier_options {
                                        ui.selectable_value(&mut self.config.area_hotkey_modifier, val.to_string(), label);
                                    }
                                });
                        });
                    });
                });

                ui.add_space(4.0);

                // Fullscreen shortcut row
                egui::Frame::group(ui.style()).corner_radius(8.0).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🖥️ Tüm Ekranı Yakala:").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let preview = AppConfig::hotkey_full_label(&self.config.fullscreen_hotkey_modifier, &self.config.fullscreen_hotkey_key);
                            ui.label(RichText::new(format!("  {}  ", preview)).strong().size(13.0).color(Color32::WHITE).background_color(Color32::from_rgb(52, 199, 89)));

                            egui::ComboBox::from_id_salt("fs_key")
                                .selected_text(AppConfig::key_display(&self.config.fullscreen_hotkey_key))
                                .show_ui(ui, |ui| {
                                    for (val, label) in key_options {
                                        ui.selectable_value(&mut self.config.fullscreen_hotkey_key, val.to_string(), label);
                                    }
                                });

                            egui::ComboBox::from_id_salt("fs_mod")
                                .selected_text(AppConfig::modifier_display(&self.config.fullscreen_hotkey_modifier))
                                .show_ui(ui, |ui| {
                                    for (val, label) in modifier_options {
                                        ui.selectable_value(&mut self.config.fullscreen_hotkey_modifier, val.to_string(), label);
                                    }
                                });
                        });
                    });
                });

                ui.add_space(4.0);

                // Pin shortcut row
                egui::Frame::group(ui.style()).corner_radius(8.0).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📌 Görsel Sabitle (Pin):").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let preview = AppConfig::hotkey_full_label(&self.config.pin_hotkey_modifier, &self.config.pin_hotkey_key);
                            ui.label(RichText::new(format!("  {}  ", preview)).strong().size(13.0).color(Color32::WHITE).background_color(Color32::from_rgb(88, 86, 214)));

                            egui::ComboBox::from_id_salt("pin_key")
                                .selected_text(AppConfig::key_display(&self.config.pin_hotkey_key))
                                .show_ui(ui, |ui| {
                                    for (val, label) in key_options {
                                        ui.selectable_value(&mut self.config.pin_hotkey_key, val.to_string(), label);
                                    }
                                });

                            egui::ComboBox::from_id_salt("pin_mod")
                                .selected_text(AppConfig::modifier_display(&self.config.pin_hotkey_modifier))
                                .show_ui(ui, |ui| {
                                    for (val, label) in modifier_options {
                                        ui.selectable_value(&mut self.config.pin_hotkey_modifier, val.to_string(), label);
                                    }
                                });
                        });
                    });
                });

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);

                // Section 2: General Options
                ui.label(RichText::new("⚙️ Genel Seçenekler").size(15.0).strong().color(Color32::from_rgb(0, 122, 255)));
                ui.add_space(6.0);

                ui.checkbox(&mut self.config.auto_copy_to_clipboard, "📋 Alıntı tamamlandığında panoya otomatik kopyala");
                ui.checkbox(&mut self.config.auto_save_to_folder, "💾 Alıntı tamamlandığında belirlenen klasöre otomatik kaydet");

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("📁 Kayıt Klasörü:");
                    ui.text_edit_singleline(&mut self.config.save_directory);
                    if ui.button("Klasör Seç...").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.config.save_directory = folder.to_string_lossy().to_string();
                        }
                    }
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("🖼️ Görsel Biçimi:");
                    ui.radio_value(&mut self.config.image_format, "PNG".to_string(), "PNG (Kayıpsız / Şeffaf)");
                    ui.radio_value(&mut self.config.image_format, "JPEG".to_string(), "JPEG");
                });

                ui.checkbox(&mut self.config.show_magnifier, "🔍 Seçim öncesi piksel büyütecini (Loupe) göster");

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);

                // Section 3: In-Capture Cheatsheet
                ui.label(RichText::new("📖 Alıntı Esnasındaki Kısayollar").size(14.0).strong());
                ui.add_space(4.0);
                egui::Grid::new("cheatsheet_grid").striped(true).spacing([20.0, 6.0]).show(ui, |ui| {
                    ui.label(RichText::new("Enter veya Cmd+C").strong());
                    ui.label("Seçimi çizimlerle birlikte panoya kopyalar ve kapatır");
                    ui.end_row();

                    ui.label(RichText::new("Space (Boşluk)").strong());
                    ui.label("İmleç pencere üzerindeyse pencereyi seçer / Seçim varsa masaüstüne kaydeder");
                    ui.end_row();

                    ui.label(RichText::new("C").strong());
                    ui.label("Büyüteç altındaki pikselin HEX renk kodunu panoya kopyalar");
                    ui.end_row();

                    ui.label(RichText::new("R, O, A, P").strong());
                    ui.label("Sırasıyla Dikdörtgen, Daire, Ok, Kalem çizim araçları");
                    ui.end_row();

                    ui.label(RichText::new("N, B, H").strong());
                    ui.label("Adım Rozeti (1,2,3), Buzlama/Mozaik, Vurgulayıcı");
                    ui.end_row();

                    ui.label(RichText::new("Cmd+Z / Esc").strong());
                    ui.label("Çizimi geri alma / Alıntıyı iptal etme");
                    ui.end_row();
                });
            });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(8.0);

            // Action Buttons
            ui.horizontal(|ui| {
                let save_btn = egui::Button::new(
                    RichText::new("💾 Kaydet ve Kısayolları Güncelle")
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0, 122, 255))
                .corner_radius(6.0);

                if ui.add(save_btn).clicked() {
                    if let Ok(()) = self.config.save() {
                        self.saved_notification = Some(std::time::Instant::now());
                        if let Some(cb) = &self.on_save_callback {
                            *cb.lock().unwrap() = true;
                        }
                    }
                }

                if ui.button("Kapat").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                if let Some(t) = self.saved_notification {
                    if t.elapsed().as_secs_f32() < 2.5 {
                        ui.label(
                            RichText::new("✓ Ayarlar başarıyla kaydedildi!")
                                .color(Color32::from_rgb(52, 199, 89))
                                .strong(),
                        );
                    } else {
                        self.saved_notification = None;
                    }
                }
            });
    }
}
