use crate::display::PWMFrequency;
use core::fmt;

#[derive(Eq, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

pub(crate) type Ball = Point;

impl Ball {
    pub(crate) fn new() -> Ball {
        Ball {
            x: 0,
            y: 0,
        }
    }
    pub(crate) fn from_point(p: &crate::maze::Point) -> Ball {
        Ball {
            x: (p.x as u16 * 4 + 2) * PWMFrequency as u16,
            y: (p.y as u16 * 4 + 2) * PWMFrequency as u16,
        }
    }
    pub(crate) fn to_point(&self, ) -> crate::maze::Point {
        crate::maze::Point {
            x: ((self.x / PWMFrequency as u16) / 4) as u8,
            y: ((self.y / PWMFrequency as u16) / 4) as u8,
        }
    }
}