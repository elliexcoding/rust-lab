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
const MAX_TIMER_SECONDS: u64 = 99 * 3600 + 59 * 60 + 59;
const DISPLAY_WIDTH: u16 = 90;
const DISPLAY_HEIGHT: u16 = 12;

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
    let mut app = App::default();

    loop {
        let tick_started = Instant::now();
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let instant = Instant::now();

        terminal.draw(|frame| render(frame, &app, now, instant))?;

        if handle_events(
            &mut app,
            instant,
            FRAME_TIME.saturating_sub(tick_started.elapsed()),
        )? {
            break Ok(());
        }
    }
}

fn handle_events(app: &mut App, now: Instant, timeout: Duration) -> io::Result<bool> {
    if !event::poll(timeout)? {
        return Ok(false);
    }

    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            if matches!(
                (key.code, key.modifiers),
                (KeyCode::Char('c'), KeyModifiers::CONTROL)
            ) {
                return Ok(true);
            }

            Ok(match app.mode {
                AppMode::Clock | AppMode::Timer => handle_main_key(app, now, key.code),
                AppMode::TimerInput => handle_timer_input_key(app, now, key.code),
            })
        }
        _ => Ok(false),
    }
}

fn handle_main_key(app: &mut App, now: Instant, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => true,
        KeyCode::Char('t') => {
            app.mode = AppMode::TimerInput;
            app.timer_input = TimerInput::default();
            app.input_invalid = false;
            false
        }
        KeyCode::Char('r') => {
            app.timer = None;
            app.mode = AppMode::Clock;
            false
        }
        KeyCode::Char('c') => {
            app.mode = AppMode::Clock;
            false
        }
        KeyCode::Char(' ') => {
            if let Some(timer) = app.timer.as_mut() {
                timer.toggle_pause(now);
            }
            false
        }
        _ => false,
    }
}

fn handle_timer_input_key(app: &mut App, now: Instant, code: KeyCode) -> bool {
    match code {
        KeyCode::Esc => {
            app.mode = if app.timer.is_some() {
                AppMode::Timer
            } else {
                AppMode::Clock
            };
            app.input_invalid = false;
            false
        }
        KeyCode::Enter => {
            if let Some(duration) = app.timer_input.duration() {
                app.timer = Some(Timer::new(duration, now));
                app.mode = AppMode::Timer;
                app.timer_input = TimerInput::default();
                app.input_invalid = false;
            } else {
                app.input_invalid = true;
            }
            false
        }
        KeyCode::Backspace => {
            app.timer_input.backspace();
            app.input_invalid = false;
            false
        }
        KeyCode::Left => {
            app.timer_input.move_left();
            app.input_invalid = false;
            false
        }
        KeyCode::Right => {
            app.timer_input.move_right();
            app.input_invalid = false;
            false
        }
        KeyCode::Up => {
            app.timer_input.increment_active();
            app.input_invalid = false;
            false
        }
        KeyCode::Down => {
            app.timer_input.decrement_active();
            app.input_invalid = false;
            false
        }
        KeyCode::Char(ch) if ch.is_ascii_digit() => {
            app.timer_input.push_digit(ch);
            app.input_invalid = false;
            false
        }
        _ => false,
    }
}

#[derive(Default)]
struct App {
    mode: AppMode,
    timer: Option<Timer>,
    timer_input: TimerInput,
    input_invalid: bool,
}

