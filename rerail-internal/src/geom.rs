#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

pub struct Rect {
    top: i32,
    bottom: i32,
    left: i32,
    right: i32,
}

impl Rect {
    pub fn new(top: i32, bottom: i32, left: i32, right: i32) -> Rect {
        Rect { top, bottom, left, right }
    }

    pub fn from_corners(a: Coord, b: Coord) -> Rect {
        Rect {
            top: a.y.min(b.y),
            bottom: a.y.max(b.y),
            left: a.x.min(b.x),
            right: a.x.max(b.x),
        }
    }

    pub fn contains(&self, pt: Coord) -> bool {
        self.top < pt.y && pt.y < self.bottom && self.left < pt.x && pt.x < self.right
    }
}
