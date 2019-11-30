#![no_main]
#![no_std]

mod maze;
mod hub;
mod display;
mod ball;
mod joystick;

use panic_halt as _;

use stm32f0xx_hal as hal;
use crate::hal::{prelude::*, stm32, serial, delay::Delay, i2c::{I2c}, gpio::*};


use embedded_hal::digital::v2::OutputPin;
use core::fmt::Write;
use crate::hal::dac::*;
use crate::maze::Point;
use core::cell::RefCell;
use core::ops::DerefMut;
use stm32f0::stm32f0x1::Interrupt;
use rtfm::Mutex;
use cortex_m::asm::wfi;


use ssd1306::prelude::*;
use ssd1306::Builder;
use ssd1306::mode::GraphicsMode;

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Rectangle, Triangle};
use joystick::Joystick;

#[rtfm::app(device = stm32f0xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // A resource
        maze: maze::Maze,
        ball: ball::Ball,

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

        led: gpioc::PC8<Output<PushPull>>,
        delay: Delay,
        display: GraphicsMode<
            ssd1306::interface::i2c::I2cInterface<
                stm32f0xx_hal::i2c::I2c<
                    stm32f0::stm32f0x1::I2C1,
                    gpiob::PB6<Alternate<hal::gpio::AF1>>,
                    gpiob::PB7<Alternate<hal::gpio::AF1>>,
                >
            >
        >,

        joystick: Joystick<
            gpioa::PA1<Analog>,
            gpioa::PA2<Analog>,
            gpioa::PA3<Input<Floating>>,
        >,
        adc: hal::adc::Adc,
        dac: hal::dac::C1,
        serial: serial::Serial<
            stm32::USART1,
            gpioa::PA9<Alternate<hal::gpio::AF1>>,
            gpioa::PA10<Alternate<hal::gpio::AF1>>,
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
        let (
            mut port,
            mut led_green, mut led_blue,
            mut btn,
            mut buz,
            mut scl, mut sda,
            mut joystick_x, mut joystick_y, mut joystick_btn,
            tx, rx,
        ) = cortex_m::interrupt::free(|cs| {
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
             gpiob.pb6.into_alternate_af1(cs),
             gpiob.pb7.into_alternate_af1(cs),
             gpioa.pa1.into_analog(cs),
             gpioa.pa2.into_analog(cs),
             gpioa.pa3.into_floating_input(cs),
             gpioa.pa9.into_alternate_af1(cs),
             gpioa.pa10.into_alternate_af1(cs),
            )
        });

        let mut i2c = I2c::i2c1(_device.I2C1, (scl, sda), 100.khz(), &mut rcc);
        let mut serial = serial::Serial::usart1(_device.USART1, (tx, rx), 115_200.bps(), &mut rcc);
        let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
        //display.init();
        //display.clear();


        let mut adc = hal::adc::Adc::new(_device.ADC, &mut rcc);


        let mut dac = hal::dac::dac(_device.DAC, buz, &mut rcc);
        dac.enable();
        dac.set_value(4095);

        led_blue.set_high().ok();

        led_blue.set_low().ok();
        // keep row selection port enabled
        port.row_selection.b.set_low().ok();

        // Setting up timer for display refresh
        // Why 60 * 32? 120 Hertz is the refresh rate for high end screens, and we have 32 rows
        // This value needs to be as low as possible, otherwise CPU utilization rate would be too high
        // For us to do anything else useful.
        let mut timer = hal::timers::Timer::tim6(_device.TIM6, hal::time::Hertz(120 * 32), &mut rcc);
        timer.listen(hal::timers::Event::TimeOut);

        let mut timer_input = hal::timers::Timer::tim14(_device.TIM14, hal::time::Hertz(100), &mut rcc);
        timer_input.listen(hal::timers::Event::TimeOut);

        let mut nvic = _core.NVIC;
        unsafe {
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM6_DAC);
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM14);
        }
        let joystick_mid_x: u16 = adc.read(&mut joystick_x).unwrap();
        let joystick_mid_y: u16 = adc.read(&mut joystick_y).unwrap();
        init::LateResources {
            hub_port: port,
            maze: maze::Maze::new(),
            ball: ball::Ball::new(),
            led: led_blue,
            delay,
            display,
            dac,
            adc,
            joystick: Joystick::new(
                joystick_x,
                joystick_y,
                joystick_btn,
                joystick_mid_x,
                joystick_mid_y,
            ),
            serial,
        }
    }

    #[task(binds = TIM6_DAC, resources=[current_row, &maze, &ball, hub_port], priority=10)]
    fn tick (ctx: tick::Context) {
        let current_row: &mut u8 = ctx.resources.current_row;
        let maze: &maze::Maze = ctx.resources.maze;
        let port = ctx.resources.hub_port;
        if *current_row == 32 {
            *current_row = 0;
        }
        display::draw_row(port, maze, ctx.resources.ball, *current_row);
        *current_row += 1;

        unsafe {
            stm32::Peripherals::steal().TIM6.sr.write(|w| w.uif().clear_bit());
        }
    }

    #[task(binds = TIM14, resources=[&ball, hub_port, adc, joystick], priority=5)]
    fn input (ctx: input::Context) {
        let valx: u16 = ctx.resources.adc.read(&mut ctx.resources.joystick.axis_x).unwrap();
        let valy: u16 = ctx.resources.adc.read(&mut ctx.resources.joystick.axis_y).unwrap();

        let mut valx: i16 = ctx.resources.joystick.mid_x as i16 - valx as i16;
        let mut valy: i16 = valy as i16 - ctx.resources.joystick.mid_y as i16;

        valx /= 256;
        valy /= 256;

        unsafe {
            let ptr = ctx.resources.ball as *const ball::Ball as *mut ball::Ball;
            let ball = &mut *ptr;
            let mut newx: i16 =  (ball.x as i16 + valx);
            let mut newy: i16 = (ball.y as i16 + valy);
            if newx < 0 {
                newx = 0;
            } else if newx > 32 * 4 * display::PWMFrequency as i16 - 1 {
                newx = 32 * 4 * display::PWMFrequency as i16 - 1;
            }
            if newy < 0 {
                newy = 0;
            } else if newy > 16 * 4 * display::PWMFrequency as i16 - 1 {
                newy = 16 * 4 * display::PWMFrequency as i16 - 1;
            }
            ball.x = newx as u16;
            ball.y = newy as u16;
        }
        unsafe {
            stm32::Peripherals::steal().TIM14.sr.write(|w| w.uif().clear_bit());
        }
    }

    #[idle(resources = [&maze, &ball])]
    fn idle (ctx: idle::Context) -> ! {
        let mut maze_generator = maze::MazeGenerator::new();

        // unsafe is ok here, idle is the only task requiring mutable access to maze.
        // Needed because 0.5.1 version of cortex-m-rtfm does not support mixed resources access
        unsafe {
            let ptr = ctx.resources.maze as *const maze::Maze as *mut maze::Maze;
            let maze = &mut *ptr;

            maze_generator.generate(maze);

            let ball_ptr = ctx.resources.ball as *const ball::Ball as *mut ball::Ball;
            let ball = &mut *ball_ptr;

            *ball = ball::Ball::from_point(&(maze.start));
        }

        loop {
            wfi();
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn USART2();
    }
};

