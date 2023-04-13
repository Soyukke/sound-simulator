use sound_simulator::*;
use std::f64::consts::PI;

use chrono::{DateTime, Utc};
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
use plotters_backend::DrawingBackend;
use plotters_iced::{Chart, ChartWidget};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const PLOT_SECONDS: usize = 60; //1 min
const TITLE_FONT_SIZE: u16 = 22;
const SAMPLE_EVERY: Duration = Duration::from_millis(10);

const FONT_REGULAR: Font = Font::External {
    name: "sans-serif-regular",
    bytes: include_bytes!("../assets/SourceHanSansJP-Regular.otf"),
};

const FONT_BOLD: Font = Font::External {
    name: "sans-serif-bold",
    bytes: include_bytes!("../assets/SourceHanSansJP-Regular.otf"),
};

fn main() {
    State::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("../assets/SourceHanSansJP-Regular.otf")),
        ..Settings::default()
    })
    .unwrap();
}

struct State {
    chart: SystemChart,
}

impl Application for State {
    type Message = self::Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                chart: Default::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "CPU Monitor Example".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick => {
                self.chart.update();
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.chart.view());

        Container::new(content)
            //.style(style::Container)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        const FPS: u64 = 120;
        iced::time::every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
    }
}

struct SystemChart {
    last_sample_time: Instant,
    items_per_row: usize,
    processors: Vec<WaveChart>,
    chart_height: f32,
    count: i32,
}

impl Default for SystemChart {
    fn default() -> Self {
        Self {
            last_sample_time: Instant::now(),
            items_per_row: 3,
            processors: Default::default(),
            chart_height: 300.0,
            count: 0,
        }
    }
}

impl SystemChart {
    #[inline]
    fn is_initialized(&self) -> bool {
        !self.processors.is_empty()
    }

    #[inline]
    fn should_update(&self) -> bool {
        !self.is_initialized() || self.last_sample_time.elapsed() > SAMPLE_EVERY
    }

    fn update(&mut self) {
        if !self.should_update() {
            return;
        }
        self.last_sample_time = Instant::now();
        self.count+=1;
        // ここで各cpuの使用率を取得している
        //let data = self.sys.cpus().iter().map(|v| v.cpu_usage() as i32);
        let xs: Vec<f64> = (0..1).map(
            |z| f64::from(z+self.count) * 2.0*PI/100.0
        ).collect();
        let data = xs.iter().map(|&x| 100. * f64::sin(x));

        //check if initialized
        if !self.is_initialized() {
            let w_test = WaveChart::new(vec![(self.count, 0.0)].into_iter());
            w_test.export();
            eprintln!("init...");
            let mut processors: Vec<_> = data
                .map(|percent| WaveChart::new(vec![(self.count, percent)].into_iter()))
                .collect();
            self.processors.append(&mut processors);
        } else {
            //eprintln!("update...");
            for (percent, p) in data.zip(self.processors.iter_mut()) {
                p.push_data(self.count, percent);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if !self.is_initialized() {
            Text::new("Loading...")
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center)
                .into()
        } else {
            let mut col = Column::new().width(Length::Fill).height(Length::Shrink);

            let chart_height = self.chart_height;
            let mut idx = 0;
            for chunk in self.processors.chunks(self.items_per_row) {
                let mut row = Row::new()
                    .spacing(15)
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Fixed(chart_height))
                    .align_items(Alignment::Center);
                for item in chunk {
                    row = row.push(item.view(idx));
                    idx += 1;
                }
                while idx % self.items_per_row != 0 {
                    row = row.push(Space::new(Length::Fill, Length::Fill));
                    idx += 1;
                }
                col = col.push(row);
            }

            Scrollable::new(col).height(Length::Fill).into()
        }
    }
}

struct CpuUsageChart {
    cache: Cache,
    data_points: VecDeque<(DateTime<Utc>, i32)>,
    limit: Duration,
}

impl CpuUsageChart {
    fn new(data: impl Iterator<Item = (DateTime<Utc>, i32)>) -> Self {
        let data_points: VecDeque<_> = data.collect();
        Self {
            cache: Cache::new(),
            data_points,
            limit: Duration::from_secs(PLOT_SECONDS as u64),
        }
    }

    fn push_data(&mut self, time: DateTime<Utc>, value: i32) {
        let cur_ms = time.timestamp_millis();
        self.data_points.push_front((time, value));
        loop {
            if let Some((time, _)) = self.data_points.back() {
                let diff = Duration::from_millis((cur_ms - time.timestamp_millis()) as u64);
                if diff > self.limit {
                    self.data_points.pop_back();
                    continue;
                }
            }
            break;
        }
        self.cache.clear();
    }

    fn view(&self, idx: usize) -> Element<Message> {
        Container::new(
            Column::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(5)
                .push(Text::new(format!("Wave")))
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

impl Chart<Message> for CpuUsageChart {
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
        let newest_time = self
            .data_points
            .front()
            .unwrap_or(&(
                chrono::DateTime::from_utc(
                    chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    chrono::Utc,
                ),
                0,
            ))
            .0;
        let oldest_time = newest_time - chrono::Duration::seconds(PLOT_SECONDS as i64);
        let mut chart = chart
            .x_label_area_size(0)
            .y_label_area_size(28)
            .margin(20)
            .build_cartesian_2d(oldest_time..newest_time, 0..100)
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