impl App {
    fn shows_timer_panel(&self) -> bool {
        self.mode == AppMode::TimerInput || self.timer.is_some()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum AppMode {
    #[default]
    Clock,
    TimerInput,
    Timer,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TimerInput {
    fields: [u32; 3],
    buffers: [String; 3],
    active: TimerField,
}

impl Default for TimerInput {
    fn default() -> Self {
        Self {
            fields: [0; 3],
            buffers: [String::new(), String::new(), String::new()],
            active: TimerField::Minutes,
        }
    }
}

impl TimerInput {
    fn duration(&self) -> Option<Duration> {
        let seconds = u64::from(self.fields[TimerField::Hours.index()]) * 3600
            + u64::from(self.fields[TimerField::Minutes.index()]) * 60
            + u64::from(self.fields[TimerField::Seconds.index()]);

        (seconds > 0).then(|| Duration::from_secs(seconds.min(MAX_TIMER_SECONDS)))
    }

    fn preview_duration(&self) -> Duration {
        self.duration().unwrap_or(Duration::ZERO)
    }

    fn push_digit(&mut self, digit: char) {
        let index = self.active.index();
        if self.buffers[index].len() >= 2 {
            self.buffers[index].clear();
        }
        self.buffers[index].push(digit);
        self.fields[index] = self.buffers[index]
            .parse::<u32>()
            .unwrap_or(0)
            .min(self.active.max_value());
    }

    fn backspace(&mut self) {
        let index = self.active.index();
        self.buffers[index].pop();
        self.fields[index] = self.buffers[index].parse::<u32>().unwrap_or(0);
    }

    fn move_left(&mut self) {
        self.active = self.active.previous();
    }

    fn move_right(&mut self) {
        self.active = self.active.next();
    }

    fn increment_active(&mut self) {
        let index = self.active.index();
        self.fields[index] = (self.fields[index] + 1).min(self.active.max_value());
        self.sync_active_buffer();
    }

    fn decrement_active(&mut self) {
        let index = self.active.index();
        self.fields[index] = self.fields[index].saturating_sub(1);
        self.sync_active_buffer();
    }

    fn sync_active_buffer(&mut self) {
        let index = self.active.index();
        self.buffers[index] = if self.fields[index] == 0 {
            String::new()
        } else {
            self.fields[index].to_string()
        };
    }

    fn field_value(&self, field: TimerField) -> u32 {
        self.fields[field.index()]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum TimerField {
    Hours,
    #[default]
    Minutes,
    Seconds,
}

impl TimerField {
    const fn index(self) -> usize {
        match self {
            Self::Hours => 0,
            Self::Minutes => 1,
            Self::Seconds => 2,
        }
    }

    const fn max_value(self) -> u32 {
        match self {
            Self::Hours => 99,
            Self::Minutes | Self::Seconds => 59,
        }
    }

    const fn previous(self) -> Self {
        match self {
            Self::Hours => Self::Seconds,
            Self::Minutes => Self::Hours,
            Self::Seconds => Self::Minutes,
        }
    }

    const fn next(self) -> Self {
        match self {
            Self::Hours => Self::Minutes,
            Self::Minutes => Self::Seconds,
            Self::Seconds => Self::Hours,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Timer {
    duration: Duration,
    started_at: Instant,
    paused_remaining: Option<Duration>,
}

impl Timer {
    fn new(duration: Duration, now: Instant) -> Self {
        Self {
            duration,
            started_at: now,
            paused_remaining: None,
        }
    }

    fn remaining(self, now: Instant) -> Duration {
        self.paused_remaining.unwrap_or_else(|| {
            self.duration
                .saturating_sub(now.saturating_duration_since(self.started_at))
        })
    }

    fn is_paused(self) -> bool {
        self.paused_remaining.is_some()
    }

    fn toggle_pause(&mut self, now: Instant) {
        if let Some(remaining) = self.paused_remaining.take() {
            self.duration = remaining;
            self.started_at = now;
        } else {
            self.paused_remaining = Some(self.remaining(now));
        }
    }
}

struct DisplayState {
    title: &'static str,
    previous: ClockFace,
    current: ClockFace,
    progress: f32,
    lock_seconds: bool,
    palette: DigitPalette,
    status: DisplayStatus,
}

impl DisplayState {
    fn clock(now: OffsetDateTime) -> Self {
        let current = ClockFace::from_hms(
            u32::from(now.hour()),
            u32::from(now.minute()),
            u32::from(now.second()),
        );
        let previous = if now.second() == 0 {
            current.previous_minute()
        } else {
            current
        };
        let progress = if now.second() == 0 {
            transition_progress(now.nanosecond())
        } else {
            1.0
        };

        Self {
            title: " focus clock ",
            previous,
            current,
            progress,
            lock_seconds: true,
            palette: CLOCK_PALETTE,
            status: DisplayStatus::Clock,
        }
    }

    fn timer_input(input: &TimerInput, invalid: bool) -> Self {
        let current = ClockFace::from_duration(input.preview_duration());

        Self {
            title: " timer ",
            previous: current,
            current,
            progress: 1.0,
            lock_seconds: false,
            palette: TIMER_PALETTE,
            status: DisplayStatus::TimerInput {
                input: input.clone(),
                invalid,
            },
        }
    }

    fn timer(timer: Timer, now: Instant) -> Self {
        let remaining = timer.remaining(now);
        let current = ClockFace::from_duration(remaining);
        let status = if remaining.is_zero() {
            DisplayStatus::TimerDone
        } else if timer.is_paused() {
            DisplayStatus::TimerPaused
        } else {
            DisplayStatus::TimerRunning
        };

        Self {
            title: " timer ",
            previous: current,
            current,
            progress: 1.0,
            lock_seconds: false,
            palette: TIMER_PALETTE,
            status,
        }
    }
}

enum DisplayStatus {
    Clock,
    TimerInput { input: TimerInput, invalid: bool },
    TimerRunning,
    TimerPaused,
    TimerDone,
}

#[derive(Clone, Copy, Debug)]
struct DigitPalette {
    border: ColorRgb,
    title: ColorRgb,
    status: ColorRgb,
    steady: ColorRgb,
    bright: ColorRgb,
    entering: ColorRgb,
    pop: ColorRgb,
    leaving: ColorRgb,
    fade: ColorRgb,
    ghost: ColorRgb,
}

const CLOCK_PALETTE: DigitPalette = DigitPalette {
    border: ColorRgb::new(64, 176, 190),
    title: ColorRgb::new(181, 236, 236),
    status: ColorRgb::new(94, 129, 141),
    steady: ColorRgb::new(74, 211, 214),
    bright: ColorRgb::new(148, 242, 255),
    entering: ColorRgb::new(35, 86, 106),
    pop: ColorRgb::new(128, 229, 255),
    leaving: ColorRgb::new(138, 209, 218),
    fade: ColorRgb::new(25, 49, 61),
    ghost: ColorRgb::new(17, 31, 42),
};

const TIMER_PALETTE: DigitPalette = DigitPalette {
    border: ColorRgb::new(172, 99, 255),
    title: ColorRgb::new(224, 191, 255),
    status: ColorRgb::new(174, 143, 211),
    steady: ColorRgb::new(184, 111, 255),
    bright: ColorRgb::new(238, 211, 255),
    entering: ColorRgb::new(91, 50, 135),
    pop: ColorRgb::new(255, 197, 255),
    leaving: ColorRgb::new(176, 128, 217),
    fade: ColorRgb::new(49, 32, 69),
    ghost: ColorRgb::new(30, 21, 42),
};

fn render(frame: &mut Frame, app: &App, now: OffsetDateTime, instant: Instant) {
    let clock = DisplayState::clock(now);

    if app.shows_timer_panel() {
        let (clock_area, timer_area) = stacked_areas(frame.area());
        render_panel(frame, &clock, clock_area);

        let timer = timer_display(app, instant);
        render_panel(frame, &timer, timer_area);
    } else {
        render_panel(
            frame,
            &clock,
            centered(frame.area(), DISPLAY_WIDTH, DISPLAY_HEIGHT),
        );
    }
}

fn timer_display(app: &App, instant: Instant) -> DisplayState {
    if app.mode == AppMode::TimerInput {
        return DisplayState::timer_input(&app.timer_input, app.input_invalid);
    }

    app.timer
        .map(|timer| DisplayState::timer(timer, instant))
        .unwrap_or_else(|| DisplayState::timer_input(&TimerInput::default(), false))
}

fn render_panel(frame: &mut Frame, display: &DisplayState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(display.palette.border.into()))
        .style(Style::default().bg(Color::Rgb(5, 8, 14)))
        .title(
            Line::from(display.title).style(
                Style::default()
                    .fg(display.palette.title.into())
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .title_alignment(Alignment::Center);

    let lines = digit_lines(display);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().bg(Color::Rgb(5, 8, 14))),
        area,
    );
}

fn stacked_areas(area: Rect) -> (Rect, Rect) {
    let gap = if area.height > DISPLAY_HEIGHT * 2 {
        1
    } else {
        0
    };
    let height = DISPLAY_HEIGHT
        .min(area.height.saturating_sub(gap) / 2)
        .max(1);
    let width = area.width.min(DISPLAY_WIDTH);
    let total_height = height * 2 + gap;
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(total_height) / 2;
    let top = Rect {
        x,
        y,
        width,
        height,
    };
    let bottom = Rect {
        x,
        y: y + height + gap,
        width,
        height,
    };

    (top, bottom)
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

fn digit_lines(display: &DisplayState) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(DIGIT_ROWS + 4);
    lines.push(Line::raw(""));
    lines.extend((0..DIGIT_ROWS).map(|row| digit_row(display, row)));
    lines.push(Line::raw(""));
    lines.push(status_line(display));
    lines
}

fn digit_row(display: &DisplayState, row: usize) -> Line<'static> {
    let mut spans = Vec::new();

    for (index, symbol) in display.current.symbols.iter().enumerate() {
        let before = if display.lock_seconds && index >= ClockFace::SECONDS_START {
            *symbol
        } else {
            display.previous.symbols[index]
        };
        let progress = if before == *symbol {
            1.0
        } else {
            display.progress
        };
        spans.extend(symbol_spans(
            before,
            *symbol,
            row,
            progress,
            index,
            display.palette,
        ));
        spans.push(Span::raw(GAP));
    }

    Line::from(spans).centered()
}

fn status_line(display: &DisplayState) -> Line<'static> {
    let style = Style::default().fg(display.palette.status.into());
    match display.status {
        DisplayStatus::Clock => Line::from(" t timer  q quit ").centered().style(style),
        DisplayStatus::TimerRunning => Line::from(" space pause  r reset  q quit ")
            .centered()
            .style(style),
        DisplayStatus::TimerPaused => Line::from(" space resume  r reset  q quit ")
            .centered()
            .style(style),
        DisplayStatus::TimerDone => Line::from(" done  r reset  t new  q quit ")
            .centered()
            .style(style),
        DisplayStatus::TimerInput { ref input, invalid } => {
            timer_input_line(input, invalid, display.palette)
        }
    }
}

fn timer_input_line(input: &TimerInput, invalid: bool, palette: DigitPalette) -> Line<'static> {
    let base_style = Style::default().fg(palette.status.into());
    let active_style = Style::default()
        .fg(palette.pop.into())
        .bg(Color::Rgb(39, 24, 57));
    let error_style = Style::default().fg(Color::Rgb(255, 118, 148));

    let mut spans = vec![Span::styled(" timer > ", base_style)];
    push_timer_field_span(
        &mut spans,
        input,
        TimerField::Hours,
        active_style,
        base_style,
    );
    spans.push(Span::styled(":", base_style));
    push_timer_field_span(
        &mut spans,
        input,
        TimerField::Minutes,
        active_style,
        base_style,
    );
    spans.push(Span::styled(":", base_style));
    push_timer_field_span(
        &mut spans,
        input,
        TimerField::Seconds,
        active_style,
        base_style,
    );

    if invalid {
        spans.push(Span::styled("  set a nonzero time", error_style));
    }

    Line::from(spans).centered()
}

fn push_timer_field_span(
    spans: &mut Vec<Span<'static>>,
    input: &TimerInput,
    field: TimerField,
    active_style: Style,
    base_style: Style,
) {
    let style = if input.active == field {
        active_style
    } else {
        base_style
    };
    let value = input.field_value(field);
    let segment = if input.active == field {
        format!("[{value:02}]")
    } else {
        format!(" {value:02} ")
    };

    spans.push(Span::styled(segment, style));
}

fn symbol_spans(
    before: ClockSymbol,
    after: ClockSymbol,
    row: usize,
    progress: f32,
    symbol_index: usize,
    palette: DigitPalette,
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
            palette,
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
    palette: DigitPalette,
) -> Style {
    let shimmer = ((row + column + symbol_index) % 5) as f32 * 0.035;
    let eased = ease_out_cubic((progress - shimmer).clamp(0.0, 1.0));

    let foreground = match (old, new) {
        (true, true) => blend(palette.steady, palette.bright, eased),
        (false, true) => blend(palette.entering, palette.pop, eased),
        (true, false) => blend(palette.leaving, palette.fade, eased),
        (false, false) => palette.ghost,
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

    fn from_duration(duration: Duration) -> Self {
        let seconds = duration.as_secs().min(MAX_TIMER_SECONDS);

        Self::from_hms(
            (seconds / 3600) as u32,
            ((seconds / 60) % 60) as u32,
            (seconds % 60) as u32,
        )
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
    fn duration_formats_as_hours_minutes_seconds() {
        assert_eq!(
            ClockFace::from_duration(Duration::from_secs(90)).hms(),
            (0, 1, 30)
        );
        assert_eq!(
            ClockFace::from_duration(Duration::from_secs(3661)).hms(),
            (1, 1, 1)
        );
    }

    #[test]
    fn duration_display_caps_at_two_digit_hours() {
        assert_eq!(
            ClockFace::from_duration(Duration::from_secs(MAX_TIMER_SECONDS + 1)).hms(),
            (99, 59, 59)
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
    fn unchanged_symbols_stay_steady_during_minute_transition() {
        let mut transitioning =
            DisplayState::clock(OffsetDateTime::from_unix_timestamp(60 * 60 + 35 * 60).unwrap());
        transitioning.previous = ClockFace::from_hms(1, 34, 0);
        transitioning.current = ClockFace::from_hms(1, 35, 0);
        transitioning.progress = 0.2;

        let steady =
            DisplayState::clock(OffsetDateTime::from_unix_timestamp(60 * 60 + 35 * 60).unwrap());

        assert_eq!(
            digit_row(&transitioning, 0).spans[0].style,
            digit_row(&steady, 0).spans[0].style
        );
    }

    #[test]
    fn clock_transition_highlight_stays_cool_toned() {
        let Style {
            fg: Some(Color::Rgb(red, green, blue)),
            ..
        } = cell_style(false, true, 0, 0, 0, 1.0, CLOCK_PALETTE)
        else {
            panic!("clock transition cell should use an rgb foreground");
        };

        assert!(blue > red);
        assert!(green > red);
    }

    #[test]
    fn completed_fade_uses_ghost_glyph_for_old_cells() {
        assert_eq!(cell_glyph(true, false, 0.5), CELL);
        assert_eq!(cell_glyph(true, false, 1.0), GHOST);
        assert_eq!(cell_glyph(false, true, 1.0), CELL);
    }

    #[test]
    fn structured_timer_input_edits_selected_field() {
        let mut input = TimerInput::default();

        input.push_digit('2');
        input.push_digit('5');
        assert_eq!(input.field_value(TimerField::Minutes), 25);
        assert_eq!(input.duration(), Some(Duration::from_secs(25 * 60)));

        input.move_right();
        input.push_digit('3');
        input.push_digit('0');
        assert_eq!(input.field_value(TimerField::Seconds), 30);
        assert_eq!(input.duration(), Some(Duration::from_secs(25 * 60 + 30)));

        input.move_left();
        input.move_left();
        input.push_digit('1');
        assert_eq!(input.field_value(TimerField::Hours), 1);
        assert_eq!(
            input.duration(),
            Some(Duration::from_secs(3600 + 25 * 60 + 30))
        );
    }

    #[test]
    fn structured_timer_input_clamps_minutes_and_seconds() {
        let mut input = TimerInput::default();

        input.push_digit('9');
        input.push_digit('9');
        assert_eq!(input.field_value(TimerField::Minutes), 59);

        input.move_right();
        input.push_digit('9');
        input.push_digit('9');
        assert_eq!(input.field_value(TimerField::Seconds), 59);
    }

    #[test]
    fn structured_timer_input_backspace_and_arrow_steps() {
        let mut input = TimerInput::default();

        input.push_digit('4');
        input.push_digit('2');
        input.backspace();
        assert_eq!(input.field_value(TimerField::Minutes), 4);

        input.increment_active();
        assert_eq!(input.field_value(TimerField::Minutes), 5);
        input.decrement_active();
        assert_eq!(input.field_value(TimerField::Minutes), 4);

        input.move_left();
        assert_eq!(input.active, TimerField::Hours);
        input.move_left();
        assert_eq!(input.active, TimerField::Seconds);
    }

    #[test]
    fn timer_input_line_width_stays_stable_when_marker_moves() {
        let mut input = TimerInput::default();
        input.push_digit('2');
        input.push_digit('5');

        let minutes_width = timer_input_line(&input, false, TIMER_PALETTE).width();
        input.move_left();
        let hours_width = timer_input_line(&input, false, TIMER_PALETTE).width();
        input.move_right();
        input.move_right();
        let seconds_width = timer_input_line(&input, false, TIMER_PALETTE).width();

        assert_eq!(minutes_width, hours_width);
        assert_eq!(minutes_width, seconds_width);
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
