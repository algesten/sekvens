use crate::Time;

#[derive(Default)]
pub struct Button {
    /// When the button is pushed down.
    on: Option<Time>,
    last_state: ButtonState,
}

impl Button {
    pub fn set_on(&mut self, on: bool, now: Time) {
        if on {
            self.on = Some(now);
        } else {
            self.on = None;
        }
    }

    pub fn tick(&mut self, now: Time) -> bool {
        let state = self.state(now);
        if self.last_state != state {
            self.last_state = state;
            true
        } else {
            false
        }
    }

    fn state(&self, now: Time) -> ButtonState {
        let Some(on) = &self.on  else {
            return ButtonState::Off;
        };

        if now - *on > Time::from_millis(200) {
            ButtonState::LongPressed
        } else {
            ButtonState::Pressed
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.last_state != ButtonState::Off
    }

    pub fn is_long_pressed(&self) -> bool {
        self.last_state == ButtonState::LongPressed
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum ButtonState {
    #[default]
    Off,
    Pressed,
    LongPressed,
}
