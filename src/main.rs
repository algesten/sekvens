#![no_std]
#![no_main]

#[macro_use]
extern crate defmt;

use alg::clock::Clock;
use alg::clock::Time;
use alg::encoder::Encoder;
use alg::input::DigitalInput;
use cortex_m_rt::entry;
use hal::gpio::{gpioa, gpiob, gpioc, gpiod, gpiof};
use hal::gpio::{DefaultMode, Floating, Input, OpenDrain, Output, PushPull};
use hal::prelude::*;
use hal::stm32::{self as pac};
use hal::time::Hertz;
use input::QuadSource;
use led_grid::{BiLed, LedGridPins};
use stm32g0xx_hal as hal;

use crate::flip_pin::{FlipPin, IntoFlipPin};
use crate::input::{AppInput, PinDigitalIn};
use crate::led_grid::LedGrid;
use crate::state::{AppState, OperQueue};

mod flip_pin;
mod input;
mod led_grid;
mod state;

// Setup logging via defmt_rtt. "rtt" is "real time transfer"
use defmt_rtt as _;

// Log via defmt on panic.
use panic_probe as _;

// 48 MHz is max.
pub const _CPU_SPEED: u32 = 48_000_000;

// Clock speed is in microseconds.
pub const CLOCK: u32 = 1_000_000;

