use iced::{
    alignment::{Horizontal, Vertical},
    executor,
    widget::{
        canvas::{Cache, Frame, Geometry},
        Column, Container, Row, Scrollable, Space, Text,
    },
    Alignment, Application, Command, Element, Font, Length, Settings, Size, Subscription, Theme,
};

use plotters::prelude::ChartBuilder;
use plotters_iced::{Chart, ChartWidget};
use plotters_backend::DrawingBackend;
use std::collections::VecDeque;

const MAX_RANGE: i32 = 100;

const FONT_REGULAR: Font = Font::External {
    name: "sans-serif-regular",
    bytes: include_bytes!("../assets/SourceHanSansJP-Regular.otf"),
};

const FONT_BOLD: Font = Font::External {
    name: "sans-serif-bold",
    bytes: include_bytes!("../assets/SourceHanSansJP-Regular.otf"),
};


#[derive(Debug)]
pub enum Message {
    /// message that cause charts' data lazily updated
    Tick,
}

pub struct WaveChart {
    cache: Cache,
    data_points: VecDeque<(i32, f64)>, // index, value
    limit: i32, // plot数
}

impl WaveChart {
    pub fn new(data: impl Iterator<Item = (i32, f64)>) -> Self {
        let data_points: VecDeque<_> = data.collect();
        Self {
            cache: Cache::new(),
            data_points,
            limit: MAX_RANGE,
        }
    }

    pub fn push_data(&mut self, time: i32, value: f64) {
        self.data_points.push_front((time, value));
        loop {
            if let Some((time, _)) = self.data_points.back() {
                if (self.limit as usize) < self.data_points.len() {
                    self.data_points.pop_back();
                    continue;
                }
            }
            break;
        }
        self.cache.clear();
    }

    pub fn view(&self, idx: usize) -> Element<Message> {
        Container::new(
            Column::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(5)
                .push(Text::new(format!("sin waveだよ")))
                .push(
                    ChartWidget::new(self).height(Length::Fill).resolve_font(
                        |_, style| match style {
                            plotters_backend::FontStyle::Bold => FONT_BOLD,
                            _ => FONT_REGULAR,
                        },
                    ),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into()
    }
}


impl Chart<Message> for WaveChart {
    type State = ();
    // fn update(
    //     &mut self,
    //     event: Event,
    //     bounds: Rectangle,
    //     cursor: Cursor,
    // ) -> (event::Status, Option<Message>) {
    //     self.cache.clear();
    //     (event::Status::Ignored, None)
    // }

    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        self.cache.draw(bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut chart: ChartBuilder<DB>) {
        use plotters::{prelude::*, style::Color};

        const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);

        // Acquire time range
        let mut chart = chart
            .x_label_area_size(0)
            .y_label_area_size(28)
            .margin(20)
            .build_cartesian_2d(self.data_points[0].0..(self.data_points[0].0-100), -100..100)
            .expect("failed to build chart");

        chart
            .configure_mesh()
            .bold_line_style(plotters::style::colors::BLUE.mix(0.1))
            .light_line_style(plotters::style::colors::BLUE.mix(0.05))
            .axis_style(ShapeStyle::from(plotters::style::colors::BLUE.mix(0.45)).stroke_width(1))
            .y_labels(10)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&plotters::style::colors::BLUE.mix(0.65))
                    .transform(FontTransform::Rotate90),
            )
            .draw()
            .expect("failed to draw chart mesh");

        chart
            .draw_series(
                AreaSeries::new(
                    self.data_points.iter().map(|x| (x.0, x.1 as i32)),
                    0,
                    PLOT_LINE_COLOR.mix(0.175),
                )
                .border_style(ShapeStyle::from(PLOT_LINE_COLOR).stroke_width(2)),
            )
            .expect("failed to draw chart data");
    }
}
