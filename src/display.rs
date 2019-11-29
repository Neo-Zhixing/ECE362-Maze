use crate::{
    MAZE, HUB_PORT,
    hal::{
        stm32,
        stm32::{
            interrupt
        }
    }
};
use core::ops::DerefMut;
use embedded_hal::digital::v2::{ToggleableOutputPin, OutputPin};
use crate::hub::HUBPort;
use crate::maze::{Point, Maze};
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;


static CURRENT_ROW: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0));


#[interrupt]
fn TIM6_DAC() {
    cortex_m::interrupt::free(|cs| {
        if let (
            &mut Some(ref mut maze),
            &mut Some(ref mut port),
        ) = (
            MAZE.borrow(cs).borrow_mut().deref_mut(),
            HUB_PORT.borrow(cs).borrow_mut().deref_mut(),
        ) {
            let mut current_row = CURRENT_ROW.borrow(cs).borrow_mut();
            if *current_row == 32 {
                *current_row = 0;
            }
            draw_row(port, maze, *current_row);
            *current_row += 1;
        }
        unsafe {
            stm32::Peripherals::steal().TIM6.sr.write(|w| w.uif().clear_bit());
        }
    });
}

fn draw_row<CLK, OEN, LT, A, B, C, R1, G1, B1, R2, G2, B2>(
    port: &mut HUBPort<CLK, OEN, LT, A, B, C, R1, G1, B1, R2, G2, B2>,
    maze: &Maze,
    row: u8
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
    if row % 4 == 0 { // top walls
        for col in 0..32 {
            let mut data: u16 = 0;
            if maze.bitmap_top.get(Point{ x: col, y: row/4 }) {
                data |= 0b100000;
            }
            if maze.bitmap_top.get(Point{ x: col, y: (row/4) + 8}) {
                data |= 0b000100;
            }
            for col in 0 .. 4 {
                port.next_pixel(data);
            }
        }
    } else { // side walls
        for col in 0 .. 32 {
            let mut data: u16 = 0;
            if maze.bitmap_left.get(Point{ x: col, y: row/4 }) {
                data |= 0b100000;
            }
            if maze.bitmap_left.get(Point{ x: col, y: (row/4) + 8}) {
                data |= 0b000100;
            }
            port.next_pixel(data);

            for _ in 0 .. 3 {
                port.next_pixel(0);
            }
        }
    }
    if row == 0 {
        port.next_page();
    } else {
        port.next_line();
    }
}