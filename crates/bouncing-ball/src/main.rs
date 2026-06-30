use std::io;

use terminal_toy_kit::{clear_screen, draw_at, sleep_ms, Point, TerminalGuard};

fn main() -> io::Result<()> {
    let _terminal = TerminalGuard::enter()?;

    let (mut x, mut y) = (2_i16, 2_i16);
    let (mut dx, mut dy) = (1_i16, 1_i16);
    let (width, height) = (60_i16, 20_i16);

    for _ in 0..240 {
        clear_screen()?;
        draw_at(Point::new(x as u16, y as u16), "o")?;
        sleep_ms(35);

        x += dx;
        y += dy;

        if x <= 1 || x >= width {
            dx *= -1;
        }

        if y <= 1 || y >= height {
            dy *= -1;
        }
    }

    Ok(())
}
