use alg::clock::Time;
use alg::tempo::Tempo;

use crate::led_grid::BiLed;
use crate::{Col, Row, CLOCK};

pub const TRACK_COUNT: usize = 4;

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

    /// LED states.
    leds: [[BiLed; 8]; 5],

    /// Parameters for pattern and tracks.
    params: Params<{ TRACK_COUNT }>,

    /// Playhead for each track.
    track_playhead: [usize; TRACK_COUNT],
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

impl AppState {
    pub fn new() -> Self {
        AppState {
            next_is_reset: false,
            tempo: Tempo::new(),
            predicted: Time::ZERO,
            tick_count: 0,
            playhead: 0,
            leds: [[BiLed::Off; 8]; 5],
            params: Params {
                pattern_length: 64,
                tracks: [TrackParams {
                    length: 64,
                    sync: TrackSync::Sync,
                }; TRACK_COUNT],
            },
            track_playhead: [0; TRACK_COUNT],
        }
    }

    pub fn led_row(&self, row: usize) -> &[BiLed; 8] {
        &self.leds[row]
    }
}

impl AppState {
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
                //
            }

            Oper::LedButton(row, col, on) => {
                //
            }

            Oper::EncoderButton(row, col, on) => {
                //
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
