#![no_std]
#![no_main]

#[macro_use]
extern crate defmt;

use alg::clock::Clock;
use alg::clock::Time;
use cortex_m_rt::entry;
use hal::gpio::{gpioa, gpiob, gpioc, gpiod, gpiof};
use hal::gpio::{DefaultMode, Floating, Input, OpenDrain, Output, PushPull};
use hal::prelude::*;
use hal::stm32 as pac;
use hal::time::Hertz;
use led_grid::{BiLed, LedGrid};
use stm32g0xx_hal as hal;

use crate::flip_pin::{FlipPin, IntoFlipPin};

mod flip_pin;
mod led_grid;

// Setup logging via defmt_rtt. "rtt" is "real time transfer"
use defmt_rtt as _;

// Log via defmt on panic.
use panic_probe as _;

// 48 MHz is max.
pub const CPU_SPEED: u32 = 48_000_000;

#[entry]
fn main() -> ! {
    // 1. HAL init
    // Set up ARM Cortex-M peripherals. These are common to many MCUs, including all STM32 ones.
    let cp = pac::CorePeripherals::take().unwrap();
    // Set up peripherals specific to the microcontroller you're using.
    let dp = pac::Peripherals::take().unwrap();

    // 2. System clock config
    let mut clocks = dp.RCC.constrain();

    let mut syst = cp.SYST;
    {
        syst.set_reload(1);
        syst.clear_current();
        syst.enable_counter();
    }

    let mut delay = syst.delay(&mut clocks);
    delay.delay_ms(1u8);

    // The global trace enable (DCB::enable_trace) should be set before enabling the cycle counter
    // cp.DCB.enable_trace();

    //  3. GPIO init.
    let gpioa = dp.GPIOA.split(&mut clocks);
    let gpiob = dp.GPIOB.split(&mut clocks);
    let gpioc = dp.GPIOC.split(&mut clocks);
    let gpiod = dp.GPIOD.split(&mut clocks);
    let gpiof = dp.GPIOF.split(&mut clocks);

    let spi = {
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

    let i2c = {
        use hal::i2c;
        let scl: I2cScl = gpiob.pb10.into_open_drain_output();
        let sda: I2cSda = gpiob.pb11.into_open_drain_output();
        i2c::I2c::i2c2(dp.I2C2, sda, scl, i2c::Config::new(Hertz(30)), &mut clocks)
    };

    let cs_fram: CsFRam = gpioa.pa1.into_push_pull_output();
    let cs_dac: CsDac = gpioa.pa2.into_push_pull_output();

    let in_reset: InReset = gpioc.pc13.into_floating_input();
    let in_sync: InSync = gpioc.pc14.into_floating_input();

    let row1_swl: Row1Swl = gpiod.pd2.into_floating_input();
    let row1_swr: Row1Swr = gpiod.pd1.into_floating_input();
    let row1_rot_a: Row1RotA = gpiod.pd0.into_floating_input();
    let row1_rot_b: Row1RotB = gpioa.pa15.into_floating_input();

    let row2_swl: Row2Swl = gpiob.pb2.into_floating_input();
    let row2_swr: Row2Swr = gpiob.pb12.into_floating_input();
    let row2_rot_a: Row2RotA = gpiob.pb13.into_floating_input();
    let row2_rot_b: Row2RotB = gpiob.pb14.into_floating_input();

    let row3_swl: Row3Swl = gpioa.pa6.into_floating_input();

    let row4_swl: Row4Swl = gpiob.pb0.into_floating_input();

    let row5_swl: Row5Swl = gpiof.pf0.into_floating_input();

    let out_gate1: OutGate1 = gpiob.pb5.into_push_pull_output();
    let out_gate2: OutGate2 = gpiob.pb7.into_push_pull_output();
    let out_gate3: OutGate3 = gpiob.pb4.into_push_pull_output();
    let out_gate4: OutGate4 = gpiob.pb6.into_push_pull_output();

    let mut led_grid = LedGrid {
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

    led_grid.set_row(0, BiLed::Off, &[BiLed::Off; 8]);

    // App state stuff

    let mut clock = Clock::<_, CPU_SPEED>::new(pac::SYST::get_current);

    let mut start = clock.now();
    let mut loop_count = 0_u64;

    info!("Starting…");

    loop {
        // wait for interrupt
        // cortex_m::asm::wfi();

        clock.tick();
        let now = clock.now();
        let time_lapsed = now - start;

        if time_lapsed > Time::from_millis(3300) {
            info!(
                "{} loop count: {}, {}µS/loop",
                now,
                loop_count,
                10_000_000.0 / loop_count as f32,
            );
            start = now;
            loop_count = 0;
        }

        loop_count += 1;
    }
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

pub type Spi2Sck = gpioa::PA0<DefaultMode>;
pub type Spi2Miso = gpioa::PA3<DefaultMode>;
pub type Spi2Mosi = gpioa::PA4<DefaultMode>;

pub type I2cScl = gpiob::PB10<Output<OpenDrain>>;
pub type I2cSda = gpiob::PB11<Output<OpenDrain>>;

pub type CsFRam = gpioa::PA1<Output<PushPull>>;
type CsDac = gpioa::PA2<Output<PushPull>>;

pub type InReset = gpioc::PC13<Input<Floating>>;
pub type InSync = gpioc::PC14<Input<Floating>>;

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
