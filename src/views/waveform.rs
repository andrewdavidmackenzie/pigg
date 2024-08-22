use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::{collections::VecDeque, time::Duration};

use chrono::{DateTime, Utc};
use iced::advanced::text::editor::Direction;
use iced::{
    widget::canvas::{Cache, Frame, Geometry},
    Element, Length, Size,
};
use plotters::backend::DrawingBackend;
use plotters::chart::ChartBuilder;
use plotters::series::LineSeries;
use plotters::style::ShapeStyle;
use plotters_iced::{Chart, ChartWidget, Renderer};

use crate::hw_definition::{config_message::LevelChange, PinLevel};
use crate::views::hardware_view::HardwareViewMessage;
use crate::views::waveform::ChartType::{Squarewave, Verbatim};

/// `Sample<T>` can be used to send new samples to a waveform widget for display in a moving chart
/// It must have a type `T` that implements `Into<u32>` for Y-axis value, and a `DateTime` when it
/// was measured/detected for the X-axis (or time axis).
#[derive(Clone, PartialEq, Debug)]
pub struct Sample<T>
where
    T: Clone + Into<u32> + PartialEq + Display,
{
    pub time: DateTime<Utc>,
    pub value: T,
}

impl<T> Display for Sample<T>
where
    T: Clone + Into<u32> + PartialEq + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.time, self.value)
    }
}

impl From<LevelChange> for Sample<PinLevel> {
    fn from(level_change: LevelChange) -> Self {
        Self {
            time: level_change.timestamp,
            value: level_change.new_level,
        }
    }
}

/// Two types of charts can be drawn:
/// - `Squarewave(min, max)` - which forces square wave display of values
/// - `Verbatim(min, max)` - for display of continuous Y-axis values in a Line Series chart
pub enum ChartType<T>
where
    T: Clone + Into<u32> + PartialEq,
{
    Squarewave(T, T),
    #[allow(dead_code)]
    Verbatim(T, T),
}

/// A Waveform chart - used to display the changes of a value over time
pub struct Waveform<T>
where
    T: Clone + Into<u32> + PartialEq + Display,
{
    chart_type: ChartType<T>,
    style: ShapeStyle,
    width: f32,
    height: f32,
    direction: RefCell<Direction>,
    cache: Cache,
    timespan: Duration,
    samples: VecDeque<Sample<T>>,
}

impl<T> Waveform<T>
where
    T: Clone + Into<u32> + PartialEq + Display,
{
    /// Create a new `Waveform` chart for display with parameters:
    /// - `chart_type` : The type of chart to draw. See [ChartType]
    /// - `line_style` : The Style to be applied to the line. See [ShapeStyle]
    /// - `width` : The width of the chart in pixels
    /// - `height` : The height of the chart in pixels
    /// - `timespan` : The period of time the chart should cover
    /// - `direction` : If chart should be drawn moving left, or moving right
    pub fn new(
        chart_type: ChartType<T>,
        line_style: ShapeStyle,
        width: f32,
        height: f32,
        timespan: Duration,
    ) -> Self {
        Self {
            chart_type,
            style: line_style,
            width,
            height,
            direction: RefCell::new(Direction::Right),
            cache: Cache::new(),
            samples: VecDeque::new(),
            timespan,
        }
    }

    /// Add a new [Sample] to the data set to be displayed in the chart
    pub fn push_data(&mut self, sample: impl Into<Sample<T>>) {
        self.samples.push_front(sample.into());
        self.trim_data();
    }

    /// Trim samples outside the timespan of the chart, except the most recent one
    fn trim_data(&mut self) {
        if !self.samples.is_empty() {
            let limit = Utc::now() - self.timespan;
            let mut last_out_of_window_sample = None;
            self.samples.retain(|sample| {
                let retain = sample.time > limit;
                if !retain && last_out_of_window_sample.is_none() {
                    last_out_of_window_sample = Some(sample.clone());
                }
                retain
            });

            if let Some(last) = last_out_of_window_sample {
                self.samples.push_back(last);
            }
        }
    }

    /// Refresh and redraw the chart even if there is no new data, as time has passed
    pub fn refresh(&mut self) {
        self.trim_data();
        self.cache.clear();
    }

    fn get_data(&self) -> Vec<(DateTime<Utc>, u32)> {
        match &self.chart_type {
            Squarewave(_, _) => {
                let mut previous_sample: Option<Sample<T>> = None;
                let mut graph_data = vec![];

                // iterate through the Samples front-back in the vecdeque, which is
                // from the most recent sample to the oldest sample
                // Add points to force the shape to be a Square wave
                for sample in &self.samples {
                    if let Some(previous) = &previous_sample {
                        if previous.value != sample.value {
                            // edge - insert a point at previous time at current level
                            graph_data.push((previous.time, sample.value.clone().into()));
                        }
                    } else {
                        // last value added, at the start of the dequeue
                        // Insert a value at current time, with the same value as previous one
                        graph_data.push((Utc::now(), sample.value.clone().into()));
                    }
                    graph_data.push((sample.time, sample.value.clone().into()));
                    previous_sample = Some(sample.clone());
                }
                graph_data
            }
            Verbatim(_, _) => self
                .samples
                .iter()
                .map(|sample| (sample.time, sample.value.clone().into()))
                .collect(),
        }
    }

    fn range(&self) -> Range<u32> {
        let (min, max) = match &self.chart_type {
            Squarewave(min, max) => (
                <T as Into<u32>>::into(min.clone()),
                <T as Into<u32>>::into(max.clone()),
            ),
            Verbatim(min, max) => (
                <T as Into<u32>>::into(min.clone()),
                <T as Into<u32>>::into(max.clone()),
            ),
        };

        min..max
    }

    /// Return an Element that can be used in views to display the chart,
    /// specifying the direction to draw the waveform view in
    pub fn view(&self, direction: Direction) -> Element<HardwareViewMessage> {
        self.direction.replace(direction);
        ChartWidget::new(self)
            .height(Length::Fixed(self.height))
            .width(Length::Fixed(self.width))
            .into()
    }
}

