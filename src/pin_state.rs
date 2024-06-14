use std::time::Duration;

use iced::advanced::text::editor::Direction;
use iced::Element;
use plotters::prelude::{RGBAColor, ShapeStyle};

use crate::hw::{LevelChange, PinLevel};
use crate::Message;
use crate::views::waveform::{ChartType, Waveform};

pub const CHART_UPDATES_PER_SECOND: u64 = 4;
pub const CHART_WIDTH: f32 = 256.0;
const CHART_HEIGHT: f32 = 30.0;
// If we move 2 pixel per update, that's CHART_WIDTH / 2 updates in the window.
// If we update CHART_UPDATES_PER_SECOND that's 2 * CHART_UPDATES_PER_SECOND pixels per second.
// So CHART_DURATION = CHART_WIDTH / CHART_UPDATES_PER_SECOND * 2(seconds)
const CHART_DURATION: Duration =
    Duration::from_secs(CHART_WIDTH as u64 / (CHART_UPDATES_PER_SECOND * 4));

const CHART_LINE_STYLE: ShapeStyle = ShapeStyle {
    color: RGBAColor(255, 255, 255, 1.0),
    filled: true,
    stroke_width: 1,
};

/// PinState captures the state of a pin, including a history of previous states set/read
pub struct PinState {
    level: Option<PinLevel>,
    pub(crate) chart: Waveform<PinLevel>,
}

impl PinState {
    /// Create a new PinState with an unknown level and a new Waveform chart of it
    pub fn new() -> Self {
        PinState {
            level: None,
            chart: Waveform::new(
                ChartType::Squarewave(false, true),
                CHART_LINE_STYLE,
                CHART_WIDTH,
                CHART_HEIGHT,
                CHART_DURATION,
            ),
        }
    }

    pub fn chart(&self, direction: Direction) -> Element<Message> {
        self.chart.view(direction)
    }

    /// Try and get the last reported level of the pin, which could be considered "current level"
    /// if everything is working correctly.
    pub fn get_level(&self) -> Option<PinLevel> {
        self.level
    }

    /// Add a LevelChange to the history of this pin's state
    pub fn set_level(&mut self, level_change: LevelChange) {
        self.level = Some(level_change.new_level);
        self.chart.push_data(level_change)
    }
}

#[cfg(test)]
mod test {
    use crate::hw::LevelChange;
    use crate::pin_state::PinState;

    #[test]
    fn level_stores_last() {
        let mut state = PinState::new();
        state.set_level(LevelChange::new(false));
        state.set_level(LevelChange::new(true));
        state.set_level(LevelChange::new(false));
        state.set_level(LevelChange::new(true));
        assert_eq!(state.get_level(), Some(true));
    }
}
