use alg::input::Edge;

use crate::app_input::AppInputUpdate;
use crate::led_grid::BiLed;
use crate::CLOCK;

pub struct AppState {
    leds: [[BiLed; 8]; 5],
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            leds: [[BiLed::Off; 8]; 5],
        }
    }

    pub fn led_row(&self, row: usize) -> &[BiLed; 8] {
        &self.leds[row]
    }
}

impl AppInputUpdate<{ CLOCK }> for AppState {
    fn update_input(
        &mut self,
        col: usize,
        clk: Option<Edge<{ CLOCK }>>,
        rst: Option<Edge<{ CLOCK }>>,
        rot_row1: i8,
        rot_row2: i8,
        swr_row1: Option<Edge<{ CLOCK }>>,
        swr_row2: Option<Edge<{ CLOCK }>>,
        swl_row1: Option<Edge<{ CLOCK }>>,
        swl_row2: Option<Edge<{ CLOCK }>>,
        swl_row3: Option<Edge<{ CLOCK }>>,
        swl_row4: Option<Edge<{ CLOCK }>>,
        swl_row5: Option<Edge<{ CLOCK }>>,
    ) {
        //todo!()
    }
}
