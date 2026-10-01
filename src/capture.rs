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
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

pub fn has_screen_capture_permission() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        CGPreflightScreenCaptureAccess()
    }
    #[cfg(not(target_os = "macos"))]
    true
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
    pub fn capture_primary() -> Result<Self, String> {
        #[cfg(target_os = "macos")]
        if !has_screen_capture_permission() {
            let _ = request_screen_capture_permission();
            return Err("macOS Ekran Kaydı izni gerekli. Lütfen Sistem Ayarları > Gizlilik ve Güvenlik > Ekran Kaydı bölümünden ScreenShot için izin verin.".to_string());
        }

        let monitors = Monitor::all().map_err(|e| format!("Failed to list monitors: {e}"))?;
        if monitors.is_empty() {
            return Err("No monitors found".to_string());
        }

        // Find primary monitor, or fallback to the first one
        let primary = monitors
            .into_iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .or_else(|| Monitor::all().ok()?.into_iter().next())
            .ok_or_else(|| "Could not find a monitor to capture".to_string())?;

        let monitor_name = primary.name().unwrap_or_else(|_| "Primary Display".to_string());
        let image = primary
            .capture_image()
            .map_err(|e| format!("Failed to capture screen image: {e}"))?;

        let physical_width = image.width();
        let physical_height = image.height();

        // Safety check: is the captured screen pure black due to stale macOS TCC permission?
        let mut has_visible_pixels = false;
        for p in image.pixels().step_by(100) {
            if p[0] > 15 || p[1] > 15 || p[2] > 15 {
                has_visible_pixels = true;
                break;
            }
        }
        if !has_visible_pixels {
            #[cfg(target_os = "macos")]
            let _ = request_screen_capture_permission();
            return Err("Ekran yakalandı ancak görüntü tamamen siyah. Lütfen macOS Sistem Ayarları > Gizlilik ve Güvenlik > Ekran Kaydı bölümünden ScreenShot iznini kapatıp tekrar açın.".to_string());
        }

        // Scan windows for smart window snapping
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

        Ok(Self {
            image,
            monitor_name,
            physical_width,
            physical_height,
            windows: detected_windows,
        })
    }
}
