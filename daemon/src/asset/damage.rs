pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub enum Damage {
    Full,
    None,
    Rect(Rect),
}
