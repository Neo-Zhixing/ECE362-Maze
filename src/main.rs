#![no_main]
#![no_std]

mod maze;
mod hub;


use panic_halt as _;

use stm32f0xx_hal as hal;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Rectangle, Triangle};
use crate::hal::{prelude::*, stm32, serial, delay::Delay, i2c, prelude::*,};

//use crate::maze::{Maze, MazeGenerator};
use cortex_m_rt::entry;

//use crate::hub::{HUBPort, HUBDataPort, HUBRowSelectionPort};
use embedded_hal::digital::v2::OutputPin;
use core::fmt::Write;
use ssd1306::prelude::*;
use ssd1306::Builder;
use crate::hal::dac::*;


#[entry]
fn main() -> ! {
    let mut peripherals = stm32::Peripherals::take().unwrap();
    let kernel_peripherals = cortex_m::Peripherals::take().unwrap();
    let mut rcc = peripherals.RCC.configure().sysclk(48.mhz()).freeze(&mut peripherals.FLASH);

    let gpioa = peripherals.GPIOA.split(&mut rcc);
    let gpiob = peripherals.GPIOB.split(&mut rcc);
    let gpioc = peripherals.GPIOC.split(&mut rcc);
    let mut delay = Delay::new(kernel_peripherals.SYST, &rcc);
    let (mut port, mut led_green, mut led_blue, mut btn, mut buz) = cortex_m::interrupt::free(|cs| {
        // Configure pins for SPI
        (hub::HUBPort {
            clock: gpiob.pb1.into_push_pull_output_hs(cs),
            output_enabled: gpiob.pb0.into_push_pull_output_hs(cs),
            latch: gpiob.pb2.into_push_pull_output_hs(cs),
            data_upper: hub::HUBDataPort {
                r: gpioc.pc0.into_push_pull_output_hs(cs),
                g: gpioc.pc1.into_push_pull_output_hs(cs),
                b: gpioc.pc2.into_push_pull_output_hs(cs),
            },
            data_lower: hub::HUBDataPort {
                r: gpioc.pc3.into_push_pull_output_hs(cs),
                g: gpioc.pc4.into_push_pull_output_hs(cs),
                b: gpioc.pc5.into_push_pull_output_hs(cs),
            },
            row_selection: hub::HUBRowSelectionPort {
                a: gpiob.pb3.into_push_pull_output_hs(cs), // LCK
                b: gpiob.pb4.into_push_pull_output_hs(cs), // BK
                c: gpiob.pb5.into_push_pull_output_hs(cs), // DIN
            }
        },
         gpioc.pc9.into_push_pull_output(cs),
         gpioc.pc8.into_push_pull_output(cs),
         gpioa.pa0.into_floating_input(cs),
         gpioa.pa4.into_analog(cs),
        )
    });

    let mut dac = hal::dac::dac(peripherals.DAC, buz, &mut rcc);
    dac.enable();
    dac.set_value(4095);

    led_blue.set_high().ok();
    let mut maze_generator = maze::MazeGenerator::new();
    let maze: maze::Maze = maze_generator.dummy_generate();
    led_blue.set_low().ok();
    // keep row selection port enabled
    port.row_selection.b.set_low().ok();
    loop {
        led_green.toggle().ok();
        for row in 0..32 {
            led_blue.toggle().ok();

            if row % 4 == 0 { // top walls
                for col in 0 .. 128 {
                    port.clock.set_high().ok();
                    port.data_upper.r.set_high().ok();
                    port.data_lower.r.set_high().ok();
                    port.data_upper.g.set_low().ok();
                    port.data_lower.g.set_low().ok();
                    port.clock.set_low().ok();
                }
                /*
                for value in maze.bitmap_top.row_iter(row) {
                    for i in 0 .. 4 {
                        port.clock.set_high().ok();
                        port.data_upper.r.set_low().ok();
                        port.data_lower.r.set_high().ok();
                        port.data_upper.g.set_low().ok();
                        port.data_lower.g.set_low().ok();
                        port.clock.set_low().ok();
                    }
                }
                */
            } else { // side walls
                for col in 0 .. 32 {
                    port.clock.set_high().ok();
                    port.data_upper.r.set_high().ok();
                    port.data_lower.r.set_high().ok();
                    port.data_upper.g.set_low().ok();
                    port.data_lower.g.set_low().ok();
                    port.clock.set_low().ok();

                    for _ in 0 .. 3 {
                        port.clock.set_high().ok();
                        port.data_upper.r.set_low().ok();
                        port.data_lower.r.set_low().ok();
                        port.data_upper.g.set_low().ok();
                        port.data_lower.g.set_low().ok();
                        port.clock.set_low().ok();
                    }
                }
                /*
                for value in maze.bitmap_left.row_iter(row) {
                    port.clock.set_high().ok();
                    port.data_upper.r.set_low().ok();
                    port.data_lower.r.set_high().ok();
                    port.data_upper.g.set_low().ok();
                    port.data_lower.g.set_low().ok();
                    port.clock.set_low().ok();


                    // clear colors
                    port.data_upper.r.set_low().ok();
                    port.data_lower.r.set_low().ok();
                    port.data_upper.g.set_low().ok();
                    port.data_lower.g.set_low().ok();

                    for _ in 0 .. 3 { // three clock cycles empty
                        port.clock.set_high().ok();
                        port.clock.set_low().ok();
                    }
                }
                */
            }






            if (row == 0) {
                port.row_selection.c.set_high().ok();
            } else {
                port.row_selection.c.set_low().ok();
            }
            port.output_enabled.set_high().ok();

            port.latch.set_high().ok();
            port.row_selection.a.set_high().ok();
            port.latch.set_low().ok();
            port.row_selection.a.set_low().ok();

            port.output_enabled.set_low().ok();
        }
    }
}



