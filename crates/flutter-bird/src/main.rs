use std::io;

use terminal_toy_kit::{draw_at, sleep_ms, Point, TerminalGuard};

const FRAMES: [&str; 4] = ["<(o )___", "<(o )~~~", "___( o)>", "~~~( o)>"];

fn main() -> io::Result<()> {
    let _terminal = TerminalGuard::enter()?;

    for tick in 0..160 {
        let x = 4 + (tick % 48) as u16;
        let y = 8 + ((tick as f32 / 4.0).sin() * 3.0) as u16;
        let frame = FRAMES[tick % FRAMES.len()];

        draw_at(
            Point::new(1, y),
            "                                                            ",
        )?;
        draw_at(Point::new(x, y), frame)?;
        sleep_ms(60);
    }

    Ok(())
}
