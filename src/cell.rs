use crate::display::PWMFrequency;
use core::fmt;

#[derive(Eq, PartialEq)]
pub struct Size {
    width: u16,
    height: u16,
}
impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} x {}", self.width, self.height)
    }
}

#[derive(Eq, PartialEq)]
pub struct Cell {
    pub(crate) position: crate::ball::Point,
    size: Size,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[({}, {}), ({}, {})]", self.position.x, self.position.y, self.position.x + self.size.width, self.position.y + self.size.height)
    }
}

impl Cell {
    pub fn contains(&self, point: &crate::ball::Point) -> bool {
        if point.x < self.position.x || point.y < self.position.y {
            return false;
        }
        if point.x >= (self.position.x + self.size.width ) || point.y >= (self.position.y + self.size.height)  {
            return  false;
        }
        return true;
    }

    pub fn of_point(point: &crate::maze::Point) -> Cell {
        Cell {
            position: crate::ball::Point::from_point(point),
            size: Size {
                height: (PWMFrequency * 3) as u16,
                width: (PWMFrequency * 3) as u16,
            }
        }
    }

    pub fn of_ball_point(point: &crate::ball::Point) -> Cell {
        let freq: u16 = (PWMFrequency as u16) * 4;
        Cell {
            position: crate::ball::Point{
                x: (point.x / freq) * freq + (PWMFrequency as u16), y: (point.y / freq) * freq + (PWMFrequency as u16)
            },
            size: Size {
                height: (PWMFrequency * 3) as u16,
                width: (PWMFrequency * 3) as u16,
            }
        }
    }

    pub fn bound_x(&self, point: &mut crate::ball::Point) {
        if point.x < self.position.x {
            point.x = self.position.x;
        } else if point.x > self.position.x + self.size.width-1 {
            point.x = self.position.x + self.size.width-1;
        }
    }

    pub fn bound_y(&self, point: &mut crate::ball::Point) {
        if point.y < self.position.y {
            point.y = self.position.y;
        } else if point.y > self.position.y + self.size.height - 1 {
            point.y = self.position.y + self.size.height - 1;
        }
    }
}