#[entry]
fn main() -> ! {
    // Ensure constants are sane.
    assert!(TIME_READ_WAIT.subsec_micros() < TIME_OFF);

    // 1. HAL init
    // Set up ARM Cortex-M peripherals. These are common to many MCUs, including all STM32 ones.
    let cp = pac::CorePeripherals::take().unwrap();
    // Set up peripherals specific to the microcontroller you're using.
    let dp = pac::Peripherals::take().unwrap();

    // 2. System clock config
    let mut clocks = dp.RCC.constrain();

    let mut delay = cp.SYST.delay(&mut clocks);
    delay.delay_us(1_u8);

    // The global trace enable (DCB::enable_trace) should be set before enabling the cycle counter
    // cp.DCB.enable_trace();

    //  3. GPIO init.
    let gpioa = dp.GPIOA.split(&mut clocks);
    let gpiob = dp.GPIOB.split(&mut clocks);
    let gpioc = dp.GPIOC.split(&mut clocks);
    let gpiod = dp.GPIOD.split(&mut clocks);
    let gpiof = dp.GPIOF.split(&mut clocks);

    let _spi = {
        use hal::spi;
        let sck: Spi2Sck = gpioa.pa0;
        let miso: Spi2Miso = gpioa.pa3;
        let mosi: Spi2Mosi = gpioa.pa4;
        spi::Spi::spi2(
            dp.SPI2,
            (sck, miso, mosi),
            spi::MODE_0,
            Hertz(3_000_000),
            &mut clocks,
        )
    };

    let _i2c = {
        use hal::i2c;
        let scl: I2cScl = gpiob.pb10.into_open_drain_output();
        let sda: I2cSda = gpiob.pb11.into_open_drain_output();
        i2c::I2c::i2c2(dp.I2C2, sda, scl, i2c::Config::new(Hertz(30)), &mut clocks)
    };

    let _cs_fram: CsFRam = gpioa.pa1.into_push_pull_output();
    let _cs_dac: CsDac = gpioa.pa2.into_push_pull_output();

    let in_reset: InReset = gpioc.pc13.into_floating_input();
    let in_clock: InClock = gpioc.pc14.into_floating_input();

    let row1_swl: Row1Swl = gpiod.pd2.into_floating_input();
    let row1_swr: Row1Swr = gpiod.pd1.into_floating_input();

    let row2_swl: Row2Swl = gpiob.pb2.into_floating_input();
    let row2_swr: Row2Swr = gpiob.pb12.into_floating_input();

    let row3_swl: Row3Swl = gpioa.pa6.into_floating_input();

    let row4_swl: Row4Swl = gpiob.pb0.into_floating_input();

    let row5_swl: Row5Swl = gpiof.pf0.into_floating_input();

    let _out_gate1: OutGate1 = gpiob.pb5.into_push_pull_output();
    let _out_gate2: OutGate2 = gpiob.pb7.into_push_pull_output();
    let _out_gate3: OutGate3 = gpiob.pb4.into_push_pull_output();
    let _out_gate4: OutGate4 = gpiob.pb6.into_push_pull_output();

    let led_grid_pins = LedGridPins {
        col1: gpioa.pa12.into_flip_pin(),
        col2: gpioa.pa11.into_flip_pin(),
        col3: gpioa.pa10.into_flip_pin(),
        col4: gpioc.pc7.into_flip_pin(),
        col5: gpioc.pc6.into_flip_pin(),
        col6: gpioa.pa9.into_flip_pin(),
        col7: gpioa.pa8.into_flip_pin(),
        col8: gpiob.pb15.into_flip_pin(),

        row1: gpiod.pd3.into_flip_pin(),
        row2: gpiob.pb1.into_flip_pin(),
        row3: gpioa.pa5.into_flip_pin(),
        row4: gpioa.pa7.into_flip_pin(),
        row5: gpiof.pf1.into_flip_pin(),
    };

    let mut app_input = AppInput {
        in_clock: PinDigitalIn(in_clock).edge(),
        in_reset: PinDigitalIn(in_reset).edge(),

        rot_row1: Encoder::new(QuadSource {
            pin_a: gpiod.pd0.into_floating_input(),
            pin_b: gpioa.pa15.into_floating_input(),
        }),
        rot_row2: Encoder::new(QuadSource {
            pin_a: gpiob.pb13.into_floating_input(),
            pin_b: gpiob.pb14.into_floating_input(),
        }),

        swr_row1: PinDigitalIn(row1_swr).debounce().edge(),
        swr_row2: PinDigitalIn(row2_swr).debounce().edge(),

        swl_row1: PinDigitalIn(row1_swl).debounce().edge(),
        swl_row2: PinDigitalIn(row2_swl).debounce().edge(),
        swl_row3: PinDigitalIn(row3_swl).debounce().edge(),
        swl_row4: PinDigitalIn(row4_swl).debounce().edge(),
        swl_row5: PinDigitalIn(row5_swl).debounce().edge(),

        last_clock: None,
    };

    let mut led_grid = LedGrid::new(led_grid_pins);

    led_grid.set_leds(0, BiLed::Off, &[BiLed::Off; 8]);

    let stop_watch = dp.TIM3.stopwatch(&mut clocks);

    let mut clock = {
        let orig = stop_watch.now();
        let sample_fn = move || stop_watch.elapsed(orig).0;
        Clock::<_, CLOCK>::new_with_bits(12, sample_fn)
    };

    // App state stuff

    let mut start = clock.now();
    let mut loop_count = 0_u64;

    let mut run_step_idx = 0;
    let mut run_step_time = GRID_STEPS[run_step_idx].time();
    let mut run_step_start = clock.now();
    let mut run_col = 0;
    let mut run_do_read = false;

    let mut app_state = AppState::new();
    let mut oper_queue = OperQueue::new();

    info!("Starting…");

    loop {
        clock.tick();
        let now = clock.now();

        {
            // Each RunStep has a time of how long we're going to "dwell",
            // on that step. This lapsed time is how long we have stayed
            // on the current step.
            let run_time_lapsed = now - run_step_start;

            if run_time_lapsed >= run_step_time {
                // Advance the RunStep forward in the GRID_STEPS, reset timers etc.
                run_step_idx += 1;
                run_step_idx %= GRID_STEPS.len();
                let run_step = &GRID_STEPS[run_step_idx];
                run_step_time = run_step.time();
                run_step_start = now;

                // The GridStep tells us what we are to do. Turning on LEDs,
                // and which to turn on, or read input (GridStep::Off) for
                // some certain column.
                match run_step {
                    GridStep::Led(_, on, row) => {
                        let leds = app_state.led_row(row.0);
                        led_grid.set_leds(row.0, *on, leds);
                    }
                    GridStep::Off(_, col) => {
                        led_grid.set_col(col.0);
                        run_col = col.0;
                        run_do_read = true;
                    }
                }
            }

            // TIME_READ_WAIT is a delay to let pin outputs settle between
            // setting the column to high and actually doing the read.
            if run_do_read && run_time_lapsed > TIME_READ_WAIT {
                run_do_read = false;

                // Read input and push update into app state.
                app_input.read_input(now, run_col, &mut oper_queue);
            }
        }

        let len = oper_queue.len();
        if len > 0 {
            /// Limit the number of operations we process max per loop so
            /// if there is a lot of input, we don't stall LED lighting etc.
            const MAX_CONSUME: usize = 3;

            let max = MAX_CONSUME.min(len);

            // Apply the operations to the state.
            app_state.update(now, oper_queue.drain(0..max));
        }

        {
            const REPORT_MILLIS: i64 = 1500;

            let time_lapsed = now - start;
            if time_lapsed > Time::from_millis(REPORT_MILLIS) {
                info!(
                    "{} {}µS/loop",
                    now,
                    (1000 * REPORT_MILLIS) as f32 / loop_count as f32,
                );
                start = now;
                loop_count = 0;
            }
            loop_count += 1;
        }
    }
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Time to keep red LEDs on.
const TIME_RED: i64 = 400;
/// Time to keep green LEDs on.
const TIME_GRN: i64 = 90;
/// Time in between LEDs being on.
const TIME_OFF: i64 = 400;
/// Time to wait after setting the grid up for
/// reading input and actually reading. Must
/// be less than TIME_OFF.
const TIME_READ_WAIT: Time<CLOCK> = Time::from_micros(50);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Row(usize);
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Col(usize);

pub enum GridStep {
    /// Light a row of LEDs
    Led(i64, BiLed, Row),
    /// Turn of all LEDs and read input for a column.
    Off(i64, Col),
}

impl GridStep {
    pub fn time(&self) -> Time<CLOCK> {
        let v = match self {
            GridStep::Led(v, _, _) => v,
            GridStep::Off(v, _) => v,
        };
        Time::from_micros(*v)
    }
}

static GRID_STEPS: &[GridStep] = &[
    GridStep::Led(TIME_RED, BiLed::Red, Row(0)),
    GridStep::Off(TIME_OFF, Col(0)),
    GridStep::Led(TIME_GRN, BiLed::Grn, Row(0)),
    GridStep::Off(TIME_OFF, Col(1)),
    GridStep::Led(TIME_RED, BiLed::Red, Row(1)),
    GridStep::Off(TIME_OFF, Col(2)),
    GridStep::Led(TIME_GRN, BiLed::Grn, Row(1)),
    GridStep::Off(TIME_OFF / 2, Col(3)),
    GridStep::Off(TIME_OFF / 2, Col(4)),
];

pub type Spi2Sck = gpioa::PA0<DefaultMode>;
pub type Spi2Miso = gpioa::PA3<DefaultMode>;
pub type Spi2Mosi = gpioa::PA4<DefaultMode>;

pub type I2cScl = gpiob::PB10<Output<OpenDrain>>;
pub type I2cSda = gpiob::PB11<Output<OpenDrain>>;

pub type CsFRam = gpioa::PA1<Output<PushPull>>;
type CsDac = gpioa::PA2<Output<PushPull>>;

pub type InReset = gpioc::PC13<Input<Floating>>;
pub type InClock = gpioc::PC14<Input<Floating>>;

pub type Row1Led = FlipPin<'D', 3>;
pub type Row1Swl = gpiod::PD2<Input<Floating>>;
pub type Row1Swr = gpiod::PD1<Input<Floating>>;
pub type Row1RotA = gpiod::PD0<Input<Floating>>;
pub type Row1RotB = gpioa::PA15<Input<Floating>>;

pub type Row2Led = FlipPin<'B', 1>;
pub type Row2Swl = gpiob::PB2<Input<Floating>>;
pub type Row2Swr = gpiob::PB12<Input<Floating>>;
pub type Row2RotA = gpiob::PB13<Input<Floating>>;
pub type Row2RotB = gpiob::PB14<Input<Floating>>;

pub type Row3Led = FlipPin<'A', 5>;
pub type Row3Swl = gpioa::PA6<Input<Floating>>;

pub type Row4Led = FlipPin<'A', 7>;
pub type Row4Swl = gpiob::PB0<Input<Floating>>;

pub type Row5Led = FlipPin<'F', 1>;
pub type Row5Swl = gpiof::PF0<Input<Floating>>;

pub type Col1 = FlipPin<'A', 12>;
pub type Col2 = FlipPin<'A', 11>;
pub type Col3 = FlipPin<'A', 10>;
pub type Col4 = FlipPin<'C', 7>;
pub type Col5 = FlipPin<'C', 6>;
pub type Col6 = FlipPin<'A', 9>;
pub type Col7 = FlipPin<'A', 8>;
pub type Col8 = FlipPin<'B', 15>;

pub type OutGate1 = gpiob::PB5<Output<PushPull>>;
pub type OutGate2 = gpiob::PB7<Output<PushPull>>;
pub type OutGate3 = gpiob::PB4<Output<PushPull>>;
pub type OutGate4 = gpiob::PB6<Output<PushPull>>;
