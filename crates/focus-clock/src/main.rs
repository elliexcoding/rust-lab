use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use time::OffsetDateTime;

const FRAME_TIME: Duration = Duration::from_millis(33);
const TRANSITION_TIME: f32 = 0.42;
const DIGIT_ROWS: usize = 7;
const CELL: &str = "██";
const GHOST: &str = "░░";
const GAP: &str = "  ";

const DIGITS: [[&str; DIGIT_ROWS]; 10] = [
    [
        "11111", "10001", "10011", "10101", "11001", "10001", "11111",
    ],
    [
        "00100", "01100", "00100", "00100", "00100", "00100", "01110",
    ],
    [
        "11111", "00001", "00001", "11111", "10000", "10000", "11111",
    ],
    [
        "11111", "00001", "00001", "11111", "00001", "00001", "11111",
    ],
    [
        "10001", "10001", "10001", "11111", "00001", "00001", "00001",
    ],
    [
        "11111", "10000", "10000", "11111", "00001", "00001", "11111",
    ],
    [
        "11111", "10000", "10000", "11111", "10001", "10001", "11111",
    ],
    [
        "11111", "00001", "00010", "00100", "01000", "01000", "01000",
    ],
    [
        "11111", "10001", "10001", "11111", "10001", "10001", "11111",
    ],
    [
        "11111", "10001", "10001", "11111", "00001", "00001", "11111",
    ],
];

const COLON: [&str; DIGIT_ROWS] = ["00", "11", "11", "00", "11", "11", "00"];

fn main() -> io::Result<()> {
    ratatui::run(run)
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    loop {
        let tick_started = Instant::now();
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let clock = ClockFace::from_hms(
            u32::from(now.hour()),
            u32::from(now.minute()),
            u32::from(now.second()),
        );
        let previous = if now.second() == 0 {
            clock.previous_minute()
        } else {
            clock
        };
        let progress = if now.second() == 0 {
            transition_progress(now.nanosecond())
        } else {
            1.0
        };

        terminal.draw(|frame| render(frame, &previous, &clock, progress))?;

        if should_quit(FRAME_TIME.saturating_sub(tick_started.elapsed()))? {
            break Ok(());
        }
    }
}

fn should_quit(timeout: Duration) -> io::Result<bool> {
    if !event::poll(timeout)? {
        return Ok(false);
    }

    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => Ok(matches!(
            (key.code, key.modifiers),
            (KeyCode::Esc | KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL)
        )),
        _ => Ok(false),
    }
}

fn render(frame: &mut Frame, previous: &ClockFace, current: &ClockFace, progress: f32) {
    let area = centered(frame.area(), 90, 15);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(64, 176, 190)))
        .style(Style::default().bg(Color::Rgb(5, 8, 14)))
        .title(
            Line::from(" focus clock ").style(
                Style::default()
                    .fg(Color::Rgb(181, 236, 236))
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .title_alignment(Alignment::Center);

    let lines = clock_lines(previous, current, progress);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().bg(Color::Rgb(5, 8, 14))),
        area,
    );
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = area.width.min(width);
    let height = area.height.min(height);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn clock_lines(previous: &ClockFace, current: &ClockFace, progress: f32) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(DIGIT_ROWS + 4);
    lines.push(Line::raw(""));
    lines.extend((0..DIGIT_ROWS).map(|row| clock_row(previous, current, row, progress)));
    lines.push(Line::raw(""));
    lines.push(
        Line::from(" q quit ")
            .centered()
            .style(Style::default().fg(Color::Rgb(94, 129, 141))),
    );
    lines
}

fn clock_row(
    previous: &ClockFace,
    current: &ClockFace,
    row: usize,
    progress: f32,
) -> Line<'static> {
    let mut spans = Vec::new();

    for (index, symbol) in current.symbols.iter().enumerate() {
        let before = if index >= ClockFace::SECONDS_START {
            *symbol
        } else {
            previous.symbols[index]
        };
        spans.extend(symbol_spans(before, *symbol, row, progress, index));
        spans.push(Span::raw(GAP));
    }

    Line::from(spans).centered()
}

fn symbol_spans(
    before: ClockSymbol,
    after: ClockSymbol,
    row: usize,
    progress: f32,
    symbol_index: usize,
) -> Vec<Span<'static>> {
    let before_pattern = before.pattern();
    let after_pattern = after.pattern();
    let mut spans = Vec::with_capacity(after_pattern[row].len());

    for (column, (old, new)) in before_pattern[row]
        .bytes()
        .zip(after_pattern[row].bytes())
        .enumerate()
    {
        let style = cell_style(
            old == b'1',
            new == b'1',
            row,
            column,
            symbol_index,
            progress,
        );
        let glyph = cell_glyph(old == b'1', new == b'1', progress);
        spans.push(Span::styled(glyph, style));
    }

    spans
}

fn cell_glyph(old: bool, new: bool, progress: f32) -> &'static str {
    if new || (old && progress < 1.0) {
        CELL
    } else {
        GHOST
    }
}

