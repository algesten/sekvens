use crate::val::Val;

pub struct Track {
    /// Parameters for this track.
    pub params: TrackParams,

    /// All the track steps. Constrained by params.length.
    pub steps: [TrackStep; 128],
}

pub struct TrackParams {
    /// Length of track. In clock-ticks.
    pub track_length: usize,

    /// Track sync parameter.
    pub sync: TrackSync,

    /// Base step length. Defaults to 50.
    pub base_step_length: Val<0, 100>,

    /// Base velocity. Defaults to 80.
    ///
    /// * Range is 0 - 127
    pub base_velocity: Val<0, 127>,

    /// Velocity or LFO mode. Defaults to false, velocity mode.
    pub lfo_mode: bool,

    /// Base probability. Defaults to 100.
    ///
    /// * Range is 0 - 100.
    pub base_probability: Val<0, 100>,

    /// Base slew. Defaults to 0.
    ///
    /// * Rage is 0 - 100.
    /// * 0 is no slew
    /// * 50 is reaching the next note at half step length.
    /// * 100 is reaching the next note at step length.
    pub base_slew: Val<0, 100>,
}

impl TrackParams {
    pub fn add_length(&mut self, v: i8) {
        self.track_length = (self.track_length as isize)
            .saturating_add(v as isize)
            .clamp(1, 128) as usize;
    }
}

#[derive(Clone, Copy)]
pub struct TrackStep {
    /// Whether the step is playing or not.
    pub on: bool,

    /// Probability between -100 - 100. Defaults to 0.
    ///
    /// Added to track level probability. The end result i 0 - 100.
    pub probability: Val<-100, 100>,

    /// Tone of step as an offset from the base tone. Defaults to 0.
    ///
    /// Value is added to track base tone.
    pub tone: Val<-100, 100>,

    /// The amount of spread applied to this step (this is only relevant when selecting more
    /// than one track, and the lowest track's spread is used.
    pub spread: Val<0, 10>,

    /// Scale of the step as an offset from the base scale. Defaults to 0.
    ///
    /// 0 is base scale, we can go 8 steps down and then wrap around to 9 steps above.
    pub scale: Val<-8, 9>,

    /// The length of the note. 100 is a legato and goes into the next. 1 is 1/100 of the step length.
    /// This value is added to the base length. Defaults to 0.
    pub length: Val<-100, 100>,

    /// Overrides the track length to 100.
    ///
    /// If we turn off the legato, the length is preserved.
    pub legato: bool,

    /// Velocity between -127 - 127. Defaults to 0.
    ///
    /// Added to track level velocity. The end result is 0 - 127.
    pub velocity: Val<-127, 127>,

    /// Glissando speed going from this step to the next. Defaults to 0.
    ///
    /// Added to track level slew. The end result i 0 - 100.
    pub slew: Val<-100, 100>,

    /// Micro offset. Defaults to 0.
    ///
    /// * -128 same time as previous step, i.e. -127 the min reasonable.
    /// *  128 same time as next step, i.e. 127 the max reasonable.
    pub offset: Val<-127, 127>,
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
            track_length: 128,
            sync: TrackSync::default(),
            base_step_length: Val(50),
            base_velocity: Val(80),
            lfo_mode: false,
            base_probability: Val(100),
            base_slew: Val(0),
        }
    }
}

impl Default for TrackStep {
    fn default() -> Self {
        Self {
            on: Default::default(),
            probability: Val(0),
            tone: Val(0),
            spread: Val(0),
            scale: Val(0),
            length: Val(0),
            legato: Default::default(),
            velocity: Val(0),
            slew: Val(0),
            offset: Val(0),
        }
    }
}
