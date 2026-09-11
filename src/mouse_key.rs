pub enum MouseKey {
    Left,
    Middle,
    Right,
}

impl From<minifb::MouseButton> for MouseKey {
    fn from(value: minifb::MouseButton) -> Self {
        match value {
            minifb::MouseButton::Left => MouseKey::Left,
            minifb::MouseButton::Middle => MouseKey::Middle,
            minifb::MouseButton::Right => MouseKey::Right,
        }
    }
}

impl From<MouseKey> for minifb::MouseButton {
    fn from(value: MouseKey) -> Self {
        match value {
            MouseKey::Left => minifb::MouseButton::Left,
            MouseKey::Middle => minifb::MouseButton::Middle,
            MouseKey::Right => minifb::MouseButton::Right,
        }
    }
}
