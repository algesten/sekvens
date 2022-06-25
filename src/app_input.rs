use core::fmt::Debug;

use crate::hal::prelude::InputPin;
use alg::clock::Time;
use alg::encoder::{Encoder, QuadratureSource};
use alg::input::{DebounceDigitalInput, DeltaInput, DigitalEdgeInput};
use alg::input::{DigitalInput, Edge, EdgeInput, HiLo};

use crate::{InClock, InReset, Row1RotA, Row1RotB, Row1Swl, Row1Swr, Row2RotA};
use crate::{Row2RotB, Row2Swl, Row2Swr, Row3Swl, Row4Swl, Row5Swl, CPU_SPEED};

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
}

impl AppInput {
    pub fn read_input(
        &mut self,
        now: Time<{ CPU_SPEED }>,
        col: usize,
        update: &mut dyn AppInputUpdate<{ CPU_SPEED }>,
    ) {
        let clk = self.in_clock.tick(now);
        let rst = self.in_reset.tick(now);

        let rot_row1 = self.rot_row1.tick(now);
        let rot_row2 = self.rot_row2.tick(now);

        let swr_row1 = self.swr_row1.tick(now);
        let swr_row2 = self.swr_row2.tick(now);

        let swl_row1 = self.swl_row1.tick(now);
        let swl_row2 = self.swl_row2.tick(now);
        let swl_row3 = self.swl_row3.tick(now);
        let swl_row4 = self.swl_row4.tick(now);
        let swl_row5 = self.swl_row5.tick(now);

        update.update_input(
            col, clk, rst, rot_row1, rot_row2, swr_row1, swr_row2, swl_row1, swl_row2, swl_row3,
            swl_row4, swl_row5,
        );
    }
}

pub trait AppInputUpdate<const CLK: u32> {
    fn update_input(
        &mut self,
        col: usize,
        clk: Option<Edge<CLK>>,
        rst: Option<Edge<CLK>>,
        rot_row1: i8,
        rot_row2: i8,
        swr_row1: Option<Edge<CLK>>,
        swr_row2: Option<Edge<CLK>>,
        swl_row1: Option<Edge<CLK>>,
        swl_row2: Option<Edge<CLK>>,
        swl_row3: Option<Edge<CLK>>,
        swl_row4: Option<Edge<CLK>>,
        swl_row5: Option<Edge<CLK>>,
    );
}

pub type DigitalIn<A> = DigitalEdgeInput<PinDigitalIn<A>, { CPU_SPEED }>;

pub type PushButton<A> =
    DigitalEdgeInput<DebounceDigitalInput<PinDigitalIn<A>, { CPU_SPEED }>, { CPU_SPEED }>;

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
