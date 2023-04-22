use alg::tempo::Tempo;

use crate::button::Button;
use crate::led_grid::BiLed;
use crate::{Col, Row, Time, CLOCK};

pub const TRACK_COUNT: usize = 4;

#[derive(Copy, Clone, defmt::Format)]
/// The operations that can be done on the state.
pub enum Oper {
    /// Clock pulse. The time is the interval from the previous clock pulse.
    Clock(Time),

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
    mstate: MachineState,

    /// If next tick is going to reset back to 0.
    next_is_reset: bool,

    /// Beat detection/tempo
    tempo: Tempo<{ CLOCK }>,

    /// Interval to next predicted clock tick.
    predicted: Time,

    /// Ever increasing count of the clock tick. Never resets.
    tick_count: u64,

    /// Current global playhead. Goes from 0..whenever external reset comes.
    playhead: u64,

    /// Parameters for pattern and tracks.
    params: Params<{ TRACK_COUNT }>,

    /// Playhead for each track.
    track_playhead: [usize; TRACK_COUNT],

    /// Buttons top right.
    buttons_top: [[Button; 4]; 3],

    /// Step buttons underneath each rotary encoder.
    buttons_step: [[Button; 8]; 2],

    /// Step buttons that is the rotary encoder.
    buttons_step_rot: [[Button; 8]; 2],

    /// LED states.
    leds: [[BiLed; 8]; 5],
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum MachineState {
    #[default]
    Normal,
    Velocity,
    Shift,
    ChordUpper,
    ChordLower,
    Reset,
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
    pub fn apply_oper(&mut self, now: Time, oper: Oper) {
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

            Oper::LedButton(row, col, on) => {
                if row.0 < 2 {
                    self.buttons_step[row.0][col.0].set_on(on, now);
                } else if row.0 >= 2 && row.0 <= 4 {
                    let col = if col.0 >= 4 { col.0 - 4 } else { col.0 };
                    self.buttons_top[row.0 - 2][col].set_on(on, now);
                } else {
                    panic!("Unknown LedButton {:?} {:?}", row, col);
                }
            }

            Oper::EncoderButton(row, col, on) => {
                if row.0 < 2 {
                    self.buttons_step_rot[row.0][col.0].set_on(on, now);
                } else {
                    panic!("Unknown EncoderButton {:?} {:?}", row, col);
                }
            }

            Oper::RotaryEncoder(row, col, v) => {
                // match (row, col) {
                // }
            }
        }
    }

    pub fn tick(&mut self, now: Time) {
        let any_button_change = self.tick_buttons(now);

        //
    }

    // Costs ~90uS
    fn tick_buttons(&mut self, now: Time) -> bool {
        let mut any_change = false;

        for row in &mut self.buttons_top {
            for button in row {
                let state_change = button.tick(now);
                any_change |= state_change;
            }
        }

        for row in &mut self.buttons_step {
            for button in row {
                any_change |= button.tick(now);
            }
        }

        for row in &mut self.buttons_step_rot {
            for button in row {
                any_change |= button.tick(now);
            }
        }

        any_change
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
