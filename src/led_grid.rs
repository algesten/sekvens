use crate::flip_pin::FlipPinExt;
use crate::{Col1, Col2, Col3, Col4, Col5, Col6, Col7, Col8};
use crate::{Row1Led, Row2Led, Row3Led, Row4Led, Row5Led};

pub struct LedGridPins {
    pub col1: Col1,
    pub col2: Col2,
    pub col3: Col3,
    pub col4: Col4,
    pub col5: Col5,
    pub col6: Col6,
    pub col7: Col7,
    pub col8: Col8,

    pub row1: Row1Led,
    pub row2: Row2Led,
    pub row3: Row3Led,
    pub row4: Row4Led,
    pub row5: Row5Led,
}

pub struct LedGrid {
    pins: LedGridPins,
    col_mode: bool,
}

impl LedGrid {
    pub fn new(pins: LedGridPins) -> Self {
        LedGrid {
            pins,
            col_mode: false,
        }
    }

    fn rows(&mut self) -> [&mut dyn FlipPinExt; 5] {
        [
            &mut self.pins.row1,
            &mut self.pins.row2,
            &mut self.pins.row3,
            &mut self.pins.row4,
            &mut self.pins.row5,
        ]
    }

    fn cols(&mut self) -> [&mut dyn FlipPinExt; 8] {
        [
            &mut self.pins.col1,
            &mut self.pins.col2,
            &mut self.pins.col3,
            &mut self.pins.col4,
            &mut self.pins.col5,
            &mut self.pins.col6,
            &mut self.pins.col7,
            &mut self.pins.col8,
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BiLed {
    Off,
    Red,
    Grn,
}

impl Default for BiLed {
    fn default() -> Self {
        BiLed::Off
    }
}

impl LedGrid {
    pub fn set_col(&mut self, col: usize) {
        // disable row pins in col mode
        if !self.col_mode {
            for r in self.rows() {
                r.disable();
            }

            self.col_mode = true;
        }

        for (i, c) in self.cols().iter_mut().enumerate() {
            if i == col {
                c.set_output(true);
            } else {
                c.disable();
            }
        }
    }

    pub fn set_leds(&mut self, row: usize, on: BiLed, leds: &[BiLed; 8]) {
        if self.col_mode {
            self.col_mode = false;
        }

        for (i, r) in self.rows().iter_mut().enumerate() {
            if row != i || on == BiLed::Off {
                r.disable();
            } else {
                if row == i {
                    let high = matches!(on, BiLed::Red);
                    r.set_output(high);
                }
            }
        }

        for (c, led) in self.cols().iter_mut().zip(leds.iter()) {
            if on == BiLed::Off || *led != on {
                c.disable();
            } else {
                // led == on
                let high = matches!(*led, BiLed::Grn);
                c.set_output(high);
            }
        }
    }
}
