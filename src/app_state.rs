use alg::input::Edge;

use crate::app_input::AppInputUpdate;
use crate::led_grid::BiLed;
use crate::CPU_SPEED;

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

impl AppInputUpdate<{ CPU_SPEED }> for AppState {
    fn update_input(
        &mut self,
        col: usize,
        clk: Option<Edge<{ CPU_SPEED }>>,
        rst: Option<Edge<{ CPU_SPEED }>>,
        rot_row1: i8,
        rot_row2: i8,
        swr_row1: Option<Edge<{ CPU_SPEED }>>,
        swr_row2: Option<Edge<{ CPU_SPEED }>>,
        swl_row1: Option<Edge<{ CPU_SPEED }>>,
        swl_row2: Option<Edge<{ CPU_SPEED }>>,
        swl_row3: Option<Edge<{ CPU_SPEED }>>,
        swl_row4: Option<Edge<{ CPU_SPEED }>>,
        swl_row5: Option<Edge<{ CPU_SPEED }>>,
    ) {
        todo!()
    }
}
