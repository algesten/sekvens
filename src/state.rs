use alg::tempo::Tempo;

use crate::buttons::Buttons;
use crate::led_grid::BiLed;
use crate::mstate::MachineState;
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
    RotaryButton(Row, Col, bool),
}

#[derive(Default)]
pub struct AppState {
    mstate: MachineState,

    /// If next tick is going to reset back to 0.
    next_is_reset: bool,

    /// Beat detection/tempo
    tempo: Tempo<{ CLOCK }>,

    /// Interval to next predicted clock.
    predicted: Time,

    /// Ever increasing count of the clock. Never resets.
    clock_count: u64,

    /// Current global playhead. Goes from 0..whenever external reset comes.
    playhead: u64,

    /// Parameters for pattern and tracks.
    params: Params<{ TRACK_COUNT }>,

    /// Playhead for each track.
    track_playhead: [usize; TRACK_COUNT],

    /// Button state
    buttons: Buttons,

    /// LED states.
    ///
    /// row 0 - step row 1
    /// row 1 - step row 2
    /// row 2 - pattern (col 4-7)
    /// row 3 - track (col 4-7)
    /// row 4 - shift, clear, vel (col 4-7)
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
    pub fn apply_oper(&mut self, now: Time, oper: Oper) {
        match oper {
            Oper::Clock(interval) => {
                self.tempo.predict(interval);
                self.clock_count += 1;

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
                    self.clock_count
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
                if *row < 2 {
                    // 0-1 step button
                    self.buttons.set_step(*row, *col, on);
                } else {
                    // 2-4 top button
                    self.buttons.set_top(*row, *col, on)
                }
            }

            Oper::RotaryButton(row, col, on) => {
                self.buttons.set_rotary(*row, *col, on);
            }
        }
    }

    pub fn tick(&mut self, now: Time) {
        self.mstate.transition(&self.buttons);
        //
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
                TrackSync::Free => (self.clock_count % parm.tracks[i].length as u64) as usize,
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
