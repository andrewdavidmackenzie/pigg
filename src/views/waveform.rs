use std::{collections::VecDeque, time::Duration};
use std::cell::RefCell;
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
    T: Clone + Into<u32> + PartialEq,
{
    pub time: DateTime<Utc>,
    pub value: T,
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
    T: Clone + Into<u32> + PartialEq,
{
    chart_type: ChartType<T>,
    style: ShapeStyle,
    width: f32,
    height: f32,
    direction: RefCell<Direction>,
    cache: Cache,
    timespan: Duration,
    data_points: VecDeque<Sample<T>>,
}

impl<T> Waveform<T>
where
    T: Clone + Into<u32> + PartialEq,
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
            data_points: VecDeque::new(),
            timespan,
        }
    }

    /// Add a new [Sample] to the data set to be displayed in the chart
    pub fn push_data(&mut self, sample: Sample<T>) {
        let limit = sample.time - self.timespan;
        self.data_points.push_front(sample);

        // trim old values based on time
        loop {
            if let Some(old_sample) = self.data_points.back() {
                if old_sample.time < limit {
                    self.data_points.pop_back();
                    continue;
                }
            }
            break;
        }
        self.cache.clear();
    }

    fn get_data(&self) -> Vec<(DateTime<Utc>, u32)> {
        match &self.chart_type {
            Value(_, _) => self
                .data_points
                .iter()
                .map(|sample| (sample.time, sample.value.clone().into()))
                .collect(),

            Logic(min, max) => {
                let mut previous_value = None;

                self.data_points
                    .iter()
                    .flat_map(|sample| {
                        if let Some(previous) = &previous_value {
                            if previous == max && sample.value == *min {
                                // falling edge
                                previous_value = Some(sample.value.clone());
                                vec![
                                    (sample.time, max.clone().into()),
                                    (sample.time, sample.value.clone().into()),
                                ]
                            } else if previous == min && sample.value == *max {
                                // rising edge
                                previous_value = Some(sample.value.clone());
                                vec![
                                    (sample.time, min.clone().into()),
                                    (sample.time, sample.value.clone().into()),
                                ]
                            } else {
                                // same value
                                vec![(sample.time, sample.value.clone().into())]
                            }
                        } else {
                            // First value
                            previous_value = Some(sample.value.clone());
                            vec![(sample.time, sample.value.clone().into())]
                        }
                    })
                    .collect()
            }
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
    T: Clone + Into<u32> + PartialEq,
{
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut chart: ChartBuilder<DB>) {
        if !self.data_points.is_empty() {
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
