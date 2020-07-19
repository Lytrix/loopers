use skia_safe::{
    BlendMode, Canvas, ClipOp, Color, Font, Paint, Path, Point, Rect, TextBlob, Typeface, Vector,
};

use crate::{AppData};

use crate::skia::{HEIGHT, WIDTH};
use skia_safe::paint::Style;
use std::cmp::Ordering;
use std::time::Duration;
use loopers_common::music::FrameTime;

#[allow(dead_code)]
enum AnimationFunction {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInCubic,
    EaseOutCubic,
}

impl AnimationFunction {
    fn value(&self, t: f32) -> f32 {
        match self {
            AnimationFunction::Linear => t,

            AnimationFunction::EaseInQuad => t * t,
            AnimationFunction::EaseOutQuad => t * (2.0 - t),

            AnimationFunction::EaseInCubic => t * t * t,
            AnimationFunction::EaseOutCubic => {
                let t = t - 1.0;
                t * t * t + 1.0
            }
        }
    }
}

struct Animation {
    start_time: FrameTime,
    length: Duration,
    function: AnimationFunction,
}

impl Animation {
    fn new(start_time: FrameTime, length: Duration, function: AnimationFunction) -> Self {
        Animation {
            start_time,
            length,
            function,
        }
    }

    fn value(&self, time: FrameTime) -> f32 {
        let p = (time.to_ms() - self.start_time.to_ms()) as f32 / self.length.as_millis() as f32;
        self.function.value(p)
    }
}

pub struct MainPage {
    loopers: Vec<LooperView>,
    beat_animation: Option<Animation>,
    bottom_bar: BottomBarView,
}

const LOOPER_HEIGHT: f32 = 80.0;
const WAVEFORM_OFFSET_X: f32 = 100.0;
const WAVEFORM_WIDTH: f32 = 650.0;
const WAVEFORM_ZERO_RATIO: f32 = 0.25;

impl MainPage {
    pub fn new() -> Self {
        MainPage {
            loopers: vec![],
            beat_animation: None,
            bottom_bar: BottomBarView::new(),
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas, data: &AppData) {
        if self.loopers.is_empty() {
            for _ in &data.loopers {
                self.loopers.push(LooperView::new());
            }
        }

        for (i, looper) in self.loopers.iter_mut().enumerate() {
            canvas.save();
            canvas.translate(Vector::new(0.0, i as f32 * 90.0));

            looper.draw(canvas, data, i);

            canvas.restore();
        }

        // draw play head
        let x = WAVEFORM_WIDTH * WAVEFORM_ZERO_RATIO;
        let h = self.loopers.len() as f32 * (LOOPER_HEIGHT + 10.0);

        canvas.save();
        canvas.translate(Vector::new(WAVEFORM_OFFSET_X, 0.0));
        let mut path = Path::new();
        {
            path.move_to(Point::new(x - 5.0, 10.0));
            path.line_to(Point::new(x + 5.0, 10.0));
            path.move_to(Point::new(x, 10.0));
            path.line_to(Point::new(x, h));
            path.move_to(Point::new(x - 5.0, h));
            path.line_to(Point::new(x + 5.0, h));
        }
        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        // draw overlay to darken time that is past
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::from_argb(120, 0, 0, 0));
        canvas.draw_rect(Rect::new(0.0, 10.0, x, h), &paint);

        // draw play head bar
        let beat = data.engine_state.metric_structure.tempo.beat(data.engine_state.time);
        let bom = data.engine_state.metric_structure.time_signature.beat_of_measure(beat);

        if bom == 0 {
            if self.beat_animation.is_none() {
                self.beat_animation = Some(Animation::new(
                    data.engine_state.time,
                    Duration::from_millis(500),
                    AnimationFunction::EaseOutCubic,
                ));
            }

            let v = self.beat_animation.as_ref().unwrap().value(data.engine_state.time);
            paint.set_stroke_width(3.0 + ((1.0 - v) * 5.0));
        } else {
            self.beat_animation = None;
            paint.set_stroke_width(3.0);
        }
        paint.set_color(Color::from_rgb(255, 255, 255));
        paint.set_style(Style::Stroke);

        canvas.draw_path(&path, &paint);
        canvas.restore();

        // draw the bottom bar
        canvas.save();
        let bar_height = 30.0;
        canvas.translate(Vector::new(0.0, HEIGHT as f32 - bar_height));
        self.bottom_bar.draw(canvas, WIDTH as f32, 30.0, data);
        canvas.restore();
    }
}

struct BottomBarView {}

impl BottomBarView {
    fn new() -> Self {
        Self {}
    }

