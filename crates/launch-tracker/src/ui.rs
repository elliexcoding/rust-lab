use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::{
    api::Launch,
    app::{countdown_label, countdown_progress, App},
};

const ACCENT: Color = Color::Rgb(80, 220, 255);
const WARM: Color = Color::Rgb(255, 179, 71);
const MUTED: Color = Color::Rgb(115, 135, 150);

pub fn draw(frame: &mut Frame<'_>, app: &App, now: DateTime<Utc>) {
    let page = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .split(frame.area());

    draw_header(frame, page[0], app, now);

    if app.launches.is_empty() {
        draw_empty_state(frame, page[1], app);
    } else {
        draw_dashboard(frame, page[1], app, now);
    }

    draw_footer(frame, page[2], app);
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, app: &App, now: DateTime<Utc>) {
    let activity = if app.loading { "  REFRESHING..." } else { "" };
    let timestamp = if area.width < 100 {
        now.format("%d %b %H:%M:%S UTC").to_string()
    } else {
        now.format("%a %d %b %Y  %H:%M:%S UTC").to_string()
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " ▲ ",
            Style::default().fg(WARM).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "LAUNCH TRACKER",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(activity, Style::default().fg(WARM)),
        Span::raw("   "),
        Span::styled(timestamp, Style::default().fg(MUTED)),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT)),
    );
    frame.render_widget(header, area);
}

