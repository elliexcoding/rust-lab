use std::io;

use terminal_toy_kit::{clear_screen, draw_at, sleep_ms, Point, TerminalGuard};

fn main() -> io::Result<()> {
    let _terminal = TerminalGuard::enter()?;
    let glyphs = ['0', '1', '+', '*', '#', '@'];

    for tick in 0..180 {
        clear_screen()?;

        for x in 1..70 {
            let y = 1 + ((tick + x * 3) % 22) as u16;
            let glyph = glyphs[(tick + x) % glyphs.len()];
            draw_at(Point::new(x as u16, y), &glyph.to_string())?;
        }

        sleep_ms(50);
    }

    Ok(())
}
