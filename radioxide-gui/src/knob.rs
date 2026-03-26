use iced::mouse;
use iced::widget::canvas::{self, Canvas, Event, Frame, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

use crate::theme::*;

/// Frequency step thresholds based on angular speed (radians per second).
fn freq_step_for_speed(rad_per_sec: f32) -> i64 {
    if rad_per_sec > 12.0 {
        10_000 // 10 kHz — very fast spin
    } else if rad_per_sec > 5.0 {
        1_000 // 1 kHz
    } else if rad_per_sec > 1.5 {
        100 // 100 Hz
    } else {
        10 // 10 Hz — slow, precise
    }
}

/// Normalize an angle to [-PI, PI].
fn normalize_angle(a: f32) -> f32 {
    let mut a = a % std::f32::consts::TAU;
    if a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    } else if a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

/// Compute the angle (radians) from `center` to `point`. 0 = right, PI/2 = down.
fn angle_to(center: Point, point: Point) -> f32 {
    (point.y - center.y).atan2(point.x - center.x)
}

const KNOB_SIZE: f32 = 160.0;

/// Radians of rotation per frequency step at the base (10 Hz) rate.
const RADIANS_PER_STEP: f32 = 0.105;

#[derive(Debug, Clone)]
pub enum KnobMessage {
    FreqDelta(i64),
}

/// Persistent state held inside the canvas widget (managed by iced).
#[derive(Default)]
pub struct KnobCanvasState {
    dragging: bool,
    last_angle: Option<f32>,
    last_time: Option<std::time::Instant>,
    accumulated_angle: f32,
    angular_speed: f32,
    rotation: f32,
}

/// Canvas program for the tuning knob. All interaction state lives in `KnobCanvasState`.
pub struct TuningKnobProgram;

impl canvas::Program<KnobMessage> for TuningKnobProgram {
    type State = KnobCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<KnobMessage>) {
        let center = Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        );

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bounds) {
                    state.dragging = true;
                    state.accumulated_angle = 0.0;
                    state.angular_speed = 0.0;
                    if let Some(pos) = cursor.position() {
                        state.last_angle = Some(angle_to(center, pos));
                    }
                    state.last_time = Some(std::time::Instant::now());
                    return (canvas::event::Status::Captured, None);
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.dragging {
                    state.dragging = false;
                    state.angular_speed = 0.0;
                    state.last_angle = None;
                    return (canvas::event::Status::Captured, None);
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if !state.dragging {
                    return (canvas::event::Status::Ignored, None);
                }

                let current_angle = angle_to(center, position);

                let prev_angle = match state.last_angle {
                    Some(a) => a,
                    None => {
                        state.last_angle = Some(current_angle);
                        return (canvas::event::Status::Captured, None);
                    }
                };

                // Signed angular delta: positive = clockwise (freq up)
                let d_angle = normalize_angle(current_angle - prev_angle);
                state.last_angle = Some(current_angle);

                // Update speed estimate
                let now = std::time::Instant::now();
                if let Some(prev_time) = state.last_time {
                    let dt = now.duration_since(prev_time).as_secs_f32();
                    if dt > 0.01 {
                        state.angular_speed = (d_angle.abs() / dt).min(30.0);
                        state.last_time = Some(now);
                    }
                } else {
                    state.last_time = Some(now);
                }

                // Accumulate rotation
                state.accumulated_angle += d_angle;
                state.rotation += d_angle;

                // Convert accumulated angle to frequency steps
                let steps = (state.accumulated_angle / RADIANS_PER_STEP) as i64;
                if steps == 0 {
                    return (canvas::event::Status::Captured, None);
                }
                state.accumulated_angle -= (steps as f32) * RADIANS_PER_STEP;

                let step_hz = freq_step_for_speed(state.angular_speed);
                let delta = steps * step_hz;

                return (
                    canvas::event::Status::Captured,
                    Some(KnobMessage::FreqDelta(delta)),
                );
            }
            _ => {}
        }

        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = bounds.width.min(bounds.height) / 2.0 - 8.0;

        let dragging = state.dragging;
        let speed = state.angular_speed;
        let rotation = state.rotation;

        // Outer ring (metallic look)
        let outer_ring = Path::circle(center, radius);
        frame.fill(&outer_ring, Color::from_rgb(0.18, 0.18, 0.18));
        frame.stroke(
            &outer_ring,
            Stroke::default()
                .with_color(Color::from_rgb(0.35, 0.35, 0.35))
                .with_width(2.0),
        );

        // Inner circle (knob body)
        let inner_radius = radius - 8.0;
        let inner = Path::circle(center, inner_radius);
        frame.fill(&inner, Color::from_rgb(0.13, 0.13, 0.13));
        frame.stroke(
            &inner,
            Stroke::default()
                .with_color(if dragging {
                    ACCENT_BLUE
                } else {
                    Color::from_rgb(0.28, 0.28, 0.28)
                })
                .with_width(1.5),
        );

        // Tick marks around the outer ring
        for i in 0..24 {
            let angle = (i as f32) * std::f32::consts::TAU / 24.0;
            let tick_outer = radius - 2.0;
            let tick_inner = radius - 6.0;
            let start = Point::new(
                center.x + tick_inner * angle.cos(),
                center.y + tick_inner * angle.sin(),
            );
            let end = Point::new(
                center.x + tick_outer * angle.cos(),
                center.y + tick_outer * angle.sin(),
            );
            frame.stroke(
                &Path::line(start, end),
                Stroke::default()
                    .with_color(Color::from_rgb(0.4, 0.4, 0.4))
                    .with_width(1.0),
            );
        }

        // Indicator dot (rotates with the knob)
        let dot_distance = inner_radius - 14.0;
        let dot_angle = rotation - std::f32::consts::FRAC_PI_2; // 0 starts at top
        let dot_center = Point::new(
            center.x + dot_distance * dot_angle.cos(),
            center.y + dot_distance * dot_angle.sin(),
        );
        let dot_color = if dragging {
            if speed > 12.0 {
                Color::from_rgb(1.0, 0.3, 0.3) // red = very fast
            } else if speed > 5.0 {
                TUNE_AMBER // amber = fast
            } else {
                FREQ_GREEN // green = normal
            }
        } else {
            ACCENT_BLUE
        };
        frame.fill(&Path::circle(dot_center, 5.0), dot_color);

        // Center cap
        let cap = Path::circle(center, 10.0);
        frame.fill(&cap, Color::from_rgb(0.22, 0.22, 0.22));
        frame.stroke(
            &cap,
            Stroke::default()
                .with_color(Color::from_rgb(0.3, 0.3, 0.3))
                .with_width(1.0),
        );

        // Label below the knob
        let step_label = if dragging && speed > 0.3 {
            let step = freq_step_for_speed(speed);
            match step {
                10 => "10 Hz",
                100 => "100 Hz",
                1_000 => "1 kHz",
                10_000 => "10 kHz",
                _ => "",
            }
        } else {
            "TUNE"
        };

        frame.fill_text(canvas::Text {
            content: step_label.to_string(),
            position: Point::new(center.x, bounds.height - 4.0),
            color: if dragging { dot_color } else { TEXT_DIM },
            size: 11.0.into(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Bottom,
            ..canvas::Text::default()
        });

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}

pub fn view_knob() -> Element<'static, KnobMessage> {
    Canvas::new(TuningKnobProgram)
        .width(Length::Fixed(KNOB_SIZE))
        .height(Length::Fixed(KNOB_SIZE))
        .into()
}
