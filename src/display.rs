use embedded_hal::digital::v2::{ToggleableOutputPin, OutputPin};
use crate::hub::HUBPort;
use crate::maze::{Point, Maze};
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;
use crate::ball::Ball;

pub(crate) const PWMFrequency: u8 = 8;

pub(crate) fn draw_row<CLK, OEN, LT, A, B, C, R1, G1, B1, R2, G2, B2>(
    port: &mut HUBPort<CLK, OEN, LT, A, B, C, R1, G1, B1, R2, G2, B2>,
    maze: &Maze,
    ball: &Ball,
    row: u8,
    pwm_counter: u8,
) where CLK: OutputPin,
        OEN: OutputPin,
        LT: OutputPin,
        A: OutputPin,
        B: OutputPin,
        C: OutputPin,
        R1: OutputPin,
        G1: OutputPin,
        B1: OutputPin,
        R2: OutputPin,
        B2: OutputPin,
        G2: OutputPin,
{
    let maze_row = row / 4;
    if row % 4 == 0 { // top walls
        for col in 0..32 {
            let mut data: u16 = 0;
            if maze.bitmap_top.get(Point{ x: col, y: maze_row }) {
                data |= 0b100000;
            }
            if maze.bitmap_top.get(Point{ x: col, y: maze_row + 8}) {
                data |= 0b000100;
            }


            // first dot
            {
                let mut altdata = data;
                if maze.bitmap_left.get(Point{ x: col, y: maze_row }) {
                    altdata |= 0b100000;
                }
                if maze.bitmap_left.get(Point{ x: col, y: maze_row + 8}) {
                    altdata |= 0b000100;
                }
                if col > 0 {
                    if maze.bitmap_top.get(Point{ x: col-1, y: maze_row }) {
                        altdata |= 0b100000;
                    }
                    if maze.bitmap_top.get(Point{ x: col-1, y: maze_row + 8}) {
                        altdata |= 0b000100;
                    }
                }
                if maze_row > 0 {
                    if maze.bitmap_left.get(Point{ x: col, y: maze_row - 1}) {
                        altdata |= 0b100000;
                    }
                    if maze.bitmap_left.get(Point{ x: col, y: maze_row + 8 - 1}) {
                        altdata |= 0b000100;
                    }
                }
                port.next_pixel(altdata);
            }

            for col in 1 .. 4 {
                port.next_pixel(data);
            }
        }
    } else { // side walls
        for col in 0 .. 32 {
            // Render the wall
            let mut data: u16 = 0;
            if maze.bitmap_left.get(Point{ x: col, y: maze_row }) {
                data |= 0b100000;
            }
            if maze.bitmap_left.get(Point{ x: col, y: maze_row + 8}) {
                data |= 0b000100;
            }
            port.next_pixel(data);

            data = 0;
            if maze.start.x == col && maze.start.y == maze_row {
                data |= 0b001000;
            } else if maze.start.x == col && maze.start.y == maze_row + 8 {
                data |= 0b000001;
            } else if maze.end.x == col && maze.end.y == maze_row {
                data |= 0b010000;
            } else if maze.end.x == col && maze.end.y == maze_row + 8 {
                data |= 0b000010;
            }
            for i in 1 .. 4 {
                port.next_pixel(data);
            }
        }
    }
    if row == 0 {
        port.next_page();
    } else {
        port.next_line();
    }
}