    fn draw(&mut self, canvas: &mut Canvas, _w: f32, h: f32, data: &AppData) {
        let font = Font::new(Typeface::default(), 20.0);

        // let mut background = Paint::default();
        // background.set_color(Color::from_argb(100, 255, 255, 255));
        // canvas.draw_rect(Rect::new(0.0, 0.0, w, h), &background);

        let mut text_paint = Paint::default();
        text_paint.set_color(Color::WHITE);
        text_paint.set_anti_alias(true);
        canvas.draw_str(
            &format!("{} bpm", data.engine_state.metric_structure.tempo.bpm() as u32),
            Point::new(10.0, h - 12.0),
            &font,
            &text_paint,
        );

        let mut x = 130.0;

        let current_beat = data.engine_state.metric_structure.tempo.beat(data.engine_state.time);
        let beat_of_measure = data.engine_state.metric_structure.time_signature.beat_of_measure(current_beat);
        let measure = data.engine_state.metric_structure.time_signature.measure(current_beat);
        for beat in 0..data.engine_state.metric_structure.time_signature.upper {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            if beat == beat_of_measure {
                paint.set_color(Color::from_rgb(0, 255, 0));
            } else {
                paint.set_color(Color::from_rgb(128, 128, 128));
            }

            let radius = 10.0;
            canvas.draw_circle(Point::new(x, h / 2.0 - 5.0), radius, &paint);
            x += 30.0;
        }

        let mut ms = data.engine_state.time.to_ms();
        let mut negative = "";
        if ms < 0.0 {
            negative = "-";
            ms = -ms;
        }

        ms = (ms / 1000.0).floor();
        let hours = ms as u64 / 60 / 60;
        ms -= (hours * 60 * 60) as f64;
        let minutes = ms as u64 / 60;
        ms -= (minutes * 60) as f64;
        let seconds = ms as u64;

        let time_blob = TextBlob::new(
            &format!("{}{:02}:{:02}:{:02}", negative, hours, minutes, seconds),
            &font,
        )
        .unwrap();

        canvas.draw_text_blob(&time_blob, Point::new(x, h - 12.0), &text_paint);

        // TODO: should probably figure out what this bounds actually represents, since it does
        //       not seem to be a bounding box of the text as I would expect
        x += time_blob.bounds().width() - 30.0;

        let measure_blob =
            TextBlob::new(format!("{:03}.{}", measure, beat_of_measure), &font).unwrap();

        canvas.draw_text_blob(&measure_blob, Point::new(x, h - 12.0), &text_paint);
    }
}

struct LooperView {
    waveform_view: WaveformView,
}

impl LooperView {
    fn new() -> Self {
        Self {
            waveform_view: WaveformView::new(),
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, data: &AppData, index: usize) {
        let looper = &data.loopers[index];

        let ratio = (data.engine_state.time.0 % looper.length.0) as f32 / looper.length.0 as f32;
        draw_circle_indicator(canvas, looper.state.color(), ratio, 25.0, 25.0, 25.0);

        canvas.save();
        canvas.translate(Vector::new(WAVEFORM_OFFSET_X, 10.0));
        self.waveform_view
            .draw(canvas, data, index, WAVEFORM_WIDTH, LOOPER_HEIGHT);

        canvas.restore();
    }
}

fn path_for_channel(d: &[f32], w: f32, h: f32, flip: bool) -> Vec<(f32, f32)> {
    let max = *d
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .unwrap() as f32;

    let size = 2048;

    let mut path = Vec::with_capacity((d.len() / size + 1) as usize);

    let flip = if flip { 1.0 } else { 0.0 };

    for t in (0..d.len()).step_by(size as usize) {
        let x = ((t as f32) / d.len() as f32) * w;

        let mut p = 0.0;
        let mut count = 0;
        for j in 0..size {
            let i = (t + j) as usize % d.len();
            if d[i] > 0.0 {
                count += 1;
                p += d[i];
            }
        }
        p = p / count as f32;

        let y = (flip - (p as f32) / max).abs();

        path.push((x, y * h));
    }

    path
}

struct WaveformView {
    path: Option<Path>,
    bar_lines: Option<Path>,
    beat_lines: Option<Path>,
    time_width: FrameTime,
}

impl WaveformView {
    fn new() -> Self {
        Self {
            path: None,
            bar_lines: None,
            beat_lines: None,
            time_width: FrameTime::from_ms(12_000.0),
        }
    }

    fn time_to_x(&self, time: FrameTime, w: f32) -> f32 {
        // offset time back so that time 0 is at the play head
        let offset_time = time.0 as f32 - (self.time_width.0 as f32 * WAVEFORM_ZERO_RATIO);
        1.0 / (self.time_width.0 as f32 / w) * offset_time
    }

