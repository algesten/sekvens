use core::ops::{Deref, DerefMut};

const TONE_MIN: i8 = -30;
const TONE_MAX: i8 = 101;

/// 0 is C0, 1 C#0, 2 D0, etc.
///
/// Min that can be represented in volts is -30 (F#-1), Max is 101 (F10).
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Tone(pub i8);
impl Tone {
    pub(crate) fn add(&mut self, v: i8) {
        self.0 = self.0.saturating_add(v).clamp(TONE_MIN, TONE_MAX);
    }

    fn modulo(&self, n: i8) -> Tone {
        let mut r = *self;
        r.0 = r.0 % n;
        r
    }
}

impl Deref for Tone {
    type Target = i8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Tone {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Scale (or mode)
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// All the tones!
    Chromatic = 0,

    /// C D E F G A B C D E F G A
    /// 1 2 3 4 5 6 7 8 9 0 1 2 3
    ///
    /// Ionian
    #[default]
    Major = 1,

    /// C D E F# G A B C
    Lydian = 2,

    /// C D E F G A A# C
    ///
    /// Mixolydian
    Seven = 3,

    /// C D F E G A A# C
    ///
    /// Mixolydian + suspended 4
    Sus = 4,

    /// C D Eb F G Ab Bb C
    ///
    /// Natural minor. Aeolian.
    Minor = 5,

    /// C D Eb F G A Bb C
    ///
    /// Dorian. Jazz minor.
    Dorian = 6,

    /// C D Eb F G Ab B C
    ///
    /// Harmonic minor scale.
    Harmonic = 7,

    /// C Db Eb F G Ab Bb C
    Phrygian = 8,

    /// C Db E F G Ab Bb C
    ///
    /// Spanish Phrygian or Phrygian Dominant
    Spanish = 9,

    /// Diminished chord
    ///
    /// 8 tone scale?!
    ///
    /// C D Eb F Gb Ab A B
    /// 1 2 3  4 5  6  7 8
    Dim = 10,
}

impl From<u8> for Scale {
    fn from(value: u8) -> Self {
        use Scale::*;
        match value % 10 {
            0 => Chromatic,
            1 => Major,
            2 => Lydian,
            3 => Seven,
            4 => Sus,
            5 => Minor,
            6 => Dorian,
            7 => Harmonic,
            8 => Phrygian,
            9 => Spanish,
            10 => Dim,
            _ => unreachable!(),
        }
    }
}

impl Scale {
    // C  C# D  D# E  F  F# G  G# A A# B  C  C# D  D#  E F  F# G
    // 0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19
    pub fn as_tones(&self) -> &'static [Tone] {
        match self {
            Scale::Chromatic => &[
                Tone(0),
                Tone(1),
                Tone(2),
                Tone(3),
                Tone(4),
                Tone(5),
                Tone(6),
                Tone(7),
                Tone(8),
                Tone(9),
                Tone(10),
                Tone(11),
            ],

            Scale::Major => &[
                Tone(0),  // C
                Tone(2),  // D
                Tone(4),  // E
                Tone(5),  // F
                Tone(7),  // G
                Tone(9),  // A
                Tone(11), // B
            ],

            Scale::Lydian => &[
                Tone(0),  // C
                Tone(2),  // D
                Tone(4),  // E
                Tone(6),  // F#
                Tone(7),  // G
                Tone(9),  // A
                Tone(11), // B
            ],

            Scale::Seven => &[
                Tone(0),  // C
                Tone(2),  // D
                Tone(4),  // E
                Tone(5),  // F
                Tone(7),  // G
                Tone(9),  // A
                Tone(10), // A#
            ],

            Scale::Sus => &[
                Tone(0),  // C
                Tone(2),  // D
                Tone(5),  // F
                Tone(4),  // E
                Tone(7),  // G
                Tone(9),  // A
                Tone(11), // A#
            ],

            Scale::Minor => &[
                Tone(0),  // C
                Tone(2),  // D
                Tone(3),  // Eb
                Tone(5),  // F
                Tone(7),  // G
                Tone(8),  // Ab
                Tone(10), // Bb
            ],

            Scale::Dorian => &[
                Tone(0),  // C
                Tone(2),  // D
                Tone(3),  // Eb
                Tone(5),  // F
                Tone(7),  // G
                Tone(9),  // A
                Tone(10), // Bb
            ],

            Scale::Harmonic => &[
                Tone(0),  // C
                Tone(2),  // D
                Tone(3),  // Eb
                Tone(5),  // F
                Tone(7),  // G
                Tone(8),  // Ab
                Tone(11), // B
            ],

            Scale::Phrygian => &[
                Tone(0),  // C
                Tone(1),  // Db
                Tone(3),  // Eb
                Tone(5),  // F
                Tone(7),  // G
                Tone(8),  // Ab
                Tone(10), // Bb
            ],

            Scale::Spanish => &[
                Tone(0),  // C
                Tone(1),  // Db
                Tone(4),  // E
                Tone(5),  // F
                Tone(7),  // G
                Tone(8),  // Ab
                Tone(10), // Bb
            ],

            Scale::Dim => &[
                Tone(0),  // C
                Tone(2),  // D
                Tone(3),  // Eb
                Tone(5),  // F
                Tone(6),  // Gb
                Tone(8),  // Ab
                Tone(9),  // A
                Tone(11), // B
            ],
        }
    }

