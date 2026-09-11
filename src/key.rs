macro_rules! shitty_enum {
    ($vis:vis $name:ident { $($element_name:ident),* $(,)? }) => {
        $vis enum $name {
            $($element_name),*
        }

        impl From<minifb::Key> for Key {
            fn from(value: minifb::Key) -> Self {
                match value {
                    $(
                        minifb::Key::$element_name => Key::$element_name,
                    )*
                    _ => Key::Unknown,
                }
            }
        }

        impl From<Key> for minifb::Key {
            fn from(value: Key) -> Self {
                match value {
                    $(
                        Key::$element_name => minifb::Key::$element_name,
                    )*
                }
            }
        }
    };
}

shitty_enum!(pub Key {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Down, Left, Right, Up,
    Apostrophe, Backquote, Backslash, Comma, Equal, LeftBracket, Minus, Period,
    RightBracket, Semicolon, Slash, Backspace, Delete, End, Enter, Escape, Home,
    Insert, Menu, PageDown, PageUp, Pause, Space, Tab, NumLock, CapsLock, ScrollLock,
    LeftShift, RightShift, LeftCtrl, RightCtrl,
    Unknown,
});
