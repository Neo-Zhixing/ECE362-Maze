use embedded_hal::adc::{ Channel, OneShot };
use embedded_hal::digital::v2::InputPin;

pub struct Joystick<X, Y, BTN>
where BTN: InputPin,
{
    pub axis_x: X,
    pub axis_y: Y,
    pub button: BTN,

    pub mid_x: u16,
    pub mid_y: u16
}

impl<X, Y, BTN> Joystick<X, Y, BTN>
    where BTN: InputPin {
    pub fn new(x: X, y: Y, btn: BTN, mid_x: u16, mid_y: u16) -> Joystick<X, Y, BTN> {
        Joystick {
            axis_x: x,
            axis_y: y,
            button: btn,
            mid_x, mid_y
        }
    }
}