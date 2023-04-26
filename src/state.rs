use alg::bitfield::Bitfield;
use alg::tempo::Tempo;

use crate::buttons::Buttons;
use crate::led_grid::BiLed;
use crate::mstate::MachineState;
use crate::music::{Scale, Tone};
use crate::track::{Track, TrackStep, TrackSync};
use crate::val::Val;
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

    /// Parameters for pattern.
    params: PatternParams,

    /// Which tracks are currently selected.
    selected_tracks: TrackSelection,

    /// Currently selected track part. 0-3
    selected_part: usize,

    /// Tracks with parameters and notes.
    tracks: [Track; TRACK_COUNT],

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

#[derive(Default)]
struct TrackSelection(Bitfield);

impl TrackSelection {
    fn set(&mut self, track: u8, on: bool) {
        assert!(track < TRACK_COUNT as u8);
        self.0.set(track, on);
    }

    fn selected(&self) -> impl Iterator<Item = usize> {
        [self.0.is(0), self.0.is(1), self.0.is(2), self.0.is(3)]
            .into_iter()
            .enumerate()
            .filter(|(_, on)| *on)
            .map(|(i, _)| i)
    }
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

    pub fn apply_oper(&mut self, now: Time, oper: Oper) {
        match oper {
            Oper::Clock(interval) => {
                self.predicted = self.tempo.predict(interval);
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
                self.handle_rotary(row, col, v);
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
        (self.playhead % self.params.length as u64) as usize
    }

    fn update_track_playhead(&mut self) {
        let parm = &self.params;
        let plen = parm.length as usize;
        let playhead = self.playhead();

        for i in 0..TRACK_COUNT {
            let track = &self.tracks[i];
            let len = track.params.track_length;
            self.track_playhead[i] = match track.params.sync {
                TrackSync::Sync => playhead % plen.min(len as usize),
                TrackSync::Reset => (self.playhead % len as u64) as usize,
                TrackSync::Free => (self.clock_count % len as u64) as usize,
            };
        }
    }

    fn handle_rotary(&mut self, row: Row, col: Col, v: i8) {
        match self.mstate {
            MachineState::Normal => self.handle_rotary_normal(row, col, v),
            MachineState::Shift => self.handle_rotary_shift(row, col, v),
            MachineState::Velocity => self.handle_rotary_vel(row, col, v),
            MachineState::HoldUpper(_) if *row == 1 => self.handle_rotary_hold(true, col, v),
            MachineState::HoldLower(_) if *row == 0 => self.handle_rotary_hold(false, col, v),
            _ => {}
        }
    }

    fn handle_rotary_normal(&mut self, row: Row, col: Col, v: i8) {
        for i in self.selected_tracks.selected() {
            let step = step_for(self.selected_part, &mut self.tracks[i], row, col);
            step.tone.add(v);
        }
    }

    fn handle_rotary_shift(&mut self, row: Row, col: Col, v: i8) {
        if *row == 0 {
            // Track functions.
            for i in self.selected_tracks.selected() {
                let track = &mut self.tracks[i];
                match *col {
                    0 => track.params.add_length(v),
                    1 => track.params.base_velocity.add(v),
                    2 => track.params.base_probability.add(v),
                    3 => track.params.base_slew.add(v),
                    _ => {}
                }
            }
        } else if *row == 1 {
            // Global/pattern functions
            match *col {
                0 => self.params.add_length(v),
                1 => self.params.swing.add(v),
                6 => self.params.scale.add(v),
                7 => self.params.root.add(v),
                _ => {}
            }
        }
    }

    fn handle_rotary_vel(&mut self, row: Row, col: Col, v: i8) {
        for i in self.selected_tracks.selected() {
            let step = step_for(self.selected_part, &mut self.tracks[i], row, col);
            step.velocity.add(v);
        }
    }

    fn handle_rotary_hold(&mut self, upper: bool, col: Col, v: i8) {
        // NB we won't get here if it's a rotary on the same road as the
        // pushed down one.
        let row = if upper { Row(1) } else { Row(0) };
        let Some(s) = self.buttons.rotary_to_step() else {
            return;
        };

        for i in self.selected_tracks.selected() {
            let step = step_for(self.selected_part, &mut self.tracks[i], row, col);
            match s {
                0 => step.length.add(v),
                1 => step.velocity.add(v),
                2 => step.probability.add(v),
                3 => step.slew.add(v),
                _ => {}
            }
        }
        // 4 spread
        // 5 pitch
        // 6 scale
        // 7 root
    }
}

fn step_for(selected_part: usize, track: &mut Track, row: Row, col: Col) -> &mut TrackStep {
    // 16 steps per part
    let offset_part = selected_part * 16;
    &mut track.steps[offset_part + *row * 16 + *col]
}

struct PatternParams {
    /// Length of entire pattern. 1-128
    pub length: usize,

    /// Percentage swing. Default 50.
    ///
    /// 50% is perfectly straight < 50 note is early > 50 note is late.
    pub swing: Val<1, 99>,

    /// How the pattern is played. Default is forward.
    pub direction: PlayDirection,

    /// Whether we are playing or paused right now. Defaults to true.
    pub play: bool,

    /// The root key.
    pub root: Tone,

    /// The default scale for the pattern. Can be overridden by step.
    pub scale: Scale,
}

impl PatternParams {
    pub fn add_length(&mut self, v: i8) {
        self.length = (self.length as isize)
            .saturating_add(v as isize)
            .clamp(1, 128) as usize;
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum PlayDirection {
    #[default]
    Forward,
    Backward,
    Random,
}

impl Default for PatternParams {
    fn default() -> Self {
        Self {
            length: 16,
            swing: Val(50),
            direction: PlayDirection::default(),
            play: true,
            root: Tone(0), // C2
            scale: Scale::Major,
        }
    }
}
