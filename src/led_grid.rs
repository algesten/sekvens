use crate::flip_pin::FlipPinExt;
use crate::{Col1, Col2, Col3, Col4, Col5, Col6, Col7, Col8};
use crate::{Row1Led, Row2Led, Row3Led, Row4Led, Row5Led};

pub struct LedGrid {
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BiLed {
    Off,
    Red,
    Grn,
}

impl LedGrid {
    pub fn set_row(&mut self, row: u8, on: BiLed, leds: &[BiLed; 8]) {
        let rows: &mut [&mut dyn FlipPinExt] = &mut [
            &mut self.row1,
            &mut self.row2,
            &mut self.row3,
            &mut self.row4,
            &mut self.row5,
        ];

        for (i, r) in rows.iter_mut().enumerate() {
            let i = i as u8;
            if row != i || on == BiLed::Off {
                r.disable();
            } else {
                if row == i {
                    let high = matches!(on, BiLed::Red);
                    r.set_output(high);
                }
            }
        }

        let cols: &mut [&mut dyn FlipPinExt] = &mut [
            &mut self.col1,
            &mut self.col2,
            &mut self.col3,
            &mut self.col4,
            &mut self.col5,
            &mut self.col6,
            &mut self.col7,
            &mut self.col8,
        ];

        for (c, led) in cols.iter_mut().zip(leds.iter()) {
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
