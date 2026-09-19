use std::num::NonZeroU32;

/// Controls how the window's contents are scaled/positioned when the OS
/// window is resized.
#[derive(Default)]
pub enum ResizeMode {
    /// Keeps the buffer at its original size and anchors it to the
    /// upper-left corner, leaving any extra window space blank rather than
    /// scaling the contents. Default variant.
    #[default]
    Trim,
    /// Stretches the buffer to exactly fill the new window size, ignoring
    /// aspect ratio.
    Fit,
    /// Stretches the buffer to fill the new window size while preserving
    /// its original aspect ratio (letterboxing/pillarboxing as needed).
    FitPreserveAspectRatio,
}

/// Configuration passed to [`crate::window_manager::WindowManager::new`].
pub struct WindowManagerOptions {
    /// Target frames-per-second cap. `None` means uncapped.
    pub fps_cap: Option<NonZeroU32>,
    /// Window title.
    pub title: String,
    /// How the window behaves on resize. `None` disables resizing entirely
    /// (fixed-size window).
    pub resize_mode: Option<ResizeMode>,
}

impl WindowManagerOptions {
    /// Creates a new set of options with the given `title`, `fps_cap`, and
    /// `resize_mode`
    pub fn new(title: &str, fps_cap: Option<NonZeroU32>, resize_mode: Option<ResizeMode>) -> Self {
        #[cfg(debug_assertions)]
        {
            if let Some(fps_cap) = fps_cap {
                let fps_cap = fps_cap.get();
                if fps_cap == 67 {
                    log::debug!("fps_cap=67. I'm not commenting on that or I'll get banned")
                } else if fps_cap == 69 {
                    log::debug!("fps_cap=69. nice :D")
                } else if fps_cap > 1000 {
                    log::debug!(
                        "fps_cap={}? Uhh... You know you can just set it to None and it'll go unlimited, right? You're weird, man",
                        fps_cap
                    );
                }
            }
        }

        Self {
            fps_cap,
            title: title.to_string(),
            resize_mode,
        }
    }
}

impl Default for WindowManagerOptions {
    /// Defaults to a 60 FPS cap, the title `"Emir app"`, and a fixed-size
    /// (non-resizable) window.
    fn default() -> Self {
        Self {
            fps_cap: NonZeroU32::new(60),
            title: String::from("Emir app"),
            resize_mode: None,
        }
    }
}