fn draw_empty_state(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let (title, body, color) = if app.loading {
        (
            "CONTACTING MISSION CONTROL",
            "Fetching the next five launches...",
            ACCENT,
        )
    } else if let Some(message) = app.message.as_deref() {
        ("TELEMETRY UNAVAILABLE", message, Color::Red)
    } else {
        ("NO LAUNCHES", "Press r to refresh the launch feed.", MUTED)
    };

    let content = Paragraph::new(vec![
        Line::from(""),
        Line::styled(title, Style::default().fg(color).bold()),
        Line::from(""),
        Line::styled(body, Style::default().fg(Color::Gray)),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .title(" UPCOMING MISSIONS ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MUTED)),
    );
    frame.render_widget(content, area);
}

fn draw_dashboard(frame: &mut Frame<'_>, area: Rect, app: &App, now: DateTime<Utc>) {
    let horizontal = area.width >= 88;
    let panels = Layout::default()
        .direction(if horizontal {
            Direction::Horizontal
        } else {
            Direction::Vertical
        })
        .constraints(if horizontal {
            [Constraint::Percentage(42), Constraint::Percentage(58)]
        } else {
            [Constraint::Percentage(48), Constraint::Percentage(52)]
        })
        .split(area);

    draw_launch_list(frame, panels[0], app, now);
    if let Some(launch) = app.selected_launch() {
        draw_launch_detail(frame, panels[1], launch, now);
    }
}

fn draw_launch_list(frame: &mut Frame<'_>, area: Rect, app: &App, now: DateTime<Utc>) {
    let items = app.launches.iter().map(|launch| {
        let target = launch
            .t0
            .map(|t0| t0.format("%d %b %H:%M UTC").to_string())
            .unwrap_or_else(|| format!("{} (estimated)", launch.date_estimate));
        ListItem::new(vec![
            Line::styled(
                format!("  {}", launch.name),
                Style::default().fg(Color::White).bold(),
            ),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&launch.vehicle, Style::default().fg(WARM)),
                Span::styled("  /  ", Style::default().fg(MUTED)),
                Span::styled(&launch.provider, Style::default().fg(Color::Gray)),
            ]),
            Line::styled(
                format!("  {target}  {}", countdown_label(launch.t0, now)),
                Style::default().fg(MUTED),
            ),
        ])
    });

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" NEXT {} LAUNCHES ", app.launches.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(MUTED)),
        )
        .highlight_symbol("▶ ")
        .highlight_style(
            Style::default()
                .fg(ACCENT)
                .bg(Color::Rgb(18, 39, 52))
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_launch_detail(frame: &mut Frame<'_>, area: Rect, launch: &Launch, now: DateTime<Utc>) {
    let block = Block::default()
        .title(" MISSION TELEMETRY ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(5),
    ])
    .split(inner);

    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                &launch.name,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::styled("  FINAL 7-DAY COUNTDOWN", Style::default().fg(MUTED).bold()),
    ]);
    frame.render_widget(title, rows[0]);

    let progress = countdown_progress(launch.t0, now);
    let gauge_color = if progress >= 0.95 { Color::Red } else { WARM };
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color).bg(Color::Rgb(25, 32, 38)))
        .label(Span::styled(
            countdown_label(launch.t0, now),
            Style::default().fg(Color::White).bold(),
        ))
        .ratio(progress);
    frame.render_widget(gauge, rows[1]);

    let target = launch
        .t0
        .map(|t0| t0.format("%A, %d %B %Y at %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| format!("{} (estimated; exact time TBD)", launch.date_estimate));
    let weather = launch.weather.as_deref().unwrap_or("No forecast available");

    let details = Paragraph::new(vec![
        detail_line("PROVIDER", &launch.provider),
        detail_line("VEHICLE", &launch.vehicle),
        detail_line("PAD", &launch.pad),
        detail_line("LOCATION", &launch.location),
        detail_line("TARGET", &target),
        detail_line("WEATHER", weather),
        Line::from(""),
        Line::styled("MISSION", Style::default().fg(ACCENT).bold()),
        Line::styled(&launch.description, Style::default().fg(Color::Gray)),
    ])
    .wrap(Wrap { trim: true });
    frame.render_widget(details, rows[2]);
}

fn detail_line<'a>(label: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{label:<10}"), Style::default().fg(MUTED).bold()),
        Span::styled(value, Style::default().fg(Color::White)),
    ])
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let updated = app
        .last_updated
        .map(|time| format!("updated {} UTC", time.format("%H:%M:%S")))
        .unwrap_or_else(|| "not yet updated".to_string());
    let message = app
        .message
        .as_deref()
        .map(|message| format!("  |  {message}"))
        .unwrap_or_default();
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑↓/jk ", Style::default().fg(ACCENT).bold()),
        Span::styled("r:refresh ", Style::default().fg(ACCENT).bold()),
        Span::styled("q:quit ", Style::default().fg(ACCENT).bold()),
        Span::styled("Data by RocketLaunch.Live", Style::default().fg(MUTED)),
        Span::styled(format!("  {updated}{message}"), Style::default().fg(MUTED)),
    ]));
    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};
    use ratatui::{backend::TestBackend, Terminal};

    use super::*;

    #[test]
    fn empty_dashboard_renders_at_common_terminal_sizes() {
        for (width, height) in [(60, 18), (120, 36)] {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).expect("test terminal");
            let app = App::new();

            terminal
                .draw(|frame| draw(frame, &app, Utc::now()))
                .expect("render dashboard");
        }
    }

    #[test]
    fn launch_and_countdown_render_at_eighty_by_twenty_four() {
        let now = Utc.with_ymd_and_hms(2026, 8, 1, 10, 0, 0).unwrap();
        let mut app = App::new();
        app.launches.push(Launch {
            id: 1,
            name: "Test Mission".to_string(),
            provider: "Test Provider".to_string(),
            vehicle: "Test Vehicle".to_string(),
            pad: "Test Pad".to_string(),
            location: "Test Site".to_string(),
            t0: Some(now + Duration::days(3) + Duration::hours(12)),
            date_estimate: "Aug 4".to_string(),
            description: "A render test mission.".to_string(),
            weather: None,
        });
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal");

        terminal
            .draw(|frame| draw(frame, &app, now))
            .expect("render dashboard");

        let screen = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(screen.contains("FINAL 7-DAY COUNTDOWN"));
        assert!(screen.contains("T-3d 12h 00m 00s"));
    }
}
