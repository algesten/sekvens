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

        // "Falling" because the input is inverted
        if matches!(self.in_reset.tick(now), Some(Edge::Falling(_))) {
            oper_queue.push(Oper::Reset);
        }

        // "Falling" because the input is inverted
        if matches!(self.in_clock.tick(now), Some(Edge::Falling(_))) {
            if let Some(last) = self.last_clock {
                let interval = now - last;
                oper_queue.push(Oper::Clock(interval));
            }
        }

        // Read row 5 before other rows, since it has the "shift" button which affects
        // other keys being pushed after.
        if let Some(edge_row5) = self.swl_row5.tick(now) {
            oper_queue.push(Oper::LedButton(Row(4), col, edge_row5.is_rising()));
        }
        if let Some(edge_row4) = self.swl_row4.tick(now) {
            oper_queue.push(Oper::LedButton(Row(3), col, edge_row4.is_rising()));
        }
        if let Some(edge_row3) = self.swl_row3.tick(now) {
            oper_queue.push(Oper::LedButton(Row(2), col, edge_row3.is_rising()));
        }
        if let Some(edge_row2) = self.swl_row2.tick(now) {
            oper_queue.push(Oper::LedButton(Row(1), col, edge_row2.is_rising()));
        }
        if let Some(edge_row1) = self.swl_row1.tick(now) {
            oper_queue.push(Oper::LedButton(Row(0), col, edge_row1.is_rising()));
        }

        // Rotary encoder buttons
        if let Some(edge_row2) = self.swr_row2.tick(now) {
            oper_queue.push(Oper::EncoderButton(Row(1), col, edge_row2.is_rising()));
        }
        if let Some(edge_row1) = self.swr_row1.tick(now) {
            oper_queue.push(Oper::EncoderButton(Row(0), col, edge_row1.is_rising()));
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
    B: InputPin,
{
    fn pin_a(&self) -> bool {
        if let Ok(v) = self.pin_a.is_high() {
            return v;
        } else {
            panic!("pin_a err");
        }
    }

    fn pin_b(&self) -> bool {
        if let Ok(v) = self.pin_b.is_high() {
            return v;
        } else {
            panic!("pin_b err");
        }
    }
}

pub struct PinDigitalIn<A>(pub A);

impl<A, const CLK: u32> DigitalInput<CLK> for PinDigitalIn<A>
where
    A: InputPin,
{
    fn tick(&mut self, now: Time<CLK>) -> HiLo<CLK> {
        if let Ok(v) = self.0.is_high() {
            if v {
                HiLo::Hi(now)
            } else {
                HiLo::Lo(now)
            }
        } else {
            panic!("tick err");
        }
    }
}
