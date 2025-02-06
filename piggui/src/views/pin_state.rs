use chrono::DateTime;
use std::time::Duration;

use iced::advanced::text::editor::Direction;
use iced::Element;
use plotters::prelude::{RGBAColor, ShapeStyle};

use crate::hw_definition::{config::LevelChange, PinLevel};
use crate::views::hardware_view::HardwareViewMessage;
use crate::views::waveform::{ChartType, Sample, Waveform};

pub const CHART_UPDATES_PER_SECOND: u64 = 4;
pub const CHART_WIDTH: f32 = 256.0;
const CHART_HEIGHT: f32 = 28.0;
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

/// PinState captures the logical level of a pin, including a history of previous states
pub struct PinState {
    // Cache the level of the last recorded level_change as the current level
    current_level: Option<PinLevel>,
    pub(crate) chart: Waveform<PinLevel>,
}

impl From<LevelChange> for Sample<PinLevel> {
    fn from(level_change: LevelChange) -> Self {
        let time = DateTime::from_timestamp(
            level_change.timestamp.as_secs() as i64,
            level_change.timestamp.subsec_nanos(),
        )
        .unwrap();
        Self {
            time,
            value: level_change.new_level,
        }
    }
}

impl PinState {
    /// Create a new PinState with an unknown level and a new Waveform chart of it
    pub fn new() -> Self {
        PinState {
            current_level: None,
            chart: Waveform::new(
                ChartType::Squarewave(false, true),
                CHART_LINE_STYLE,
                CHART_WIDTH,
                CHART_HEIGHT,
                CHART_DURATION,
            ),
        }
    }

    pub fn view(&self, direction: Direction) -> Element<HardwareViewMessage> {
        self.chart.view(direction)
    }

    /// Try and get the last reported level of the pin, which could be considered "current level"
    /// if everything is working correctly.
    pub fn get_level(&self) -> Option<PinLevel> {
        self.current_level
    }

    /// Add a LevelChange to the history of this pin's state
    pub fn set_level(&mut self, level_change: LevelChange) {
        self.current_level = Some(level_change.new_level);

        let dt = self.chart.date_time(level_change.timestamp);
        let mut sample: Sample<PinLevel> = level_change.into();
        sample.time = dt;

        self.chart.push_data(sample)
    }
}

#[cfg(test)]
mod test {
    use crate::hw_definition::config::LevelChange;
    use crate::views::pin_state::PinState;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn level_stores_last() {
        let mut state = PinState::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        state.set_level(LevelChange::new(false, now));
        state.set_level(LevelChange::new(true, now));
        state.set_level(LevelChange::new(false, now));
        state.set_level(LevelChange::new(true, now));
        assert_eq!(state.get_level(), Some(true));
    }
}