impl<T> Chart<HardwareViewMessage> for Waveform<T>
where
    T: Clone + Into<u32> + PartialEq + Display,
{
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut chart: ChartBuilder<DB>) {
        if !self.samples.is_empty() {
            let last_time = Utc::now();
            let start_of_chart_time =
                last_time - chrono::Duration::seconds(self.timespan.as_secs() as i64);
            let time_axis = match *self.direction.borrow() {
                Direction::Left => start_of_chart_time..last_time,
                Direction::Right => last_time..start_of_chart_time,
            };
            let mut chart = chart
                .build_cartesian_2d(time_axis, self.range())
                .expect("failed to build chart");
            let _ = chart.draw_series(LineSeries::new(self.get_data(), self.style));
        }
    }

    #[inline]
    fn draw<R: Renderer, F: Fn(&mut Frame)>(
        &self,
        renderer: &R,
        bounds: Size,
        draw_fn: F,
    ) -> Geometry {
        renderer.draw_cache(&self.cache, bounds, draw_fn)
    }
}

#[cfg(test)]
mod test {
    use std::ops::Sub;
    use std::time::Duration;

    use chrono::Utc;
    use plotters::prelude::{RGBAColor, ShapeStyle};

    use crate::hw_definition::{config_message::LevelChange, PinLevel};
    use crate::views::waveform::{ChartType, Sample, Waveform};

    const CHART_LINE_STYLE: ShapeStyle = ShapeStyle {
        color: RGBAColor(255, 255, 255, 1.0),
        filled: true,
        stroke_width: 1,
    };