fn cell_style(
    old: bool,
    new: bool,
    row: usize,
    column: usize,
    symbol_index: usize,
    progress: f32,
) -> Style {
    let shimmer = ((row + column + symbol_index) % 5) as f32 * 0.035;
    let eased = ease_out_cubic((progress - shimmer).clamp(0.0, 1.0));

    let foreground = match (old, new) {
        (true, true) => blend(
            ColorRgb::new(74, 211, 214),
            ColorRgb::new(185, 255, 222),
            eased,
        ),
        (false, true) => blend(
            ColorRgb::new(35, 86, 106),
            ColorRgb::new(245, 255, 185),
            eased,
        ),
        (true, false) => blend(
            ColorRgb::new(138, 209, 218),
            ColorRgb::new(25, 49, 61),
            eased,
        ),
        (false, false) => ColorRgb::new(17, 31, 42),
    };

    let mut style = Style::default()
        .fg(foreground.into())
        .bg(Color::Rgb(5, 8, 14));
    if new && progress > 0.12 {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}

fn transition_progress(nanosecond: u32) -> f32 {
    (nanosecond as f32 / 1_000_000_000.0 / TRANSITION_TIME).clamp(0.0, 1.0)
}

fn ease_out_cubic(value: f32) -> f32 {
    1.0 - (1.0 - value).powi(3)
}

fn blend(from: ColorRgb, to: ColorRgb, amount: f32) -> ColorRgb {
    let amount = amount.clamp(0.0, 1.0);

    ColorRgb {
        red: lerp(from.red, to.red, amount),
        green: lerp(from.green, to.green, amount),
        blue: lerp(from.blue, to.blue, amount),
    }
}

fn lerp(from: u8, to: u8, amount: f32) -> u8 {
    (from as f32 + (to as f32 - from as f32) * amount).round() as u8
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ColorRgb {
    red: u8,
    green: u8,
    blue: u8,
}

impl ColorRgb {
    const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl From<ColorRgb> for Color {
    fn from(value: ColorRgb) -> Self {
        Self::Rgb(value.red, value.green, value.blue)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClockFace {
    symbols: [ClockSymbol; 8],
}

impl ClockFace {
    const SECONDS_START: usize = 6;

    fn from_hms(hour: u32, minute: u32, second: u32) -> Self {
        Self {
            symbols: [
                ClockSymbol::Digit(hour / 10),
                ClockSymbol::Digit(hour % 10),
                ClockSymbol::Colon,
                ClockSymbol::Digit(minute / 10),
                ClockSymbol::Digit(minute % 10),
                ClockSymbol::Colon,
                ClockSymbol::Digit(second / 10),
                ClockSymbol::Digit(second % 10),
            ],
        }
    }

    fn previous_minute(self) -> Self {
        let (hour, minute, _) = self.hms();
        let previous_total = (hour * 60 + minute + 24 * 60 - 1) % (24 * 60);

        Self::from_hms(previous_total / 60, previous_total % 60, 0)
    }

    fn hms(self) -> (u32, u32, u32) {
        let [ClockSymbol::Digit(h1), ClockSymbol::Digit(h2), _, ClockSymbol::Digit(m1), ClockSymbol::Digit(m2), _, ClockSymbol::Digit(s1), ClockSymbol::Digit(s2)] =
            self.symbols
        else {
            unreachable!("clock faces are only constructed with digit separators");
        };

        (h1 * 10 + h2, m1 * 10 + m2, s1 * 10 + s2)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClockSymbol {
    Digit(u32),
    Colon,
}

impl ClockSymbol {
    fn pattern(self) -> &'static [&'static str; DIGIT_ROWS] {
        match self {
            Self::Digit(value) => &DIGITS[value as usize],
            Self::Colon => &COLON,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_face_formats_hms_as_symbols() {
        let clock = ClockFace::from_hms(9, 5, 7);

        assert_eq!(
            clock.symbols,
            [
                ClockSymbol::Digit(0),
                ClockSymbol::Digit(9),
                ClockSymbol::Colon,
                ClockSymbol::Digit(0),
                ClockSymbol::Digit(5),
                ClockSymbol::Colon,
                ClockSymbol::Digit(0),
                ClockSymbol::Digit(7),
            ]
        );
    }

    #[test]
    fn previous_minute_wraps_midnight() {
        assert_eq!(
            ClockFace::from_hms(0, 0, 0).previous_minute().hms(),
            (23, 59, 0)
        );
    }

    #[test]
    fn completed_fade_uses_ghost_glyph_for_old_cells() {
        assert_eq!(cell_glyph(true, false, 0.5), CELL);
        assert_eq!(cell_glyph(true, false, 1.0), GHOST);
        assert_eq!(cell_glyph(false, true, 1.0), CELL);
    }

    #[test]
    fn transition_progress_is_done_after_transition_window() {
        assert_eq!(transition_progress(0), 0.0);
        assert_eq!(transition_progress(420_000_000), 1.0);
        assert_eq!(transition_progress(999_999_999), 1.0);
    }

    #[test]
    fn digit_patterns_have_stable_height_and_width() {
        for digit in DIGITS {
            assert_eq!(digit.len(), DIGIT_ROWS);
            assert!(digit.iter().all(|row| row.len() == 5));
        }

        assert_eq!(COLON.len(), DIGIT_ROWS);
        assert!(COLON.iter().all(|row| row.len() == 2));
    }

    #[test]
    fn color_blending_reaches_both_ends() {
        let first = ColorRgb::new(0, 10, 20);
        let second = ColorRgb::new(100, 110, 120);

        assert_eq!(blend(first, second, 0.0), first);
        assert_eq!(blend(first, second, 1.0), second);
    }
}