    pub fn add(&mut self, v: i8) {
        let n = (*self as i8).saturating_add(v).clamp(0, 9) as u8;
        *self = n.into();
    }
}

impl From<i8> for Tone {
    fn from(value: i8) -> Self {
        Tone(value)
    }
}

impl From<Tone> for i8 {
    fn from(value: Tone) -> Self {
        value.0
    }
}

pub fn spread(tones: &[Tone], amount: usize) -> [Tone; 4] {
    let tone_count = tones.len();
    assert!(tone_count > 0 && tone_count <= 4);

    let spread_pat = SPREADS[tone_count - 1];
    let spread_row = spread_pat[amount.clamp(0, spread_pat.len())];

    let mut ret = [
        tones[0 % tone_count].modulo(12),
        tones[1 % tone_count].modulo(12),
        tones[2 % tone_count].modulo(12),
        tones[3 % tone_count].modulo(12),
    ];

    let Some(idx_a) = spread_row.iter().position(|x| *x == b'A') else {
        return ret;
    };

    let octave_a = idx_a / 4;
    ret[0].add(octave_a as i8);

    let Some(idx_b) = spread_row.iter().position(|x| *x == b'B') else {
        return ret;
    };

    let octave_b = idx_b / 4;
    ret[1].add(octave_b as i8);

    let idx_c = spread_row.iter().position(|x| *x == b'C');
    let idx_d = spread_row.iter().position(|x| *x == b'D');

    ret
}

const SPREADS: [&[&[u8]]; 4] = [SPREAD_1, SPREAD_2, SPREAD_3, SPREAD_4];
const SPREAD_1: &[&[u8]] = &[b"A"];
#[rustfmt::skip]
const SPREAD_2: &[&[u8]] = &[
    b"AB",
    b"A B",
    b"A  B",
    b"A   B",
    b"A    B",
    b"A     B",
    b"A      B",
    b"A       B",
    b"A        B",
    b"A         B",
    b"A          B",
    b"A           B",
    b"A            B",
    b"A             B",
    b"A              B",
    b"A               B",
];
#[rustfmt::skip]
const SPREAD_3: &[&[u8]] = &[
    b"ABC",
    b"A BC",
    b"A B C",
    b"A  B C",
    b"A  B  C",
    b"A  B   C",
    b"A   B   C",
    b"A   B    C",
    b"A    B    C",
    b"A    B     C",
    b"A     B     C",
    b"A     B      C",
    b"A      B      C",
    b"A      B       C",
    b"A       B       C",
    b"A       B        C",
    b"A        B        C",
];
#[rustfmt::skip]
const SPREAD_4: &[&[u8]] = &[
    b"ABCD",
    b"AB CD",
    b"A B CD",
    b"A B C D",
    b"A B  C D",
    b"A B  C  D",
    b"A  B  C  D",
    b"A  B  C   D",
    b"A  B   C   D",
    b"A   B   C   D",
    b"A   B   C    D",
    b"A   B    C    D",
    b"A    B    C    D",
    b"A    B    C     D",
    b"A    B     C     D",
    b"A     B     C     D",
];
