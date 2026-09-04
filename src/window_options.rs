pub struct WindowManagerOptions {
    pub fps_cap: Option<u32>,
    pub title: String
}

impl WindowManagerOptions {
    pub fn new(title: &str, fps: Option<u32>) -> Self {
        #[cfg(debug_assertions)]
        {
            if let Some(fps_cap) = fps {
                if fps_cap == 0 {
                    println!("fps_cap=0. Okay, Mr. Smartpants. You studied minifb really good, didn't ya? So just stfu, use fps_cap=None and don't be that guy")
                }
                else if fps_cap == 67 {
                    println!("fps_cap=67. I'm not commenting on that or I'll get banned")
                }
                else if fps_cap == 69 {
                    println!("fps_cap=69. nice :D")
                }
                else if fps_cap >= 1000 {
                    println!("fps_cap={}? Uhh... You know you can just set it to None and it'll go unlimited, right? You're weird, man", fps_cap);
                }
            }
        }

        Self { fps_cap: fps, title: title.to_string()}
    }
}

impl Default for WindowManagerOptions {
    fn default() -> Self {
        WindowManagerOptions {
            fps_cap: Some(60),
            title: String::from("Emir app")
        }
    }
}
