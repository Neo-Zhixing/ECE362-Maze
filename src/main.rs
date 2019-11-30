#![no_main]
#![no_std]

mod maze;
mod hub;
mod display;

use panic_halt as _;

use stm32f0xx_hal as hal;
use crate::hal::{prelude::*, stm32, serial, delay::Delay, i2c, gpio::*, stm32::{interrupt}};


use embedded_hal::digital::v2::OutputPin;
use core::fmt::Write;
use ssd1306::prelude::*;
use ssd1306::Builder;
use crate::hal::dac::*;
use crate::maze::Point;
use core::cell::RefCell;
use core::ops::DerefMut;
use stm32f0::stm32f0x1::Interrupt;
use rtfm::Mutex;

#[rtfm::app(device = stm32f0xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // A resource
        maze: maze::Maze,

        #[init(0)]
        current_row: u8,

        hub_port: hub::HUBPort<
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
            gpioc::PC5<Output<PushPull>>
        >,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        // Cortex-M _device
        let mut _core: cortex_m::Peripherals = ctx.core;

        // Device specific _device
        let mut _device: stm32::Peripherals = ctx.device;

        let mut rcc = _device.RCC.configure().sysclk(48.mhz()).freeze(&mut _device.FLASH);

        let gpioa = _device.GPIOA.split(&mut rcc);
        let gpiob = _device.GPIOB.split(&mut rcc);
        let gpioc = _device.GPIOC.split(&mut rcc);
        let mut delay = Delay::new(_core.SYST, &rcc);
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

        let mut dac = hal::dac::dac(_device.DAC, buz, &mut rcc);
        dac.enable();
        dac.set_value(4095);

        led_blue.set_high().ok();

        led_blue.set_low().ok();
        // keep row selection port enabled
        port.row_selection.b.set_low().ok();

        // Setting up timer for display refresh
        let mut timer = hal::timers::Timer::tim6(_device.TIM6, hal::time::Hertz(120 * 32), &mut rcc);
        timer.listen(hal::timers::Event::TimeOut);



        let mut nvic = _core.NVIC;
        unsafe {
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM6_DAC);
        }
        init::LateResources {
            hub_port: port,
            maze: maze::Maze::new(),
        }
    }

    #[task(binds = TIM6_DAC, resources=[current_row, &maze, hub_port], priority=10)]
    fn tim6 (ctx: tim6::Context) {
        let current_row: &mut u8 = ctx.resources.current_row;
        let maze: &maze::Maze = ctx.resources.maze;
        let port = ctx.resources.hub_port;
        if *current_row == 32 {
            *current_row = 0;
        }
        display::draw_row(port, maze, *current_row);
        *current_row += 1;
        unsafe {
            stm32::Peripherals::steal().TIM6.sr.write(|w| w.uif().clear_bit());
        }
    }

    #[idle(resources = [&maze])]
    fn idle (ctx: idle::Context) -> ! {
        let mut maze_generator = maze::MazeGenerator::new();

        // unsafe is ok here, idle is the only task requiring mutable access to maze.
        // Needed because 0.5.1 version of cortex-m-rtfm does not support mixed resources access
        unsafe {
            let ptr = ctx.resources.maze as *const maze::Maze as *mut maze::Maze;
            let maze = &mut *ptr;
            maze_generator.generate(maze);
        }



        loop {
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn USART1();
    }
};

