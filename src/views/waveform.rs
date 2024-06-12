use std::{collections::VecDeque, time::Duration};
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::ops::Range;

use chrono::{DateTime, Utc};
use iced::{
    Element,
    Length, Size, widget::canvas::{Cache, Frame, Geometry},
};
use iced::advanced::text::editor::Direction;
use plotters::backend::DrawingBackend;
use plotters::chart::ChartBuilder;
use plotters::series::LineSeries;
use plotters::style::ShapeStyle;
use plotters_iced::{Chart, ChartWidget, Renderer};

use crate::hw::{LevelChange, PinLevel};
use crate::Message;
use crate::views::waveform::ChartType::{Logic, Value};

/// `Sample<T>` can be used to send new samples to a waveform widget for display in a moving chart
/// It must have a type `T` that implements `Into<u32>` for Y-axis value, and a `DateTime` when it
/// was measured/detected for the X-axis (or time axis).
#[derive(Clone)]
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
/// - `Logic(false, true)` - which forces square wave display of boolean values
/// - `Value(min, max)` - for display of continuous Y-axis values in a Line Series chart
pub enum ChartType<T>
where
    T: Clone + Into<u32> + PartialEq,
{
    Logic(T, T),
    #[allow(dead_code)]
    Value(T, T),
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
    // TODO write a unit test for this to check it trims correctly
    pub fn push_data(&mut self, sample: Sample<T>) {
        let limit = Utc::now() - self.timespan;
        self.samples.push_front(sample);

        // trim values outside the timespan of the chart
        let mut last_sample = self.samples.back().cloned();
        self.samples.retain(|sample| {
            last_sample = Some(sample.clone()); // TODO do with a reference?
            sample.time > limit
        });

        println!("samples len = {}", self.samples.len());
        if self.samples.len() < 2 {
            if let Some(last) = last_sample {
                println!("Added back last sample: {}", last);
                self.samples.push_back(last);
            }
        }
    }

    /// Refresh and redraw the chart even if there is no new data, as time has passed
    pub fn refresh(&mut self) {
        self.cache.clear();
    }

    // TODO write a unit test for this to cover a number of corner cases
    fn get_data(&self) -> Vec<(DateTime<Utc>, u32)> {
        match &self.chart_type {
            Logic(min, max) => {
                let mut previous_sample: Option<Sample<T>> = None;

                // iterate through the level changes front-back in the vecdeque, which is
                // from the most recent sample to the oldest sample
                // Add points to force the shape to be a Square wave
                let data: Vec<(DateTime<Utc>, u32)> = self
                    .samples
                    .iter()
                    .flat_map(|sample| {
                        if let Some(previous) = &previous_sample {
                            if previous.value == *max && sample.value == *min {
                                // falling edge when going from right to left
                                let vec = vec![
                                    (previous.time, min.clone().into()),
                                    (sample.time, sample.value.clone().into()),
                                ];
                                previous_sample = Some(sample.clone());
                                vec
                            } else if previous.value == *min && sample.value == *max {
                                // rising edge when going from left to right
                                let vec = vec![
                                    (previous.time, max.clone().into()),
                                    (sample.time, sample.value.clone().into()),
                                ];
                                previous_sample = Some(sample.clone());
                                vec
                            } else {
                                // same value
                                vec![]
                            }
                        } else {
                            // last value added, at the start of the dequeue
                            // Insert a value at current time, with the same value as previous one
                            previous_sample = Some(sample.clone());
                            vec![
                                (Utc::now(), sample.value.clone().into()),
                                (sample.time, sample.value.clone().into()),
                            ]
                        }
                    })
                    .collect();

                // TODO add values to cover specific corner cases

                data
            }
            Value(_, _) => self
                .samples
                .iter()
                .map(|sample| (sample.time, sample.value.clone().into()))
                .collect(),
        }
    }

    fn range(&self) -> Range<u32> {
        let (min, max) = match &self.chart_type {
            Logic(min, max) => (
                <T as Into<u32>>::into(min.clone()),
                <T as Into<u32>>::into(max.clone()),
            ),
            Value(min, max) => (
                <T as Into<u32>>::into(min.clone()),
                <T as Into<u32>>::into(max.clone()),
            ),
        };

        min..max
    }

    /// Return an Element that can be used in views to display the chart,
    /// specifying the direction to draw the waveform view in
    pub fn view(&self, direction: Direction) -> Element<Message> {
        self.direction.replace(direction);
        ChartWidget::new(self)
            .height(Length::Fixed(self.height))
            .width(Length::Fixed(self.width))
            .into()
    }
}

impl<T> Chart<Message> for Waveform<T>
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

    use crate::hw::PinLevel;
    use crate::views::waveform::{ChartType, Sample, Waveform};

    const CHART_LINE_STYLE: ShapeStyle = ShapeStyle {
        color: RGBAColor(255, 255, 255, 1.0),
        filled: true,
        stroke_width: 1,
    };

    #[test]
    fn falling_edge() {
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Logic(false, true),
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
            ChartType::Logic(false, true),
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
    fn expired_sample() {
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Logic(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        // create a sample that will be added but then pruned as it's older than the window
        let sent_time = Utc::now().sub(Duration::from_secs(20));
        chart.push_data(Sample {
            time: sent_time,
            value: false,
        });

        // Check the raw data still contains it
        assert_eq!(chart.samples.len(), 1);

        // CHeck the chart data
        let data = chart.get_data();

        // chart data should have added a new point at time of query with the same level
        assert_eq!(data.len(), 2);

        // Next most recent (and first) value should be a "low" inserted at query time
        assert_eq!(data.first().unwrap().1, 0);

        // Next most recent value should be "low" value sent
        assert_eq!(data.get(1).unwrap(), &(sent_time, 0));
    }

    #[test]
    fn expired_then_new_sample() {
        let mut chart = Waveform::<PinLevel>::new(
            ChartType::Logic(false, true),
            CHART_LINE_STYLE,
            256.0,
            16.0,
            Duration::from_secs(10),
        );

        let now = Utc::now();

        // create a sample that will be added but then pruned as it's older than the window
        let low_sent_time = now.sub(Duration::from_secs(20));
        chart.push_data(Sample {
            time: low_sent_time,
            value: false,
        });

        let high_sent_time = now.sub(Duration::from_secs(50));
        chart.push_data(Sample {
            time: high_sent_time,
            value: true,
        });

        // Check the raw data has both
        assert_eq!(chart.samples.len(), 2);

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
    }
}
