sekvens
=======

# Functional requirements

## 4 tracks

Sekvens has 4 track outputs. Each track output consists of:

* A PITCH CV with volt per octave.
* A GATE signal.
* An LFO CV (also called "accent") output.

Each time the step of the sequencer changes, the outputs can change for each channel according to the settings of that step.

## Track lengths

The four tracks have individual lengths. Each track will play its full length and then loop from the beginning. The loop can either be synced to the RESET signal, or run free.

### 16 steps

The 16 steps are laid out with individual controls for each step. The step moves forward with each pulse of the inomcing SYNC signal.

### 8 parts

`G` is green diode. `R` is red diode. To access part 1-4, short push on the part button. To access the 5-8, hold down the part button for a little longer.

  * `G---` Part 1
  * `-G--` Part 2
  * `--G-` Part 3
  * `---G` Part 4
  * `R---` Part 5
  * `-R--` Part 6
  * `--R-` Part 7
  * `---R` Part 8

## Step input

* Rotary encoder with push button
* BiLED push button

Turning the rotary encoder sets the pitch of the step. The pitch adjusts in the quantization setting.

# TODO

## Global

## Track/part selectors

  * [x] switch part - changes the 16 steps.
  * [x] select - select which track the 16 steps show
  * [x] mute - mute a track stops the gate (and pitch?)
  * [x] copy/paste - copy step to other step, track to track, part to part, (bank to bank?)

### HOLD SHIFT

  * [x] pattern root - root key for the scale
  * [x] pattern scale
  * [x] play/pause
  * [ ] switch bank - switches all 8 parts.
  * [x] play direction (forward, backward, random).
  * [x] swing

  * [x] length - length of the track 1-128 (track spanning multiple parts)
  * [x] "loop mode" - restart track on each SYNC or loop free
  * [x] base velocity - starting point for velocity or lfo offset.
  * [x] velocity/lfo - switch mode between velocity or lfo for the track
  * [x] base probability - base probabilty of each step triggering
  * [x] Slew (glissando)

## Per step

  * [x] step on/off (skip?)
  * [x] note
  * [x] legato
  * [x] length
  * [x] chord mode :)]
  * [x] probability - (additive to step triggering, negative values possible)
  * [x] Slew (glissando)

### HOLD VELOCITY

  * [x] set velocity (and lfo?) - (additive to base velocity, negative values possible)
  * [x] restart lfo (accent?!)
  * [x] micro offset
  * [ ] ratchet

## Chord mode

Press and hold step.

  * [x] step root - root key for the scale
  * [x] step scale
  * [x] step pitch - same as pitch without chord mode
  * [x] spread - how wide the chord is


## Reset mode

Hold shift + velocity

  * [x] reset (bank) - blank everything in the current bank.
  * [x] factory reset - blank all the things.

# Scales

```
[C] D [E] F [G] A [B] C [D] E [F] G [A] B C
root  3rd   5th   7th   9th   11th  13th
```

1. Select root node
2. Select scale
  3. Sinfonion has modes to mean "collection of scales"

Inversions of Cm7

```
               G
          Eb   Eb
     C    C    C
Bb   Bb   Bb   Bb
G    G    G
Eb   Eb
C

root 1st  2nd  3rd
```

Sinfonion has a confusion of inversion + pitch (which becomes octave)
vs "free inversion", where the pitch just gradually climbs up/down.
I think there should be only one of these methods.

Spread

These are the spread configs of Sinfonion

```
1 3 5 7
1 * 5 7 * 3
1 * * 7 * 3 5
1 * 5 * * 3 * 7
1 * * 7 * * 5 * * 3
1 * * * * 3 * 7 * * 5
1 * 5 * * * * * * 3 * 7
1 * * 7 * * * * * * 5 * * 3
1 * * * * 3 * * * * * 7 * * 5
1 * * * * 3 * * * * 5 * * * * 7
```

# Pitch 1v/octave

Analyze Sinfonion.

Octave switch       : -3 0 +3 octaves
Transpose           : -12 0 +12 notes
Pitch + spread goes : 0v - 8.5v octaves roughly. With a 13th + some inversion

```
   Volts  Tone   i8
   -2.53  MIN   ---
   -2.5   F#-1  -30
   -2     C0    -24
   -1     C1    -12
    0     C2      0
    1     C3     12
    2     C4     24
    3     C5     36
    4     C6     48
    5     C7     60
    6     C8     72
    7     C9     84
    8     C10    96
    8.42  F10   101
    8.47  MAX   ---
```

Notice it's 11V total difference.
