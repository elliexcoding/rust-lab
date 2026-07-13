use std::{env, fmt::Write as _, io};

use terminal_toy_kit::{draw_at, sleep_ms, Point, TerminalGuard};

const WIDTH: usize = 96;
const HEIGHT: usize = 30;
const FRAMES: usize = 900;
const FRAME_MS: u64 = 45;
const RESET: &str = "\x1b[0m";

fn main() -> io::Result<()> {
    let _terminal = TerminalGuard::enter()?;
    let frames = env_usize("BLACK_HOLE_FRAMES").unwrap_or(FRAMES);
    let frame_ms = env_u64("BLACK_HOLE_FRAME_MS").unwrap_or(FRAME_MS);

    for tick in 0..frames {
        let frame = render_frame(tick);
        draw_at(Point::new(1, 1), &frame)?;
        sleep_ms(frame_ms);
    }

    Ok(())
}

fn env_usize(name: &str) -> Option<usize> {
    env::var(name).ok()?.parse().ok()
}

fn env_u64(name: &str) -> Option<u64> {
    env::var(name).ok()?.parse().ok()
}

fn render_frame(tick: usize) -> String {
    let mut frame = String::with_capacity(WIDTH * HEIGHT * 16);

    frame.push_str("\x1b[1;97mBLACK HOLE\x1b[0m  ");
    frame.push_str("\x1b[38;5;201mpurple accretion disc\x1b[0m  ");
    frame.push_str("\x1b[38;5;81mcyan warped light\x1b[0m  ");
    frame.push_str("\x1b[38;5;244mdark event horizon\x1b[0m\n");
    frame.push_str(
        "\x1b[38;5;244mTeaching sketch: colors show what to notice, not a telescope measurement.\x1b[0m\n",
    );
    frame.push_str(
        "\x1b[38;5;244mPurple is hot gas orbiting fast. Cyan is light bent around the black hole.\x1b[0m\n",
    );
    frame.push_str(
        "\x1b[38;5;244mThe blank center is the event horizon: inside it, light cannot escape.\x1b[0m\n\n",
    );

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let sample = sample_cell(x, y, tick);
            let glyph = glyph_for(sample.brightness);

            if glyph == ' ' {
                frame.push(' ');
            } else {
                let _ = write!(frame, "\x1b[38;5;{}m{}{}", sample.color, glyph, RESET);
            }
        }
        if let Some(annotation) = annotation_for_row(y) {
            frame.push_str(annotation);
        }
        frame.push('\n');
    }

    frame.push_str("\n\x1b[38;5;244mWatch the purple texture drift: that is the disc rotating around the black hole.\x1b[0m");
    frame
}

#[derive(Clone, Copy, Debug)]
struct CellSample {
    brightness: f64,
    color: u8,
}

fn sample_cell(x: usize, y: usize, tick: usize) -> CellSample {
    let cx = (WIDTH as f64 - 1.0) / 2.0;
    let cy = (HEIGHT as f64 - 1.0) / 2.0 + 1.0;
    let dx = x as f64 - cx;
    let dy = y as f64 - cy;
    let phase = tick as f64 * 0.15;

    let horizon_radius = ((dx / 7.7).powi(2) + (dy / 3.9).powi(2)).sqrt();
    let photon_radius = ((dx / 10.0).powi(2) + (dy / 4.8).powi(2)).sqrt();

    if horizon_radius < 0.98 {
        return CellSample {
            brightness: 0.0,
            color: 16,
        };
    }

    let disc_x = dx / 39.0;
    let disc_y = dy / 8.4;
    let disc_radius = (disc_x.powi(2) + disc_y.powi(2)).sqrt();
    let disc_angle = disc_y.atan2(disc_x);

    let disc_band = band_strength(disc_radius, 0.42, 1.23);
    let spiral = ((disc_angle * 5.0) - phase + disc_radius * 10.5).sin();
    let ripple = ((disc_angle * 12.0) + phase * 0.55 + disc_radius * 22.0).sin();
    let doppler = ((disc_angle - phase * 0.18).cos() + 1.0) * 0.18;
    let disc_brightness = disc_band * (0.62 + spiral * 0.22 + ripple * 0.09 + doppler);

    let photon_ring = (1.0 - ((photon_radius - 1.0).abs() / 0.12)).clamp(0.0, 1.0);
    let lens_arc = lensed_arc(dx, dy, phase);
    let star_field = star_brightness(x, y, tick);

    let brightness = (star_field + disc_brightness + photon_ring * 0.9 + lens_arc).clamp(0.0, 1.0);
    let color = if disc_brightness > 0.12 {
        disc_color(disc_angle, disc_brightness)
    } else if photon_ring > 0.55 {
        159
    } else if lens_arc > 0.1 {
        81
    } else {
        245
    };

    CellSample { brightness, color }
}

