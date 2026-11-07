use crossbeam_channel::Receiver;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
use std::time::Duration;

pub struct SmtcBridge {
    controls: MediaControls,
    event_rx: Receiver<MediaControlEvent>,
}

#[cfg(windows)]
fn create_dummy_window() -> Option<*mut std::ffi::c_void> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, RegisterClassW, WNDCLASSW,
    };

    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        let class_name: Vec<u16> = "LunulaSMTCClass\0".encode_utf16().collect();
        let window_name: Vec<u16> = "Lunula SMTC Bridge\0".encode_utf16().collect();

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(DefWindowProcW),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };

        RegisterClassW(&wc);

        // HWND_MESSAGE is -3 in Win32
        let hwnd_message = (-3isize) as HWND;
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            hwnd_message,
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null_mut(),
        );

        if hwnd.is_null() {
            let fallback_hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null_mut(),
            );
            if fallback_hwnd.is_null() {
                None
            } else {
                Some(fallback_hwnd as *mut std::ffi::c_void)
            }
        } else {
            Some(hwnd as *mut std::ffi::c_void)
        }
    }
}

impl SmtcBridge {
    pub fn new() -> Option<Self> {
        #[cfg(windows)]
        let hwnd = create_dummy_window();

        #[cfg(not(windows))]
        let hwnd = None;

        let config = PlatformConfig {
            dbus_name: "lunula_audio_player",
            display_name: "Lunula Audio Player",
            hwnd,
        };

        let mut controls = MediaControls::new(config).ok()?;
        let (tx, rx) = crossbeam_channel::unbounded();

        let attach_res = controls.attach(move |event| {
            let _ = tx.send(event);
        });

        if attach_res.is_err() {
            return None;
        }

        Some(Self {
            controls,
            event_rx: rx,
        })
    }

    pub fn event_receiver(&self) -> &Receiver<MediaControlEvent> {
        &self.event_rx
    }

    pub fn update_metadata(&mut self, title: &str, artist: &str, album: &str, duration_secs: f32, cover_path: Option<&str>) {
        let dur = if duration_secs > 0.0 {
            Some(Duration::from_secs_f32(duration_secs))
        } else {
            None
        };

        let _ = self.controls.set_metadata(MediaMetadata {
            title: Some(title),
            artist: Some(artist),
            album: Some(album),
            duration: dur,
            cover_url: cover_path,
        });
    }

    pub fn update_playback_state(&mut self, is_playing: bool, position_secs: f32) {
        let progress = if position_secs >= 0.0 {
            Some(MediaPosition(Duration::from_secs_f32(position_secs)))
        } else {
            None
        };

        let playback = if is_playing {
            MediaPlayback::Playing { progress }
        } else {
            MediaPlayback::Paused { progress }
        };

        let _ = self.controls.set_playback(playback);
    }
}
