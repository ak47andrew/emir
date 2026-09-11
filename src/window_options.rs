use std::num::NonZeroU32;

#[derive(Default)]
pub enum ResizeMode {
    #[default]
    Trim,
    Fit,
    FitPreserveAspectRatio,
}

pub struct WindowManagerOptions {
    pub fps_cap: Option<NonZeroU32>,
    pub title: String,
    pub resize_mode: Option<ResizeMode>,
}

impl WindowManagerOptions {
    pub fn new(title: &str, fps_cap: Option<NonZeroU32>, resize_mode: Option<ResizeMode>) -> Self {
        #[cfg(debug_assertions)]
        {
            if let Some(fps_cap) = fps_cap {
                let fps_cap = fps_cap.get();
                if fps_cap == 67 {
                    println!("fps_cap=67. I'm not commenting on that or I'll get banned")
                } else if fps_cap == 69 {
                    println!("fps_cap=69. nice :D")
                } else if fps_cap > 1000 {
                    println!(
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
    fn default() -> Self {
        Self {
            fps_cap: NonZeroU32::new(60),
            title: String::from("Emir app"),
            resize_mode: None,
        }
    }
}
