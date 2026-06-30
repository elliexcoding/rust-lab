use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

pub fn clear_screen() -> io::Result<()> {
    print!("\x1b[2J\x1b[H");
    io::stdout().flush()
}

pub fn hide_cursor() -> io::Result<()> {
    print!("\x1b[?25l");
    io::stdout().flush()
}

pub fn show_cursor() -> io::Result<()> {
    print!("\x1b[?25h");
    io::stdout().flush()
}

pub fn draw_at(point: Point, text: &str) -> io::Result<()> {
    print!("\x1b[{};{}H{}", point.y, point.x, text);
    io::stdout().flush()
}

pub fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        hide_cursor()?;
        clear_screen()?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = show_cursor();
        let _ = clear_screen();
    }
}