    fn draw(&mut self, canvas: &mut Canvas, data: &AppData, index: usize, w: f32, h: f32) {
        let looper = &data.loopers[index];

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::from_rgb(0, 65, 122));

        canvas.draw_rect(Rect::new(0.0, 0.0, w, h), &paint);

        let full_w = (looper.length.0 as f32 / self.time_width.0 as f32) * w;

        if self.path.is_none() {
            let p_l = path_for_channel(&looper.waveform[0], full_w, h - 10.0, true);
            let p_r = path_for_channel(&looper.waveform[1], full_w, h - 10.0, false);

            let mut p = Path::new();
            p.move_to(Point::new(0.0, h / 2.0));
            for (x, y) in &p_l {
                p.line_to(Point::new(*x, *y));
            }

            if let Some((x, y)) = &p_l.last() {
                p.line_to(Point::new(*x, *y));
            }

            if let Some((x, y)) = &p_r.last() {
                p.line_to(Point::new(*x, *y + 10.0));
            }

            for (x, y) in p_r.iter().rev() {
                p.line_to(Point::new(*x, *y + 10.0));
            }
            p.close();

            self.path = Some(p);
        }

        if self.bar_lines.is_none() {
            let mut beat_p = Path::new();
            let mut bar_p = Path::new();

            let samples_per_beat = FrameTime::from_ms(1000.0 /
                (data.engine_state.metric_structure.tempo.bpm() / 60.0) as f64);
            let number_of_beats = looper.length.0 / samples_per_beat.0;
            for i in 0..number_of_beats {
                let x = i as f32 * full_w / number_of_beats as f32;

                if i % data.engine_state.metric_structure.time_signature.upper as i64 == 0 {
                    bar_p.move_to(Point::new(x, 5.0));
                    bar_p.line_to(Point::new(x, h - 5.0));
                } else {
                    beat_p.move_to(Point::new(x, 20.0));
                    beat_p.line_to(Point::new(x, h - 20.0));
                }
            }

            self.beat_lines = Some(beat_p);
            self.bar_lines = Some(bar_p);
        }

        paint.set_color(looper.state.dark_color());
        paint.set_style(Style::StrokeAndFill);

        canvas.save();

        canvas.clip_rect(
            Rect::new(0.0, 0.0, w, h),
            Some(ClipOp::Intersect),
            Some(false),
        );

        let mut beat_paint = Paint::default();
        beat_paint
            .set_color(Color::from_argb(170, 200, 200, 255))
            .set_anti_alias(true)
            .set_stroke_width(1.0)
            .set_style(Style::Stroke)
            .set_blend_mode(BlendMode::Lighten);

        let mut bar_paint = Paint::default();
        bar_paint
            .set_color(Color::from_argb(255, 255, 255, 255))
            .set_anti_alias(true)
            .set_stroke_width(3.0)
            .set_style(Style::Stroke);
        let mut bar_outer_paint = bar_paint.clone();
        bar_outer_paint.set_color(Color::from_argb(130, 0, 0, 0));
        bar_outer_paint.set_stroke_width(4.0);

        let mut x = -self.time_to_x(data.engine_state.time, w);
        while x < w * 2.0 {
            if x + full_w > 0.0 && x < w {
                canvas.save();
                canvas.translate(Vector::new(x, 0.0));
                canvas.draw_path(self.path.as_ref().unwrap(), &paint);

                // draw beats
                canvas.draw_path(self.beat_lines.as_ref().unwrap(), &beat_paint);

                // draw bar lines
                canvas.draw_path(self.bar_lines.as_ref().unwrap(), &bar_outer_paint);
                canvas.draw_path(self.bar_lines.as_ref().unwrap(), &bar_paint);
                canvas.restore();
            }

            x += full_w;
        }

        canvas.restore();
    }
}

fn draw_circle_indicator(canvas: &mut Canvas, color: Color, p: f32, x: f32, y: f32, r: f32) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    paint.set_alpha_f(0.3);
    canvas.draw_circle(Point::new(x + r, y + r), r, &paint);

    paint.set_alpha_f(1.0);

    let mut path = Path::new();
    path.move_to(Point::new(x + r, y + r));
    path.line_to(Point::new(x + r, y));
    path.arc_to(
        Rect::new(x, y, x + 2.0 * r, y + 2.0 * r),
        270.0,
        270.0 + (p + 0.25) * 360.0,
        true,
    );
    path.line_to(Point::new(x + r, y + r));
    path.close();

    paint.set_stroke_width(2.0);
    paint.set_style(Style::StrokeAndFill);
    canvas.draw_path(&path, &paint);
}
