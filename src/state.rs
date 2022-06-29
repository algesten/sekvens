use alg::clock::Time;
use alg::tempo::Tempo;

use crate::led_grid::BiLed;
use crate::{Col, Row, CLOCK};

pub const TRACK_COUNT: usize = 4;

#[derive(Copy, Clone, defmt::Format)]
/// The operations that can be done on the state.
pub enum Oper {
    /// Clock pulse. The time is the interval from the previous clock pulse.
    Clock(Time<{ CLOCK }>),

    /// Reset.
    Reset,

    /// Input from a rotary encoder.
    RotaryEncoder(Row, Col, i8),

    /// Input from a LED push button.
    LedButton(Row, Col, bool),

    /// Input from a rotary encoder button.
    EncoderButton(Row, Col, bool),
}

#[derive(Default)]
pub struct AppState {
    /// If next tick is going to reset back to 0.
    pub next_is_reset: bool,

    /// Beat detection/tempo
    tempo: Tempo<{ CLOCK }>,

    /// Interval to next predicted clock tick.
    predicted: Time<{ CLOCK }>,

    /// Ever increasing count of the clock tick. Never resets.
    tick_count: u64,

    /// Current global playhead. Goes from 0..whenever external reset comes.
    playhead: u64,

    /// Parameters for pattern and tracks.
    params: Params<{ TRACK_COUNT }>,

    /// Playhead for each track.
    track_playhead: [usize; TRACK_COUNT],

    /// Buttons top right.
    top_buttons: [[Button; 4]; 3],

    /// Step buttons underneath each rotary encoder.
    step_buttons: [[Button; 8]; 2],

    /// Step buttons that is the rotary encoder.
    step_rot_buttons: [[Button; 8]; 2],

    /// LED states.
    leds: [[BiLed; 8]; 5],
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            ..Default::default()
        }
    }

    pub fn led_row(&self, row: usize) -> &[BiLed; 8] {
        &self.leds[row]
    }
}

impl AppState {
    #[inline(never)]
    pub fn apply_oper(&mut self, now: Time<{ CLOCK }>, oper: Oper) {
        match oper {
            Oper::Clock(interval) => {
                self.tempo.predict(interval);
                self.tick_count += 1;

                if self.next_is_reset {
                    self.next_is_reset = false;
                    self.playhead = 0;
                } else {
                    self.playhead += 1;
                }

                self.update_track_playhead();

                trace!(
                    "Tick playhead: {} tick_count: {}",
                    self.playhead,
                    self.tick_count
                );
            }

            Oper::Reset => {
                // Reset might affect the tempo detection.
                self.tempo.reset();

                // Whatever tick is coming next, it's going to reset back to 0.
                self.next_is_reset = true;

                info!("Reset");
            }

            Oper::RotaryEncoder(row, col, v) => {
                // match (row, col) {
                // }
            }

            Oper::LedButton(row, col, on) => {
                if row.0 < 2 {
                    self.step_buttons[row.0][col.0].set_on(on, now);
                } else if row.0 >= 2 && row.0 <= 4 {
                    let col = if col.0 >= 4 { col.0 - 4 } else { col.0 };
                    self.top_buttons[row.0 - 2][col].set_on(on, now);
                } else {
                    panic!("Unknown LedButton {:?} {:?}", row, col);
                }
            }

            Oper::EncoderButton(row, col, on) => {
                if row.0 < 2 {
                    self.step_rot_buttons[row.0][col.0].set_on(on, now);
                } else {
                    panic!("Unknown EncoderButton {:?} {:?}", row, col);
                }
            }
        }
    }

    /// Current playhead, 0-63 for instance (depends on pattern length).
    pub fn playhead(&self) -> usize {
        (self.playhead % self.params.pattern_length as u64) as usize
    }

    fn update_track_playhead(&mut self) {
        let parm = &self.params;
        let plen = parm.pattern_length as usize;
        let playhead = self.playhead();

        for i in 0..TRACK_COUNT {
            self.track_playhead[i] = match self.params.tracks[i].sync {
                TrackSync::Sync => playhead % plen.min(parm.tracks[i].length as usize),
                TrackSync::Reset => (self.playhead % parm.tracks[i].length as u64) as usize,
                TrackSync::Free => (self.tick_count % parm.tracks[i].length as u64) as usize,
            };
        }
    }
}

#[derive(Default)]
struct Button {
    /// When the button is pushed down.
    on: Option<Time<{ CLOCK }>>,
}

impl Button {
    fn set_on(&mut self, on: bool, now: Time<{ CLOCK }>) {
        if on {
            self.on = Some(now);
        } else {
            self.on = None;
        }
    }

    fn state(&self, now: Time<{ CLOCK }>) -> ButtonState {
        if let Some(on) = &self.on {
            if now - *on > Time::from_millis(200) {
                ButtonState::LongPressed
            } else {
                ButtonState::Pressed
            }
        } else {
            ButtonState::Off
        }
    }

    fn is_pressed(&self, now: Time<{ CLOCK }>) -> bool {
        !matches!(self.state(now), ButtonState::Off)
    }

    fn is_long_pressed(&self, now: Time<{ CLOCK }>) -> bool {
        matches!(self.state(now), ButtonState::LongPressed)
    }
}

enum ButtonState {
    Off,
    Pressed,
    LongPressed,
}

pub struct Params<const X: usize> {
    pattern_length: u8,
    tracks: [TrackParams; X],
}

#[derive(Clone, Copy)]
pub struct TrackParams {
    /// Length of track. In clock-ticks.
    pub length: u8,

    /// Track sync parameter.
    pub sync: TrackSync,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrackSync {
    /// Track is restarted at pattern length and reset.
    Sync,
    /// Track is restarted only by reset.
    Reset,
    /// Track just keeps looping, ignoring both pattern length and reset.
    Free,
}

impl Default for TrackParams {
    fn default() -> Self {
        Self {
            length: 64,
            sync: TrackSync::Sync,
        }
    }
}

impl<const X: usize> Default for Params<X> {
    fn default() -> Self {
        Self {
            pattern_length: 64,
            tracks: [TrackParams::default(); X],
        }
    }
}
