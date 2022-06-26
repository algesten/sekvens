use core::fmt::Debug;

use crate::hal::prelude::InputPin;
use crate::state::Oper;
use alg::clock::Time;
use alg::encoder::{Encoder, QuadratureSource};
use alg::input::{DebounceDigitalInput, DeltaInput, DigitalEdgeInput};
use alg::input::{DigitalInput, Edge, EdgeInput, HiLo};

use crate::{
    Col, InClock, InReset, OperQueue, Row, Row1RotA, Row1RotB, Row1Swl, Row1Swr, Row2RotA,
};
use crate::{Row2RotB, Row2Swl, Row2Swr, Row3Swl, Row4Swl, Row5Swl, CLOCK};

/// Holder of input for the app.
pub struct AppInput {
    pub in_clock: DigitalIn<InClock>,
    pub in_reset: DigitalIn<InReset>,

    pub rot_row1: Encoder<QuadSource<Row1RotA, Row1RotB>>,
    pub rot_row2: Encoder<QuadSource<Row2RotA, Row2RotB>>,

    pub swr_row1: PushButton<Row1Swr>,
    pub swr_row2: PushButton<Row2Swr>,

    pub swl_row1: PushButton<Row1Swl>,
    pub swl_row2: PushButton<Row2Swl>,
    pub swl_row3: PushButton<Row3Swl>,
    pub swl_row4: PushButton<Row4Swl>,
    pub swl_row5: PushButton<Row5Swl>,

    pub last_clock: Option<Time<{ CLOCK }>>,
}

impl AppInput {
    pub fn read_input(&mut self, now: Time<{ CLOCK }>, col: Col, oper_queue: &mut OperQueue) {
        // Handle reset before clock, in case we handle them at the same time, the reset
        // should be handled in AppState before the clock.
        {
            let rst = self.in_reset.tick(now);
            // falling since inverted
            if let Some(Edge::Falling(_)) = rst {
                oper_queue.push(Oper::Reset);
            }
        }

        {
            let clk = self.in_clock.tick(now);
            // falling since inverted
            if let Some(Edge::Falling(_)) = clk {
                if let Some(last) = self.last_clock {
                    let interval = now - last;
                    oper_queue.push(Oper::Clock(interval));
                }
            }
        }

        // Read row 5 before other rows, since it has the "shift" button which affects
        // other keys being pushed after.
        {
            let swl_row5 = self.swl_row5.tick(now);
            if let Some(swl_row5) = swl_row5 {
                oper_queue.push(Oper::LedButton(Row(4), col, swl_row5.is_rising()));
            }
        }
        {
            let swl_row4 = self.swl_row4.tick(now);
            if let Some(swl_row4) = swl_row4 {
                oper_queue.push(Oper::LedButton(Row(3), col, swl_row4.is_rising()));
            }
        }
        {
            let swl_row3 = self.swl_row3.tick(now);
            if let Some(swl_row3) = swl_row3 {
                oper_queue.push(Oper::LedButton(Row(2), col, swl_row3.is_rising()));
            }
        }
        {
            let swl_row2 = self.swl_row2.tick(now);
            if let Some(swl_row2) = swl_row2 {
                oper_queue.push(Oper::LedButton(Row(1), col, swl_row2.is_rising()));
            }
        }
        {
            let swl_row1 = self.swl_row1.tick(now);
            if let Some(swl_row1) = swl_row1 {
                oper_queue.push(Oper::LedButton(Row(0), col, swl_row1.is_rising()));
            }
        }

        // Rotary encoder buttons
        {
            let swr_row2 = self.swr_row2.tick(now);
            if let Some(swr_row2) = swr_row2 {
                oper_queue.push(Oper::EncoderButton(Row(1), col, swr_row2.is_rising()));
            }
        }
        {
            let swr_row1 = self.swr_row1.tick(now);
            if let Some(swr_row1) = swr_row1 {
                oper_queue.push(Oper::EncoderButton(Row(0), col, swr_row1.is_rising()));
            }
        }

        // Rotary encoder knob
        {
            let rot_row1 = self.rot_row1.tick(now);
            if rot_row1 != 0 {
                oper_queue.push(Oper::RotaryEncoder(Row(0), col, rot_row1));
            }
        }
        {
            let rot_row2 = self.rot_row2.tick(now);
            if rot_row2 != 0 {
                oper_queue.push(Oper::RotaryEncoder(Row(1), col, rot_row2));
            }
        }
    }
}

pub type DigitalIn<A> = DigitalEdgeInput<PinDigitalIn<A>, { CLOCK }>;

pub type PushButton<A> =
    DigitalEdgeInput<DebounceDigitalInput<PinDigitalIn<A>, { CLOCK }>, { CLOCK }>;

pub struct QuadSource<A, B> {
    pub pin_a: A,
    pub pin_b: B,
}

impl<A, B> QuadratureSource for QuadSource<A, B>
where
    A: InputPin,
    <A as crate::hal::prelude::InputPin>::Error: Debug,
    B: InputPin,
    <B as crate::hal::prelude::InputPin>::Error: Debug,
{
    fn pin_a(&self) -> bool {
        self.pin_a.is_high().unwrap()
    }

    fn pin_b(&self) -> bool {
        self.pin_b.is_high().unwrap()
    }
}

pub struct PinDigitalIn<A>(pub A);

impl<A, const CLK: u32> DigitalInput<CLK> for PinDigitalIn<A>
where
    A: InputPin,
    <A as crate::hal::prelude::InputPin>::Error: Debug,
{
    fn tick(&mut self, now: Time<CLK>) -> HiLo<CLK> {
        if self.0.is_high().unwrap() {
            HiLo::Hi(now)
        } else {
            HiLo::Lo(now)
        }
    }
}
