# `ECE 362 Mini Project - Maze Game`

This is a project which uses the STM32F05R8T6 microcrontroller to build a randomly generated maze using DFS and recursive backtracking algorithms custom tailored to this project. We also used a custom designed PCB to implement this design to give it a professional look.
 
After deciding that we wanted to build a game, various ideas were hatched, and a maze was decided upon as it is a game that does not require quick reflexes, and isnt time based / constraint based. As mentioned before, the maze is generated using a DFS and recursive backtracking algorithms algorithm, and when a maze is completed, a sound plays and a new maze is generated, using the old maze's endpoint as the new mazes beginning.
 
For this project, we used the STM32F05R8T6 microcontroller chip, and initially we were considering using a gyroscope, but ended up deciding to forgo it as would take more time than we could manage. The game is now controlled using an analog joystick with a button. To display the maze, we are using a 64 x 128 LED matrix, and a passive buzzer to play a sound when the maze is completed. Since we decided to make a PCB, we designed it using Autodesk Eagle, and used several surface mount 0603 resistors and surface mount 0603 decoupling ceramic capacitors to prevent noise and ensure smooth operation.

The programs for this project was coded in Rust, which ensures thread safety and memory safety with its unique ownership system.

To accomodate the recursive backtracker algorithm to the limited 8K of RAM for the STM32 microcontroller, we modified the implementation of the recursive backtracker so that it runs iteratively, and takes up a fixed amount of RAM. It was also a learning experience for us to fabricate and solder the PCB components in the limited time given.

If any other teams would like to use the LED matrix we've selected, it's important to remind them that the LED matrix works very differently compared to the standard. For a standard, 64 row LED matrix, you would have five row selection pins (ABCDE) connected to a demux to control which row you're currently displaying. Then, you would shift data into a group of 74HC595 shift registers. However, the LED matrix we bought from Taobao uses some different chips for its controlling circuits. The shift registers on the horizontal axis used 8*3*2 ChipOne ICND2038S chips (http://www.xlix.ru/datasheet/icn2038s.pdf), which are 16-channel shift registers with dual latch, optimized for LED current sinking. However, on the vertical axis, instead of a demux, it used 8 D5958SSP shift registers (http://www.dmax168.com/h-por-11-0_329_11.html), which are just regular 8-bit shift registers optimized for LED matrix. Note that the datasheet for D5958SSP is only available in Chinese, and you might notice that the images of the datasheet on the website I have given above has a very low resolution. To view the high resolution version of the datasheet, right click on the images, select "Copy Image Location", and remove the "!160x160.png" from the URL.

Basically, to drive this LED matrix, you first shift in data for each row, and then for every 32 clock cycles, you pull the data input high on the vertical shift register. That way, you can scan the LED matrix in a way similiar to the original HUB75E products. On the modified HUB75E port, A => CLK, B => EN, C => DIN.

## Dependencies

To build embedded programs using this template you'll need:

- Rust 1.31, 1.30-beta, nightly-2018-09-13 or a newer toolchain. e.g. `rustup
  default beta`

- The `cargo generate` subcommand. [Installation
  instructions](https://github.com/ashleygwilliams/cargo-generate#installation).

- `rust-std` components (pre-compiled `core` crate) for the ARM Cortex-M
  targets. Run:

``` console
$ rustup target add thumbv6m-none-eabi thumbv7m-none-eabi thumbv7em-none-eabi thumbv7em-none-eabihf
```

## Using this program

0. Before we begin you need to identify some characteristics of the target
  device as these will be used to configure the project:

- The ARM core. e.g. Cortex-M3.

- Does the ARM core include an FPU? Cortex-M4**F** and Cortex-M7**F** cores do.

- How much Flash memory and RAM does the target device has? e.g. 256 KiB of
  Flash and 32 KiB of RAM.

- Where are Flash memory and RAM mapped in the address space? e.g. RAM is
  commonly located at address `0x2000_0000`.

You can find this information in the data sheet or the reference manual of your
device.

In this example we'll be using the STM32F3DISCOVERY. This board contains an
STM32F303VCT6 microcontroller. This microcontroller has:

- A Cortex-M4F core that includes a single precision FPU

- 256 KiB of Flash located at address 0x0800_0000.

- 40 KiB of RAM located at address 0x2000_0000. (There's another RAM region but
  for simplicity we'll ignore it).

1. Instantiate the template.

``` console
$ cargo generate --git https://github.com/rust-embedded/cortex-m-quickstart
 Project Name: app
 Creating project called `app`...
 Done! New project created /tmp/app

$ cd app
```

2. Set a default compilation target. There are four options as mentioned at the
   bottom of `.cargo/config`. For the STM32F303VCT6, which has a Cortex-M4F
   core, we'll pick the `thumbv7em-none-eabihf` target.

``` console
$ tail -n6 .cargo/config
```

``` toml
[build]
# Pick ONE of these compilation targets
# target = "thumbv6m-none-eabi"    # Cortex-M0 and Cortex-M0+
# target = "thumbv7m-none-eabi"    # Cortex-M3
# target = "thumbv7em-none-eabi"   # Cortex-M4 and Cortex-M7 (no FPU)
target = "thumbv7em-none-eabihf" # Cortex-M4F and Cortex-M7F (with FPU)
```

3. Enter the memory region information into the `memory.x` file.

``` console
$ cat memory.x
/* Linker script for the STM32F303VCT6 */
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K
  RAM : ORIGIN = 0x20000000, LENGTH = 40K
}
```

4. Build the template application or one of the examples.

``` console
$ cargo build
```

# License

This template is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Code of Conduct

Contribution to this crate is organized under the terms of the [Rust Code of
Conduct][CoC], the maintainer of this crate, the [Cortex-M team][team], promises
to intervene to uphold that code of conduct.

[CoC]: https://www.rust-lang.org/policies/code-of-conduct
[team]: https://github.com/rust-embedded/wg#the-cortex-m-team