    #[test]
    fn falling_edge() {
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Squarewave(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        let high_sent_time = Utc::now().sub(Duration::from_secs(2));
        chart.push_data(Sample {
            time: high_sent_time,
            value: true,
        });

        let low_sent_time = Utc::now().sub(Duration::from_secs(1));
        chart.push_data(Sample {
            time: low_sent_time,
            value: false,
        });

        let data = chart.get_data();
        assert_eq!(data.len(), 4);

        // Next most recent (and first) value should be the "low" inserted at query time
        assert_eq!(data.first().unwrap().1, 0);

        // Next most recent value should be "low" value sent
        assert_eq!(data.get(1).unwrap(), &(low_sent_time, 0));

        // Next most recent value should be the "high" inserted when "low" was sent
        assert_eq!(data.get(2).unwrap(), &(low_sent_time, 1));

        // Next most recent value should be the "high" sent initially
        assert_eq!(data.get(3).unwrap(), &(high_sent_time, 1));
    }

    #[test]
    fn rising_edge() {
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Squarewave(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        let low_sent_time = Utc::now().sub(Duration::from_secs(2));
        chart.push_data(Sample {
            time: low_sent_time,
            value: false,
        });

        let high_sent_time = Utc::now().sub(Duration::from_secs(1));
        chart.push_data(Sample {
            time: high_sent_time,
            value: true,
        });

        let data = chart.get_data();
        assert_eq!(data.len(), 4);

        // Next most recent (and first) value should be the "high" inserted at query time
        assert_eq!(data.first().unwrap().1, 1);

        // Next most recent value should be "high" value sent
        assert_eq!(data.get(1).unwrap(), &(high_sent_time, 1));

        // Next most recent value should be the "low" inserted when "high" was sent
        assert_eq!(data.get(2).unwrap(), &(high_sent_time, 0));

        // Next most recent value should be the "low" sent initially
        assert_eq!(data.get(3).unwrap(), &(low_sent_time, 0));
    }

    #[test]
    fn sample_outside_window_preserved() {
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Squarewave(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        // create a sample older than the window
        let sent_time = Utc::now().sub(Duration::from_secs(20));
        chart.push_data(Sample {
            time: sent_time,
            value: false,
        });

        // Check the raw data still contains it
        assert_eq!(chart.samples.len(), 1);

        // Check the chart data
        let data = chart.get_data();

        // chart data should have added a new point at time of query with the same level
        assert_eq!(data.len(), 2);

        // Next most recent (and first) value should be a "low" inserted at query time
        assert_eq!(data.first().unwrap().1, 0);

        // Next most recent value should be "low" value sent
        assert_eq!(data.get(1).unwrap(), &(sent_time, 0));
    }

    #[test]
    fn extra_samples_outside_window_deleted() {
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Squarewave(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        // create a sample older than the window
        let oldest = Utc::now().sub(Duration::from_secs(20));
        chart.push_data(Sample {
            time: oldest,
            value: true,
        });

        let next_oldest = Utc::now().sub(Duration::from_secs(15));
        chart.push_data(Sample {
            time: next_oldest,
            value: false,
        });

        // Check the raw data does not contain the oldest
        assert_eq!(chart.samples.len(), 1);

        // Check the chart data
        let data = chart.get_data();

        // chart data should have added a new point at time of query with the same level
        assert_eq!(data.len(), 2);

        // Next most recent (and first) value should be a "low" inserted at query time
        assert_eq!(data.first().unwrap().1, 0);

        // Next most recent value should be "low" value sent
        assert_eq!(data.get(1).unwrap(), &(next_oldest, 0));
    }

    #[test]
    fn expired_then_new_sample() {
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Squarewave(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        let now = Utc::now();

        // create a sample that will be added but then pruned as it's older than the window
        let low_sent_time = now.sub(Duration::from_secs(20));
        let low_sample = LevelChange {
            new_level: false,
            timestamp: low_sent_time,
        };
        chart.push_data(low_sample.clone());
        assert_eq!(chart.samples.len(), 1);

        // Send a sample in the middle of the time window
        let high_sent_time = now.sub(Duration::from_secs(5));
        let high_sample = Sample {
            time: high_sent_time,
            value: true,
        };
        chart.push_data(high_sample.clone());

        // Check the raw data has both
        assert_eq!(chart.samples.len(), 2);
        assert_eq!(chart.samples.front().unwrap(), &high_sample);
        assert_eq!(chart.samples.get(1).unwrap(), &low_sample.into());

        // Get the chart data
        let data = chart.get_data();

        // chart data should have:
        // - a new point at time of query with the same level
        // - the high we sent
        // - a low added at the same time as high sent, to create an edge
        // - original low that is out of window
        assert_eq!(data.len(), 4);

        // Next most recent (and first) value should be a "low" inserted at query time
        assert_eq!(data.first().unwrap().1, 1);

        // Next most recent value should be "high" value sent
        assert_eq!(data.get(1).unwrap(), &(high_sent_time, 1));

        // Next most recent value should be "low" added with same time as "high" sent
        assert_eq!(data.get(2).unwrap(), &(high_sent_time, 0));

        // Next most recent value should be "low" that is outside window but kept
        assert_eq!(data.get(3).unwrap(), &(low_sent_time, 0));
    }

    #[test]
    fn check_range_verbatim() {
        let chart = Waveform::<u32>::new(
            ChartType::Verbatim(0, 100),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        assert_eq!(chart.range(), 0..100);
    }

    #[test]
    fn display_sample() {
        let level_change = LevelChange {
            new_level: false,
            timestamp: Utc::now(),
        };

        let sample: Sample<PinLevel> = level_change.into();

        let display = format!("{sample}");
        assert!(display.contains("UTC"));
        assert!(display.contains("false"));
    }

    // Samples very close together or at the same time - what is the finest resolution
    #[test]
    fn resolution() {}

    #[test]
    fn no_sample_empty_graph() {
        let chart = Waveform::<PinLevel>::new(
            ChartType::Squarewave(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        assert!(chart.samples.is_empty());

        // Get the chart data
        let data = chart.get_data();

        assert!(data.is_empty());
    }

    #[test]
    fn rising_edge_from_old_sample() {
        //  |                   |
        //  +--------o          |
        //  |        |          |
        //  |        +----------|---o
        //  |                   |
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Squarewave(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        let now = Utc::now();

        // Create an old low sample that is out of the display window
        let old_sample = LevelChange {
            new_level: false,
            timestamp: now.sub(Duration::from_secs(20)),
        };
        chart.push_data(old_sample.clone());

        // create a new high sample that is in the window
        let new_sample = LevelChange {
            new_level: true,
            timestamp: now.sub(Duration::from_secs(2)),
        };
        chart.push_data(new_sample.clone());

        // Check the raw data has all
        assert_eq!(chart.samples.len(), 2);

        // Get the chart data
        let data = chart.get_data();

        // graph should need 4 points to represent the edge
        assert_eq!(data.len(), 4);
        assert_eq!(data.first().unwrap().1, 1); // added at query time
        assert_eq!(data.get(1).unwrap().1, 1); // top of rising edge
        assert_eq!(data.get(2).unwrap().1, 0); // bottom of rising edge
        assert_eq!(data.get(3).unwrap().1, 0); // old low sample
    }

    #[test]
    fn falling_edge_from_old_sample() {
        //  |                   |
        //  |        +----------|---o
        //  |        |          |
        //  +--------o          |
        //  |                   |
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Squarewave(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        let now = Utc::now();

        // Create an old high sample that is out of the display window
        let old_sample = LevelChange {
            new_level: true,
            timestamp: now.sub(Duration::from_secs(20)),
        };
        chart.push_data(old_sample.clone());

        // create a new low sample that is in the window
        let new_sample = LevelChange {
            new_level: false,
            timestamp: now.sub(Duration::from_secs(2)),
        };
        chart.push_data(new_sample.clone());

        // Check the raw data has all
        assert_eq!(chart.samples.len(), 2);

        // Get the chart data
        let data = chart.get_data();

        // graph should need 4 points to represent the edge
        assert_eq!(data.len(), 4);
        assert_eq!(data.first().unwrap().1, 0); // added at query time
        assert_eq!(data.get(1).unwrap().1, 0); // bottom of rising edge
        assert_eq!(data.get(2).unwrap().1, 1); // top of rising edge
        assert_eq!(data.get(3).unwrap().1, 1); // old high sample
    }

    #[test]
    fn pulse_up_and_down_from_low_base() {
        //  |                   |
        //  |    +---o          |
        //  |    |   |          |
        //  +----o   +-------o--|
        //  |                   |
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Squarewave(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        let now = Utc::now();

        // Create a low sample that is in the display window
        let old_sample = LevelChange {
            new_level: false,
            timestamp: now.sub(Duration::from_secs(9)),
        };
        chart.push_data(old_sample.clone());

        // create a pulse, up and down
        let new_sample = LevelChange {
            new_level: true,
            timestamp: now.sub(Duration::from_secs(5)),
        };
        chart.push_data(new_sample.clone());
        let new_sample = LevelChange {
            new_level: false,
            timestamp: now.sub(Duration::from_secs(4)),
        };
        chart.push_data(new_sample.clone());

        // Check the raw data has all
        assert_eq!(chart.samples.len(), 3);

        // Get the chart data
        let data = chart.get_data();

        // graph should need 4 points to represent the edge
        assert_eq!(data.len(), 6);
        assert_eq!(data.first().unwrap().1, 0);
        assert_eq!(data.get(1).unwrap().1, 0); // bottom left of pulse
        assert_eq!(data.get(2).unwrap().1, 1); // top left of pulse
        assert_eq!(data.get(3).unwrap().1, 1); // top right of pulse
        assert_eq!(data.get(4).unwrap().1, 0); // bottom right of pulse
        assert_eq!(data.get(5).unwrap().1, 0); // old low sample
    }

    #[test]
    fn pulse_down_and_up_from_high_base() {
        //  |                   |
        //  +----o   +-------o--|
        //  |    |   |          |
        //  |    +---o          |
        //  |                   |
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Squarewave(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        let now = Utc::now();

        // Create a high sample that is in the display window
        let old_sample = LevelChange {
            new_level: true,
            timestamp: now.sub(Duration::from_secs(9)),
        };
        chart.push_data(old_sample.clone());

        // create a pulse, down then back up
        let new_sample = LevelChange {
            new_level: false,
            timestamp: now.sub(Duration::from_secs(5)),
        };
        chart.push_data(new_sample.clone());
        let new_sample = LevelChange {
            new_level: true,
            timestamp: now.sub(Duration::from_secs(4)),
        };
        chart.push_data(new_sample.clone());

        // Check the raw data has all
        assert_eq!(chart.samples.len(), 3);

        // Get the chart data
        let data = chart.get_data();

        // graph should need 4 points to represent the edge
        assert_eq!(data.len(), 6);
        assert_eq!(data.first().unwrap().1, 1); // added at query time
        assert_eq!(data.get(1).unwrap().1, 1); // top left of pulse
        assert_eq!(data.get(2).unwrap().1, 0); // bottom left of pulse
        assert_eq!(data.get(3).unwrap().1, 0); // bottom right of pulse
        assert_eq!(data.get(4).unwrap().1, 1); // rop right of pulse
        assert_eq!(data.get(5).unwrap().1, 1); // old low sample
    }
}
