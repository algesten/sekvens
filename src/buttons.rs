use alg::bitfield::Bitfield;

#[derive(Default)]
pub struct Buttons {
    // 0-3  : Part (row 1)
    // 4-7  : Track (row 2)
    // 8-11 : Shift, Copy, Clear, Vel (row 3)
    top: Bitfield,

    // 0-15  : Rotary button
    // 16-31 : Step button
    step: Bitfield,
}

impl Buttons {
    pub fn set_top(&mut self, row: usize, col: usize, on: bool) {
        // The buttons are at row 3,4,5 column 5,6,7,8
        let bit = (row - 2) * 4 + col - 4;
        self.top.set(bit as u8, on);
    }

    pub fn set_rotary(&mut self, row: usize, col: usize, on: bool) {
        let bit = row * 8 + col;
        self.step.set(bit as u8, on);
    }

    pub fn set_step(&mut self, row: usize, col: usize, on: bool) {
        let bit = 16 + row * 8 + col;
        self.step.set(bit as u8, on);
    }

    pub fn is_clear(&self) -> bool {
        *self.top == 0 && *self.step == 0
    }

    pub fn is_shift(&self) -> bool {
        self.top.is(8)
    }

    pub fn is_vel(&self) -> bool {
        self.top.is(11)
    }

    pub fn is_rotary_top(&self) -> Option<u32> {
        // We want exactly one rotary button pressed.
        let x = *self.step & 0x00ff;
        if x.count_ones() == 1 {
            Some(x)
        } else {
            None
        }
    }

    pub fn is_rotary_bottom(&self) -> Option<u32> {
        // We want exactly one rotary button pressed.
        let x = *self.step & 0xff00;
        if x.count_ones() == 1 {
            Some(x)
        } else {
            None
        }
    }
}
