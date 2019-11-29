#![no_main]
#![no_std]

mod maze;
mod hub;


use panic_halt as _;

use stm32f0xx_hal as hal;
use crate::hal::{prelude::*, stm32, serial, delay::Delay, i2c, gpio::*, stm32::{interrupt}};

//use crate::maze::{Maze, MazeGenerator};
use cortex_m_rt::entry;

//use crate::hub::{HUBPort, HUBDataPort, HUBRowSelectionPort};
use embedded_hal::digital::v2::OutputPin;
use core::fmt::Write;
use ssd1306::prelude::*;
use ssd1306::Builder;
use crate::hal::dac::*;
use crate::maze::Point;
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;
use core::ops::DerefMut;
use stm32f0::stm32f0x1::Interrupt;

// Mutex is a data structure when you're trying to access it, it disable interrupts for safety
static MAZE: Mutex<RefCell<Option<maze::Maze>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<gpioc::PC8<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static HUB_PORT: Mutex<RefCell<Option<hub::HUBPort<
    gpiob::PB1<Output<PushPull>>,
    gpiob::PB0<Output<PushPull>>,
    gpiob::PB2<Output<PushPull>>,
    gpiob::PB3<Output<PushPull>>,
    gpiob::PB4<Output<PushPull>>,
    gpiob::PB5<Output<PushPull>>,
    gpioc::PC0<Output<PushPull>>,
    gpioc::PC1<Output<PushPull>>,
    gpioc::PC2<Output<PushPull>>,
    gpioc::PC3<Output<PushPull>>,
    gpioc::PC4<Output<PushPull>>,
    gpioc::PC5<Output<PushPull>>>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn TIM6_DAC() {
    cortex_m::interrupt::free(|cs| {
        if let (
            &mut Some(ref mut maze),
            &mut Some(ref mut port),
            &mut Some(ref mut led),
        ) = (
            MAZE.borrow(cs).borrow_mut().deref_mut(),
            HUB_PORT.borrow(cs).borrow_mut().deref_mut(),
            LED.borrow(cs).borrow_mut().deref_mut(),
        ) {
            led.toggle().ok();
            for row in 0..32 {
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
        }
        unsafe {
            stm32::Peripherals::steal().TIM6.sr.write(|w| w.uif().clear_bit());
        }
    });
}

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
    let maze = maze_generator.dummy_generate();

    led_blue.set_low().ok();
    // keep row selection port enabled
    port.row_selection.b.set_low().ok();

    cortex_m::interrupt::free(|cs| {
        *MAZE.borrow(cs).borrow_mut() = Some(maze);
        *HUB_PORT.borrow(cs).borrow_mut() = Some(port);
        *LED.borrow(cs).borrow_mut() = Some(led_blue);
    });

    let mut timer = hal::timers::Timer::tim6(peripherals.TIM6, hal::time::Hertz(100), &mut rcc);
    timer.listen(hal::timers::Event::TimeOut);
    let mut nvic = kernel_peripherals.NVIC;
    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM6_DAC);
    }
    loop {
    }
}



