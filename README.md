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

  * [x] root key
  * quantize setting
  * start/pause
  * switch play mode between immediate/next 16th
  * copy/paste
  * full reset

## Per track

  * length
  * restart with SYNC or loop free
  * velocity/lfo
  * select lfo

## Per step

  * set velocity
  * ratchet
  * repeat previous (or selected?)


