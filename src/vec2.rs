#[derive(Default, Debug, PartialEq, Eq)]
pub struct Vec2 {
    pub x: usize,
    pub y: usize,
}

impl Vec2 {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<(u16, u16)> for Vec2 {
    fn from(tuple: (u16, u16)) -> Self {
        Self::new(tuple.0 as usize, tuple.1 as usize)
    }
}
