use std::num::NonZeroU32;

pub struct WindowManagerOptions {
    pub fps_cap: Option<NonZeroU32>,
    pub title: String,
}

impl WindowManagerOptions {
    pub fn new(title: &str, fps_cap: Option<NonZeroU32>) -> Self {
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
        }
    }
}

impl Default for WindowManagerOptions {
    fn default() -> Self {
        WindowManagerOptions {
            fps_cap: NonZeroU32::new(60),
            title: String::from("Emir app"),
        }
    }
}
