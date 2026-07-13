use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::api::{GeoPoint, Launch};

const ACCENT: Color = Color::Rgb(80, 220, 255);
const WARM: Color = Color::Rgb(255, 179, 71);
const MUTED: Color = Color::Rgb(115, 135, 150);
const ROTATION_PERIOD_SECONDS: f64 = 90.0;

pub const ROTATION_DEGREES_PER_SECOND: f64 = 360.0 / ROTATION_PERIOD_SECONDS;

pub fn draw(
    frame: &mut Frame<'_>,
    area: Rect,
    launches: &[Launch],
    selected: usize,
    rotation_degrees: f64,
    paused: bool,
) {
    let motion = if paused { "PAUSED" } else { "ROTATING" };
    let centered_longitude = normalize_longitude(rotation_degrees);
    let hemisphere = if centered_longitude < 0.0 { 'W' } else { 'E' };
    let block = Block::default()
        .title(format!(
            " {motion} GLOBE  {:03.0}°{hemisphere} ",
            centered_longitude.abs()
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 18 || inner.height < 7 {
        frame.render_widget(
            Paragraph::new("Globe needs a little more terminal space")
                .style(Style::default().fg(MUTED))
                .wrap(Wrap { trim: true }),
            inner,
        );
        return;
    }

    let label_width = if inner.width >= 42 {
        (inner.width / 3).clamp(12, 24)
    } else {
        0
    };
    let globe_width = inner.width.saturating_sub(label_width);
    let radius_y = ((inner.height.saturating_sub(1)) as f64 / 2.0).max(2.0);
    let radius_x = (radius_y * 2.0)
        .min((globe_width.saturating_sub(2)) as f64 / 2.0)
        .max(3.0);
    let center_x = inner.x as f64 + (globe_width.saturating_sub(1)) as f64 / 2.0;
    let center_y = inner.y as f64 + (inner.height.saturating_sub(1)) as f64 / 2.0;

    let buffer = frame.buffer_mut();
    draw_star_field(buffer, inner);
    draw_sphere(
        buffer,
        inner,
        center_x,
        center_y,
        radius_x,
        radius_y,
        rotation_degrees,
    );
    draw_markers(
        buffer,
        inner,
        globe_width,
        center_x,
        center_y,
        radius_x,
        radius_y,
        launches,
        selected,
        rotation_degrees,
    );
}

fn draw_star_field(buffer: &mut Buffer, area: Rect) {
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let hash = u32::from(x)
                .wrapping_mul(1_103_515_245)
                .wrapping_add(u32::from(y).wrapping_mul(12_345));
            if hash % 97 == 0 {
                buffer[(x, y)]
                    .set_char('·')
                    .set_style(Style::default().fg(Color::Rgb(47, 65, 82)));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_sphere(
    buffer: &mut Buffer,
    area: Rect,
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation_degrees: f64,
) {
    for screen_y in area.y..area.bottom() {
        for screen_x in area.x..area.right() {
            let x = (f64::from(screen_x) - center_x) / radius_x;
            let y = (center_y - f64::from(screen_y)) / radius_y;
            let distance_squared = x * x + y * y;
            if distance_squared > 1.0 {
                continue;
            }

            let z = (1.0 - distance_squared).sqrt();
            let latitude = y.clamp(-1.0, 1.0).asin().to_degrees();
            let relative_longitude = x.atan2(z).to_degrees();
            let longitude = normalize_longitude(rotation_degrees + relative_longitude);
            let light = (0.18 + x * 0.15 + y * 0.12 + z * 0.7).clamp(0.0, 1.0);
            let grid = near_grid_line(latitude, 30.0, 2.2)
                || (latitude.abs() < 78.0 && near_grid_line(longitude, 30.0, 2.2));
            let land = is_land(latitude, longitude);

            let (symbol, color) = if distance_squared > 0.94 {
                ('•', shade(Color::Rgb(42, 145, 180), light))
            } else if grid {
                ('·', shade(Color::Rgb(69, 174, 194), light))
            } else if land {
                (
                    if light > 0.55 { '▓' } else { '▒' },
                    shade(Color::Rgb(45, 173, 116), light),
                )
            } else {
                ('░', shade(Color::Rgb(28, 111, 157), light))
            };

            buffer[(screen_x, screen_y)]
                .set_char(symbol)
                .set_style(Style::default().fg(color));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_markers(
    buffer: &mut Buffer,
    area: Rect,
    globe_width: u16,
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
    launches: &[Launch],
    selected: usize,
    rotation_degrees: f64,
) {
    for (index, launch) in launches.iter().enumerate() {
        if index == selected {
            continue;
        }
        let Some(point) = launch.coordinates else {
            continue;
        };
        let projection = project(point, rotation_degrees);
        if !projection.visible {
            continue;
        }
        let (x, y) = screen_position(projection, center_x, center_y, radius_x, radius_y);
        if area.contains((x, y).into()) {
            buffer[(x, y)]
                .set_char('•')
                .set_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD));
        }
    }

    let Some(launch) = launches.get(selected) else {
        return;
    };
    let Some(point) = launch.coordinates else {
        draw_unavailable_label(buffer, area, globe_width, &launch.site_name);
        return;
    };

    let projection = project(point, rotation_degrees);
    let marker_projection = if projection.visible {
        projection
    } else {
        projection.on_nearest_rim()
    };
    let (marker_x, marker_y) =
        screen_position(marker_projection, center_x, center_y, radius_x, radius_y);
    let suffix = if projection.visible { "" } else { "  FAR SIDE" };
    let label = format!("{}{}", launch.site_name, suffix);

    draw_leader_line(buffer, area, globe_width, marker_x, marker_y, &label);
    buffer[(marker_x, marker_y)]
        .set_char(if projection.visible { '◆' } else { '◇' })
        .set_style(Style::default().fg(WARM).add_modifier(Modifier::BOLD));
}

fn draw_leader_line(
    buffer: &mut Buffer,
    area: Rect,
    globe_width: u16,
    marker_x: u16,
    marker_y: u16,
    label: &str,
) {
    let style = Style::default().fg(WARM);
    let has_label_column = globe_width < area.width;

    if has_label_column {
        let label_x = area.x + globe_width + 1;
        let label_y = marker_y.clamp(area.y, area.bottom().saturating_sub(1));
        for x in marker_x.saturating_add(1)..label_x {
            if area.contains((x, label_y).into()) {
                buffer[(x, label_y)].set_char('─').set_style(style);
            }
        }
        let available = area.right().saturating_sub(label_x) as usize;
        buffer.set_stringn(label_x, label_y, label, available, style);
        return;
    }

    let label_y = area.bottom().saturating_sub(1);
    for y in marker_y.saturating_add(1)..label_y {
        buffer[(marker_x, y)].set_char('│').set_style(style);
    }
    if marker_y < label_y {
        buffer[(marker_x, label_y)].set_char('┘').set_style(style);
    }
    let label_x = area.x.saturating_add(1);
    let (start, end) = if label_x < marker_x {
        (label_x, marker_x)
    } else {
        (marker_x.saturating_add(1), label_x)
    };
    for x in start..end {
        buffer[(x, label_y)].set_char('─').set_style(style);
    }
    let available = area.right().saturating_sub(label_x) as usize;
    buffer.set_stringn(label_x, label_y, label, available, style);
}

fn draw_unavailable_label(buffer: &mut Buffer, area: Rect, globe_width: u16, site_name: &str) {
    let text = format!("{site_name}  LOCATION UNAVAILABLE");
    let (x, y) = if globe_width < area.width {
        (area.x + globe_width + 1, area.y + area.height / 2)
    } else {
        (area.x + 1, area.bottom().saturating_sub(1))
    };
    let available = area.right().saturating_sub(x) as usize;
    buffer.set_stringn(x, y, text, available, Style::default().fg(MUTED));
}

#[derive(Clone, Copy, Debug)]
struct Projection {
    x: f64,
    y: f64,
    visible: bool,
}

impl Projection {
    fn on_nearest_rim(self) -> Self {
        let magnitude = (self.x * self.x + self.y * self.y).sqrt();
        let (x, y) = if magnitude > 0.08 {
            (self.x / magnitude, self.y / magnitude)
        } else {
            (1.0, 0.0)
        };
        Self {
            x: x * 0.96,
            y: y * 0.96,
            visible: false,
        }
    }
}

fn project(point: GeoPoint, rotation_degrees: f64) -> Projection {
    let latitude = point.latitude.to_radians();
    let delta_longitude = normalize_longitude(point.longitude - rotation_degrees).to_radians();
    let x = latitude.cos() * delta_longitude.sin();
    let y = latitude.sin();
    let z = latitude.cos() * delta_longitude.cos();
    Projection {
        x,
        y,
        visible: z >= 0.0,
    }
}

fn screen_position(
    projection: Projection,
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
) -> (u16, u16) {
    let x = (center_x + projection.x * radius_x).round().max(0.0) as u16;
    let y = (center_y - projection.y * radius_y).round().max(0.0) as u16;
    (x, y)
}

fn shade(base: Color, light: f64) -> Color {
    let Color::Rgb(red, green, blue) = base else {
        return base;
    };
    let factor = 0.34 + light * 0.66;
    Color::Rgb(
        (f64::from(red) * factor).round() as u8,
        (f64::from(green) * factor).round() as u8,
        (f64::from(blue) * factor).round() as u8,
    )
}

fn near_grid_line(value: f64, spacing: f64, tolerance: f64) -> bool {
    let remainder = value.rem_euclid(spacing);
    remainder.min(spacing - remainder) <= tolerance
}

fn normalize_longitude(longitude: f64) -> f64 {
    (longitude + 180.0).rem_euclid(360.0) - 180.0
}

fn is_land(latitude: f64, longitude: f64) -> bool {
    if latitude < -68.0 {
        return true;
    }
    LAND_POLYGONS
        .iter()
        .any(|polygon| point_in_polygon(longitude, latitude, polygon))
}

fn point_in_polygon(longitude: f64, latitude: f64, polygon: &[(f64, f64)]) -> bool {
    let mut inside = false;
    let mut previous = polygon.len() - 1;
    for current in 0..polygon.len() {
        let (current_lon, current_lat) = polygon[current];
        let (previous_lon, previous_lat) = polygon[previous];
        let crosses = (current_lat > latitude) != (previous_lat > latitude)
            && longitude
                < (previous_lon - current_lon) * (latitude - current_lat)
                    / (previous_lat - current_lat)
                    + current_lon;
        if crosses {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

const NORTH_AMERICA: &[(f64, f64)] = &[
    (-168.0, 72.0),
    (-145.0, 60.0),
    (-125.0, 50.0),
    (-125.0, 30.0),
    (-105.0, 23.0),
    (-91.0, 18.0),
    (-82.0, 8.0),
    (-76.0, 25.0),
    (-65.0, 45.0),
    (-55.0, 50.0),
    (-62.0, 66.0),
    (-95.0, 80.0),
    (-140.0, 75.0),
];
const SOUTH_AMERICA: &[(f64, f64)] = &[
    (-81.0, 12.0),
    (-65.0, 10.0),
    (-49.0, 2.0),
    (-35.0, -8.0),
    (-44.0, -24.0),
    (-55.0, -55.0),
    (-70.0, -50.0),
    (-81.0, -20.0),
];
const GREENLAND: &[(f64, f64)] = &[
    (-73.0, 59.0),
    (-44.0, 60.0),
    (-20.0, 75.0),
    (-40.0, 84.0),
    (-64.0, 80.0),
];
const AFRICA: &[(f64, f64)] = &[
    (-17.0, 37.0),
    (12.0, 37.0),
    (34.0, 31.0),
    (51.0, 11.0),
    (42.0, -20.0),
    (20.0, -35.0),
    (5.0, -35.0),
    (-10.0, 5.0),
];
const EURASIA: &[(f64, f64)] = &[
    (-10.0, 36.0),
    (-10.0, 60.0),
    (20.0, 72.0),
    (60.0, 76.0),
    (105.0, 76.0),
    (145.0, 70.0),
    (178.0, 62.0),
    (165.0, 50.0),
    (140.0, 40.0),
    (125.0, 20.0),
    (105.0, 5.0),
    (80.0, 8.0),
    (62.0, 25.0),
    (36.0, 30.0),
    (20.0, 38.0),
];
const ARABIA_INDIA: &[(f64, f64)] = &[
    (34.0, 31.0),
    (58.0, 26.0),
    (78.0, 30.0),
    (90.0, 22.0),
    (80.0, 7.0),
    (70.0, 20.0),
    (52.0, 13.0),
    (42.0, 13.0),
];
const SOUTH_EAST_ASIA: &[(f64, f64)] = &[
    (95.0, 24.0),
    (122.0, 25.0),
    (132.0, 8.0),
    (120.0, -10.0),
    (105.0, -8.0),
];
const AUSTRALIA: &[(f64, f64)] = &[
    (112.0, -10.0),
    (145.0, -11.0),
    (155.0, -27.0),
    (145.0, -40.0),
    (115.0, -35.0),
];
const MADAGASCAR: &[(f64, f64)] = &[(47.0, -12.0), (51.0, -17.0), (48.0, -26.0), (44.0, -20.0)];

const LAND_POLYGONS: &[&[(f64, f64)]] = &[
    NORTH_AMERICA,
    SOUTH_AMERICA,
    GREENLAND,
    AFRICA,
    EURASIA,
    ARABIA_INDIA,
    SOUTH_EAST_ASIA,
    AUSTRALIA,
    MADAGASCAR,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orthographic_projection_hides_the_far_side() {
        let front = project(GeoPoint::new(0.0, 0.0), 0.0);
        let back = project(GeoPoint::new(0.0, 180.0), 0.0);

        assert!(front.visible);
        assert!(front.x.abs() < f64::EPSILON);
        assert!(!back.visible);
    }

    #[test]
    fn coarse_land_mask_distinguishes_continent_and_ocean() {
        assert!(is_land(34.742, -120.572));
        assert!(is_land(-25.0, 135.0));
        assert!(!is_land(0.0, -140.0));
    }

    #[test]
    fn rotation_speed_completes_one_turn_in_ninety_seconds() {
        assert!((ROTATION_DEGREES_PER_SECOND * 90.0 - 360.0).abs() < f64::EPSILON);
        assert!((normalize_longitude(370.0) - 10.0).abs() < f64::EPSILON);
    }
}
