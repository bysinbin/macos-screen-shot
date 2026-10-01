use image::RgbaImage;
use xcap::{Monitor, Window};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub app_name: String,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
pub struct CapturedScreen {
    pub image: RgbaImage,
    pub monitor_name: String,
    pub physical_width: u32,
    pub physical_height: u32,
    pub windows: Vec<WindowInfo>,
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGRequestScreenCaptureAccess() -> bool;
}

pub fn request_screen_capture_permission() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        CGRequestScreenCaptureAccess()
    }
    #[cfg(not(target_os = "macos"))]
    true
}

impl CapturedScreen {
    pub fn detect_windows() -> Vec<WindowInfo> {
        let mut detected_windows = Vec::new();
        if let Ok(windows) = Window::all() {
            for w in windows {
                if let Ok(is_min) = w.is_minimized() {
                    if is_min {
                        continue;
                    }
                }
                let width = w.width().unwrap_or_default();
                let height = w.height().unwrap_or_default();
                if width < 50 || height < 50 {
                    continue;
                }
                let app_name = w.app_name().unwrap_or_default();
                let title = w.title().unwrap_or_default();
                let x = w.x().unwrap_or_default();
                let y = w.y().unwrap_or_default();

                detected_windows.push(WindowInfo {
                    app_name,
                    title,
                    x,
                    y,
                    width,
                    height,
                });
            }
        }
        detected_windows
    }

    pub fn capture_primary() -> Result<Self, String> {
        // Method 1: Use macOS native screencapture utility.
        // This is 100% reliable across Spaces, Fullscreen Apps (like Antigravity),
        // and doesn't suffer from TCC ad-hoc window-redaction bugs.
        #[cfg(target_os = "macos")]
        {
            let tmp_path = std::env::temp_dir().join(format!("scr_shot_{}.png", std::process::id()));
            let output = std::process::Command::new("/usr/sbin/screencapture")
                .arg("-x")
                .arg(&tmp_path)
                .output();

            if let Ok(out) = output {
                if out.status.success() && tmp_path.exists() {
                    if let Ok(dyn_img) = image::open(&tmp_path) {
                        let _ = std::fs::remove_file(&tmp_path);
                        let rgba = dyn_img.to_rgba8();
                        let pw = rgba.width();
                        let ph = rgba.height();

                        let detected_windows = Self::detect_windows();

                        return Ok(Self {
                            image: rgba,
                            monitor_name: "Primary Display".to_string(),
                            physical_width: pw,
                            physical_height: ph,
                            windows: detected_windows,
                        });
                    }
                    let _ = std::fs::remove_file(&tmp_path);
                }
            }
        }

        // Method 2: Fallback to xcap
        let monitors = Monitor::all().map_err(|e| {
            #[cfg(target_os = "macos")]
            let _ = request_screen_capture_permission();
            format!("Monitör listesi alınamadı: {e}")
        })?;
        if monitors.is_empty() {
            return Err("Monitör bulunamadı".to_string());
        }

        let primary = monitors
            .into_iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .or_else(|| Monitor::all().ok()?.into_iter().next())
            .ok_or_else(|| "Could not find a monitor to capture".to_string())?;

        let monitor_name = primary.name().unwrap_or_else(|_| "Primary Display".to_string());
        let image = primary
            .capture_image()
            .map_err(|e| {
                #[cfg(target_os = "macos")]
                let _ = request_screen_capture_permission();
                format!("Ekran görüntüsü yakalanamadı: {e}")
            })?;

        let physical_width = image.width();
        let physical_height = image.height();

        let detected_windows = Self::detect_windows();

        Ok(Self {
            image,
            monitor_name,
            physical_width,
            physical_height,
            windows: detected_windows,
        })
    }
}