fn disc_color(angle: f64, brightness: f64) -> u8 {
    let approaching_side = angle.cos() > 0.0;

    if brightness > 0.78 {
        201
    } else if approaching_side && brightness > 0.52 {
        207
    } else if brightness > 0.42 {
        171
    } else if brightness > 0.28 {
        135
    } else {
        93
    }
}

fn annotation_for_row(y: usize) -> Option<&'static str> {
    match y {
        6 => Some("   \x1b[38;5;81m<- warped light from behind the hole\x1b[0m"),
        10 => Some("   \x1b[38;5;201m<- purple accretion disc: hot orbiting gas\x1b[0m"),
        14 => Some("   \x1b[38;5;244m<- event horizon: the dark no-return region\x1b[0m"),
        18 => Some("   \x1b[38;5;201m<- same disc, visually wrapped by gravity\x1b[0m"),
        22 => Some("   \x1b[38;5;81m<- lower lensed image: light path bent downward\x1b[0m"),
        _ => None,
    }
}

fn band_strength(radius: f64, inner: f64, outer: f64) -> f64 {
    if !(inner..=outer).contains(&radius) {
        return 0.0;
    }

    let middle = (inner + outer) * 0.5;
    let half_width = (outer - inner) * 0.5;
    (1.0 - ((radius - middle).abs() / half_width).powf(1.7)).clamp(0.0, 1.0)
}

fn lensed_arc(dx: f64, dy: f64, phase: f64) -> f64 {
    let upper_radius = ((dx / 25.0).powi(2) + ((dy + 5.6) / 3.1).powi(2)).sqrt();
    let lower_radius = ((dx / 22.0).powi(2) + ((dy - 5.2) / 2.7).powi(2)).sqrt();
    let shimmer = 0.9 + ((dx * 0.27) - phase * 1.4).sin() * 0.18;

    let upper = (1.0 - ((upper_radius - 1.0).abs() / 0.1)).clamp(0.0, 1.0);
    let lower = (1.0 - ((lower_radius - 1.0).abs() / 0.13)).clamp(0.0, 1.0);

    ((upper * 0.36 + lower * 0.16) * shimmer).clamp(0.0, 0.48)
}

fn star_brightness(x: usize, y: usize, tick: usize) -> f64 {
    let hash = ((x * 37 + y * 91 + tick / 12) % 211) as f64;
    if hash > 207.0 {
        0.18
    } else if hash < 1.0 {
        0.11
    } else {
        0.0
    }
}

fn glyph_for(brightness: f64) -> char {
    match (brightness * 10.0).round() as u8 {
        0 => ' ',
        1 => '.',
        2 => ':',
        3 => '-',
        4 => '=',
        5 => '+',
        6 => '*',
        7 => '#',
        8 => '%',
        _ => '@',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_frame_has_stable_line_count() {
        let frame = render_frame(0);

        assert_eq!(frame.lines().count(), HEIGHT + 7);
    }

    #[test]
    fn accretion_disc_changes_between_frames() {
        assert_ne!(render_frame(0), render_frame(1));
    }

    #[test]
    fn event_horizon_stays_dark() {
        let center = sample_cell(WIDTH / 2, HEIGHT / 2 + 1, 20);

        assert_eq!(center.brightness, 0.0);
        assert_eq!(glyph_for(center.brightness), ' ');
    }

    #[test]
    fn bright_disc_uses_purple_palette() {
        assert_eq!(disc_color(0.0, 0.8), 201);
        assert_eq!(disc_color(0.0, 0.55), 207);
        assert_eq!(disc_color(std::f64::consts::PI, 0.2), 93);
    }

    #[test]
    fn annotations_explain_key_visual_regions() {
        assert!(annotation_for_row(6).is_some());
        assert!(annotation_for_row(14)
            .expect("event horizon annotation")
            .contains("event horizon"));
        assert!(annotation_for_row(29).is_none());
    }
}
