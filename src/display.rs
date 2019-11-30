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
    let mut buf = [0_u8; 128];
    let mut buf_iter = buf.iter_mut();
    if row % 4 == 0 { // top walls
        for col in 0..32 {
            let mut data: u8 = 0;
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
                *buf_iter.next().unwrap() = altdata;
            }

            for col in 1 .. 4 {
                *buf_iter.next().unwrap() = data;
            }
        }
    } else { // side walls
        for col in 0 .. 32 {
            // Render the wall
            let mut data: u8 = 0;
            if maze.bitmap_left.get(Point{ x: col, y: maze_row }) {
                data |= 0b100000;
            }
            if maze.bitmap_left.get(Point{ x: col, y: maze_row + 8}) {
                data |= 0b000100;
            }
            *buf_iter.next().unwrap() = data;

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
                *buf_iter.next().unwrap() = data;
            }
        }
    }



    let ball_screen_x = (ball.x / PWMFrequency as u16) as u8;
    let ball_screen_y = (ball.y / PWMFrequency as u16) as u8;
    for offset in 0 .. 2 {
        let current_screen_row = row + (32 & (offset << 5));
        if ball_screen_y == current_screen_row {
            let shift = offset | (offset << 1); // 3 if offset is 1, or 0 if offset is 0
            buf[ball_screen_x as usize] |= 0b111000 >> shift;
        }
    }


    for i in buf.iter().cloned() {
        port.next_pixel(i);
    }
    if row == 0 {
        port.next_page();
    } else {
        port.next_line();
    }
}


pub(crate) fn draw_ball<CLK, OEN, LT, A, B, C, R1, G1, B1, R2, G2, B2>(
    port: &mut HUBPort<CLK, OEN, LT, A, B, C, R1, G1, B1, R2, G2, B2>,
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

}