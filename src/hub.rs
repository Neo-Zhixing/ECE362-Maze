use embedded_hal::digital::v2::OutputPin;

pub struct HUBRowSelectionPort<A, B, C>
where A: OutputPin,
      B: OutputPin,
      C: OutputPin, {
    pub a: A,
    pub b: B,
    pub c: C,
}

pub struct HUBDataPort<R, G, B>
where R: OutputPin,
      G: OutputPin,
      B: OutputPin, {
    pub r: R,
    pub g: G,
    pub b: B,
}
pub struct HUBPort<CLK, OEN, LT, A, B, C, R1, G1, B1, R2, G2, B2>
where CLK: OutputPin,
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
    pub clock: CLK,
    pub output_enabled: OEN,
    pub latch: LT,
    pub row_selection: HUBRowSelectionPort<A, B, C>,
    pub data_upper: HUBDataPort<R1, G1, B1>,
    pub data_lower: HUBDataPort<R2, G2, B2>,
}

impl<CLK, OEN, LT, A, B, C, R1, G1, B1, R2, G2, B2> HUBPort<CLK, OEN, LT, A, B, C, R1, G1, B1, R2, G2, B2>
    where CLK: OutputPin,
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
    pub(crate) fn select_row(&mut self, row: u8) {
        if row & 0b00001 != 0 {
            self.row_selection.a.set_high().ok();
        } else {
            self.row_selection.a.set_low().ok();
        }

        if row & 0b00010 != 0 {
            self.row_selection.b.set_high().ok();
        } else {
            self.row_selection.b.set_low().ok();
        }

        if row & 0b00100 != 0 {
            self.row_selection.c.set_high().ok();
        } else {
            self.row_selection.c.set_low().ok();
        }
    }
}

