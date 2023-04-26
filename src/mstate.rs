use crate::buttons::Buttons;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum MachineState {
    /// No qualifier button is pressed.
    #[default]
    Normal,

    /// Shift button is down
    Shift,

    /// Velocity button is down
    Velocity,

    /// A rotary button on the upper row is pressed.
    ///
    /// The u32 tells us which rotary button is pressed.
    HoldUpper(u32),

    /// A rotary button on the lower row is pressed.
    ///
    /// The u32 tells us which one.
    HoldLower(u32),

    /// Both shift and velocity is pressed.
    Reset,

    /// Wait for buttons to clear so we can go back to normal.
    WaitForClear,
}

impl MachineState {
    pub fn transition(&mut self, buttons: &Buttons) {
        match self {
            MachineState::Normal => match (
                buttons.is_shift(),
                buttons.is_vel(),
                buttons.is_rotary_upper(),
                buttons.is_rotary_lower(),
            ) {
                (true, true, None, None) => *self = MachineState::Reset,
                (true, false, None, None) => *self = MachineState::Shift,
                (false, true, None, None) => *self = MachineState::Velocity,
                (false, false, Some(b), None) => *self = MachineState::HoldUpper(b),
                (false, false, None, Some(b)) => *self = MachineState::HoldLower(b),
                _ => {}
            },

            MachineState::Shift => match (buttons.is_shift(), buttons.is_vel()) {
                (true, false) => {} // stay in state
                (true, true) => *self = MachineState::Reset,
                _ => *self = MachineState::WaitForClear,
            },

            MachineState::Velocity => match (buttons.is_shift(), buttons.is_vel()) {
                (false, true) => {} //stay in state
                (true, true) => *self = MachineState::Reset,
                _ => *self = MachineState::WaitForClear,
            },

            MachineState::HoldUpper(b) => match buttons.is_rotary_upper() {
                Some(x) if *b == x => {} // stay in state
                _ => *self = MachineState::WaitForClear,
            },

            MachineState::HoldLower(b) => match buttons.is_rotary_lower() {
                Some(x) if *b == x => {} // stay in state
                _ => *self = MachineState::WaitForClear,
            },

            MachineState::Reset => match (buttons.is_shift(), buttons.is_vel()) {
                (true, true) => {} // stay in state
                (true, false) => *self = MachineState::Shift,
                (false, true) => *self = MachineState::Velocity,
                (false, false) => *self = MachineState::WaitForClear,
            },

            MachineState::WaitForClear => {
                if buttons.is_clear() {
                    *self = MachineState::Normal;
                }
            }
        }
    }
}
