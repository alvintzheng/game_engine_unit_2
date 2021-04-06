#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Savefile)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub fn translate(&mut self, x: i32, y: i32) {
        self.x += x;
        self.y += y;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Savefile)]
pub struct Vec2i(pub i32,pub i32);

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Savefile)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);
