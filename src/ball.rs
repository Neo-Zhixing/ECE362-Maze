use crate::maze::Point;

pub struct Ball {
    pub x: u16,
    pub y: u16,
}

impl Ball {
    pub(crate) fn new() -> Ball {
        Ball {
            x: 0,
            y: 0,
        }
    }
    pub(crate) fn from_point(p: &Point) -> Ball {
        Ball {
            x: (p.x as u16) * 128 * 4,
            y: (p.y as u16) * 128 * 4,
        }
    }
}