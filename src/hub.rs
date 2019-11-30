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
    pub(crate) fn next_line(&mut self) {
        self.row_selection.c.set_low().ok();
        self.output_enabled.set_high().ok();
        self.latch.set_high().ok();
        self.latch.set_low().ok();

        self.row_selection.a.set_high().ok();
        self.row_selection.a.set_low().ok();
        self.output_enabled.set_low().ok();
    }
     pub(crate) fn next_page(&mut self) {
         self.row_selection.c.set_high().ok();
         self.output_enabled.set_high().ok();
         self.latch.set_high().ok();
         self.latch.set_low().ok();

         self.row_selection.a.set_high().ok();
         self.row_selection.a.set_low().ok();
         self.output_enabled.set_low().ok();
     }
     pub(crate) fn flush(&mut self) {
         self.output_enabled.set_high().ok();
         self.latch.set_high().ok();
         self.latch.set_low().ok();
         self.output_enabled.set_low().ok();
     }
     pub(crate) fn next_pixel(&mut self, pixel: u8) {
         self.clock.set_high().ok();

         if ((pixel >> 5) & 0b001) == 1 {
             self.data_upper.r.set_high().ok();
         } else {
             self.data_upper.r.set_low().ok();
         }
         if ((pixel >> 4) & 0b001) == 1 {
             self.data_upper.g.set_high().ok();
         } else {
             self.data_upper.g.set_low().ok();
         }
         if ((pixel >> 3) & 0b001) == 1 {
             self.data_upper.b.set_high().ok();
         } else {
             self.data_upper.b.set_low().ok();
         }
         if ((pixel >> 2) & 0b001) == 1 {
             self.data_lower.r.set_high().ok();
         } else {
             self.data_lower.r.set_low().ok();
         }
         if ((pixel >> 1) & 0b001) == 1 {
             self.data_lower.g.set_high().ok();
         } else {
             self.data_lower.g.set_low().ok();
         }
         if ((pixel >> 0) & 0b001) == 1 {
             self.data_lower.b.set_high().ok();
         } else {
             self.data_lower.b.set_low().ok();
         }
         self.clock.set_low().ok();
     }
}


