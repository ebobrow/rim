use crossterm::{event, Result};
use screen::Screen;

mod buffer;
mod screen;

// TODO list:
// [ ] Data structure for keybinds, so that no nested match tree and also user customization
// [ ] Edit modes (for now, Normal, Insert, and Command)
// [x] Data structure for text so that you aren't allowed to move cursor off of text
// [ ] Scroll
// [ ] Files (async for stream stuff?)

fn main() -> Result<()> {
    let mut screen = Screen::new()?;

    screen.load_text("this is a file\n\nwith\nmultiple lines")?;

    loop {
        match event::read()? {
            event::Event::Key(key_event) => {
                // TODO: data structure with all these
                if let event::KeyCode::Char(c) = key_event.code {
                    match c {
                        'h' => screen.move_cursor(-1, 0)?,
                        'j' => screen.move_cursor(0, 1)?,
                        'k' => screen.move_cursor(0, -1)?,
                        'l' => screen.move_cursor(1, 0)?,
                        'c' => {
                            if key_event.modifiers == event::KeyModifiers::CONTROL {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    screen.finish()
}
