use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration as StdDuration, Instant},
};

use chrono::{DateTime, Duration, Utc};

use crate::api::{fetch_launches, Launch};

const AUTO_REFRESH_INTERVAL: StdDuration = StdDuration::from_secs(5 * 60);
const COUNTDOWN_HORIZON: Duration = Duration::days(7);

type FetchResult = Result<Vec<Launch>, String>;

pub struct App {
    pub launches: Vec<Launch>,
    pub selected: usize,
    pub loading: bool,
    pub last_updated: Option<DateTime<Utc>>,
    pub message: Option<String>,
    receiver: Option<Receiver<FetchResult>>,
    last_fetch_started: Option<Instant>,
}

impl App {
    pub fn new() -> Self {
        Self {
            launches: Vec::new(),
            selected: 0,
            loading: false,
            last_updated: None,
            message: None,
            receiver: None,
            last_fetch_started: None,
        }
    }

    pub fn begin_refresh(&mut self) {
        if self.loading {
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.loading = true;
        self.message = None;
        self.receiver = Some(receiver);
        self.last_fetch_started = Some(Instant::now());

        thread::spawn(move || {
            let result = fetch_launches().map_err(|error| format!("{error:#}"));
            let _ = sender.send(result);
        });
    }

    pub fn poll_refresh(&mut self, now: DateTime<Utc>) {
        let Some(receiver) = self.receiver.as_ref() else {
            return;
        };

        match receiver.try_recv() {
            Ok(Ok(launches)) => {
                let selected_id = self.selected_launch().map(|launch| launch.id);
                self.launches = launches;
                self.selected = selected_id
                    .and_then(|id| self.launches.iter().position(|launch| launch.id == id))
                    .unwrap_or(0)
                    .min(self.launches.len().saturating_sub(1));
                self.loading = false;
                self.last_updated = Some(now);
                self.message = self
                    .launches
                    .is_empty()
                    .then(|| "The launch feed returned no missions".to_string());
                self.receiver = None;
            }
            Ok(Err(error)) => {
                self.loading = false;
                self.message = Some(error);
                self.receiver = None;
            }
            Err(TryRecvError::Disconnected) => {
                self.loading = false;
                self.message = Some("The refresh worker stopped unexpectedly".to_string());
                self.receiver = None;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    pub fn maybe_auto_refresh(&mut self) {
        let refresh_due = self
            .last_fetch_started
            .is_none_or(|started| started.elapsed() >= AUTO_REFRESH_INTERVAL);
        if !self.loading && refresh_due {
            self.begin_refresh();
        }
    }

    pub fn selected_launch(&self) -> Option<&Launch> {
        self.launches.get(self.selected)
    }

    pub fn select_next(&mut self) {
        if !self.launches.is_empty() {
            self.selected = (self.selected + 1) % self.launches.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.launches.is_empty() {
            self.selected = self
                .selected
                .checked_sub(1)
                .unwrap_or(self.launches.len() - 1);
        }
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    pub fn select_last(&mut self) {
        self.selected = self.launches.len().saturating_sub(1);
    }
}

pub fn countdown_label(t0: Option<DateTime<Utc>>, now: DateTime<Utc>) -> String {
    let Some(t0) = t0 else {
        return "TBD - awaiting an exact launch time".to_string();
    };

    let remaining = t0 - now;
    if remaining >= Duration::zero() {
        format!("T-{}", format_duration(remaining))
    } else {
        format!("T+{}", format_duration(-remaining))
    }
}

pub fn countdown_progress(t0: Option<DateTime<Utc>>, now: DateTime<Utc>) -> f64 {
    let Some(t0) = t0 else {
        return 0.0;
    };

    let remaining = t0 - now;
    if remaining <= Duration::zero() {
        return 1.0;
    }

    let remaining_seconds = remaining.num_milliseconds() as f64 / 1_000.0;
    let horizon_seconds = COUNTDOWN_HORIZON.num_seconds() as f64;
    (1.0 - remaining_seconds / horizon_seconds).clamp(0.0, 1.0)
}

fn format_duration(duration: Duration) -> String {
    let total = duration.num_seconds().max(0);
    let days = total / 86_400;
    let hours = (total % 86_400) / 3_600;
    let minutes = (total % 3_600) / 60;
    let seconds = total % 60;

    if days > 0 {
        format!("{days}d {hours:02}h {minutes:02}m {seconds:02}s")
    } else {
        format!("{hours:02}h {minutes:02}m {seconds:02}s")
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn countdown_formats_future_past_and_unknown_targets() {
        let now = Utc.with_ymd_and_hms(2026, 8, 1, 10, 0, 0).unwrap();

        assert_eq!(
            countdown_label(
                Some(now + Duration::days(2) + Duration::seconds(3_661)),
                now
            ),
            "T-2d 01h 01m 01s"
        );
        assert_eq!(
            countdown_label(Some(now - Duration::seconds(90)), now),
            "T+00h 01m 30s"
        );
        assert_eq!(
            countdown_label(None, now),
            "TBD - awaiting an exact launch time"
        );
    }

    #[test]
    fn progress_fills_over_the_final_seven_days() {
        let t0 = Utc.with_ymd_and_hms(2026, 8, 8, 10, 0, 0).unwrap();

        assert_eq!(countdown_progress(Some(t0), t0 - Duration::days(8)), 0.0);
        assert!(
            (countdown_progress(Some(t0), t0 - Duration::days(3) - Duration::hours(12)) - 0.5)
                .abs()
                < f64::EPSILON
        );
        assert_eq!(countdown_progress(Some(t0), t0), 1.0);
        assert_eq!(countdown_progress(Some(t0), t0 + Duration::hours(1)), 1.0);
        assert_eq!(countdown_progress(None, t0), 0.0);
    }
}
