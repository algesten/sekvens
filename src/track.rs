use crate::music::Tone;

pub struct Track {
    /// Parameters for this track.
    pub params: TrackParams,

    /// All the track steps. Constrained by params.length.
    pub steps: [TrackStep; 128],
}

pub struct TrackParams {
    /// Length of track. In clock-ticks.
    pub length: usize,

    /// Track sync parameter.
    pub sync: TrackSync,

    /// Base velocity. Defaults to 80.
    ///
    /// * Range is 0 - 127
    pub base_velocity: i8,

    /// Velocity or LFO mode. Defaults to false, velocity mode.
    pub lfo_mode: bool,

    /// Base probability. Defaults to 100.
    ///
    /// * Range is 0 - 100.
    pub base_probability: i8,

    /// Base slew. Defaults to 0.
    ///
    /// * Rage is 0 - 100.
    /// * 0 is no slew
    /// * 50 is reaching the next note at half step length.
    /// * 100 is reaching the next note at step length.
    pub base_slew: i8,
}

#[derive(Default, Clone, Copy)]
pub struct TrackStep {
    /// Whether the step is playing or not.
    pub on: bool,

    /// Probability between -100 - 100. Defaults to 0.
    ///
    /// Added to track level probability. The end result i 0 - 100.
    pub probability: i8,

    /// Tone of step.
    pub tone: Tone,

    /// The length of the note. 255 is a legato and goes into the next.
    pub length: u8,

    /// Overrides the track length to 255.
    ///
    /// If we turn off the legato, the length is preserved.
    pub legato: bool,

    /// Velocity between -127 - 127. Defaults to 0.
    ///
    /// Added to track level velocity. The end result is 0 - 127.
    pub velocity: i8,

    /// Glissando speed going from this step to the next. Defaults to 0.
    ///
    /// Added to track level slew. The end result i 0 - 100.
    pub slew: i8,

    /// Micro offset.
    ///
    /// * -128 same time as previous step, i.e. -127 the min reasonable.
    /// *  128 same time as next step, i.e. 127 the max reasonable.
    pub offset: i8,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TrackSync {
    /// Track is restarted at pattern length and reset.
    #[default]
    Sync,
    /// Track is restarted only by reset.
    Reset,
    /// Track just keeps looping, ignoring both pattern length and reset.
    Free,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            params: TrackParams::default(),
            steps: [TrackStep::default(); 128],
        }
    }
}

impl Default for TrackParams {
    fn default() -> Self {
        Self {
            length: 128,
            sync: TrackSync::default(),
            base_velocity: 80,
            lfo_mode: false,
            base_probability: 100,
            base_slew: 0,
        }
    }
}
