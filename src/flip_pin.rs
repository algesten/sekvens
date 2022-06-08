#![allow(unused)]

use crate::pac::{GPIOA, GPIOB, GPIOC, GPIOD, GPIOF};

/// Generic pin type
///
/// - `P` is port name: `A` for GPIOA, `B` for GPIOB, etc.
/// - `N` is pin number: from `0` to `15`.
/// - `MODE` is one of the pin modes (see [Modes](crate::gpio#modes) section).
pub struct FlipPin<const P: char, const N: u8> {
    mode: Mode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Input,
    Output,
}

pub trait IntoFlipPin<const P: char, const N: u8> {
    fn into_flip_pin(self) -> FlipPin<P, N>;
}

macro_rules! flip_pin {
    ($GPIOX:ident, $gpiox:ident, $port_id:expr, [
        $($PX: ident: ($pxi:ident, $i:expr),)+
    ]) => {
        pub mod $gpiox {
            use super::*;
            $(
                use stm32g0xx_hal::gpio::$gpiox::$PX;

                impl<MODE> IntoFlipPin<$port_id, $i> for $PX<MODE> {
                    fn into_flip_pin(self) -> FlipPin<$port_id, $i> {
                        let mut x: FlipPin<$port_id, $i> = FlipPin {
                            mode: Mode::Input
                        };
                        x.set_floating_input();
                        x
                    }
                }
            )+
        }

        impl<const N: u8> FlipPin<$port_id, N> {
            /// Configures the pin to operate as a floating input pin
            #[inline(always)]
            fn set_floating_input(&mut self) {
                if self.mode == Mode::Input {
                    return;
                }

                let offset = 2 * N;
                unsafe {
                    let gpio = &(*$GPIOX::ptr());
                    gpio.pupdr
                        .modify(|r, w| w.bits(r.bits() & !(0b11 << offset)));
                    gpio.moder
                        .modify(|r, w| w.bits(r.bits() & !(0b11 << offset)))
                };

                self.mode = Mode::Input;
            }

            /// Configures the pin to operate as a push pull output pin
            #[inline(always)]
            fn set_push_pull_output(&mut self) {
                if self.mode == Mode::Output {
                    return;
                }

                let offset = 2 * N;
                unsafe {
                    let gpio = &(*$GPIOX::ptr());
                    gpio.pupdr
                        .modify(|r, w| w.bits(r.bits() & !(0b11 << offset)));
                    gpio.otyper.modify(|r, w| w.bits(r.bits() & !(0b1 << N)));
                    gpio.moder
                        .modify(|r, w| w.bits((r.bits() & !(0b11 << offset)) | (0b01 << offset)))
                };

                self.mode = Mode::Output;
            }

            pub fn set_output(&mut self, high: bool) {
                self.set_push_pull_output();
                if high {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << N)) };
                } else {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << (N + 16))) };
                }
            }

            pub fn disable(&mut self) {
                self.set_floating_input();
            }
        }
    };
}

flip_pin!(
    GPIOA,
    gpioa,
    'A',
    [
        PA0: (pa0, 0),
        PA1: (pa1, 1),
        PA2: (pa2, 2),
        PA3: (pa3, 3),
        PA4: (pa4, 4),
        PA5: (pa5, 5),
        PA6: (pa6, 6),
        PA7: (pa7, 7),
        PA8: (pa8, 8),
        PA9: (pa9, 9),
        PA10: (pa10, 10),
        PA11: (pa11, 11),
        PA12: (pa12, 12),
        PA13: (pa13, 13),
        PA14: (pa14, 14),
        PA15: (pa15, 15),
    ]
);

flip_pin!(
    GPIOB,
    gpiob,
    'B',
    [
        PB0: (pb0, 0),
        PB1: (pb1, 1),
        PB2: (pb2, 2),
        PB3: (pb3, 3),
        PB4: (pb4, 4),
        PB5: (pb5, 5),
        PB6: (pb6, 6),
        PB7: (pb7, 7),
        PB8: (pb8, 8),
        PB9: (pb9, 9),
        PB10: (pb10, 10),
        PB11: (pb11, 11),
        PB12: (pb12, 12),
        PB13: (pb13, 13),
        PB14: (pb14, 14),
        PB15: (pb15, 15),
    ]
);

flip_pin!(
    GPIOC,
    gpioc,
    'C',
    [
        //
        PC6: (pc6, 6),
        PC7: (pc7, 7),
        PC13: (pc13, 13),
        PC14: (pc14, 14),
        PC15: (pc15, 15),
    ]
);

flip_pin!(
    GPIOD,
    gpiod,
    'D',
    [
        //
        PD0: (pd0, 0),
        PD1: (pd1, 1),
        PD2: (pd2, 2),
        PD3: (pd3, 3),
    ]
);

flip_pin!(
    GPIOF,
    gpiof,
    'F',
    [
        //
        PF0: (pf0, 0),
        PF1: (pf1, 1),
    ]
);
