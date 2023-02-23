use std::io::stdout;

use crossterm::{
    cursor, event, execute, style,
    terminal::{self, enable_raw_mode},
    Result,
};

// TODO list:
// - Data structure for keybinds, so that no nested match tree and also user customization
// - Edit modes (for now, Normal, Insert, and Command)
// - Data structure for text so that you aren't allowed to move cursor off of text
// - Scroll

fn main() -> Result<()> {
    enable_raw_mode()?;

    execute!(
        stdout(),
        // style::SetBackgroundColor(style::Color::DarkGrey),
        terminal::EnterAlternateScreen,
        cursor::MoveTo(0, 0),
        style::Print("hey"),
    )?;

    // execute!(
    //     stdout(),
    //     SetForegroundColor(Color::Blue),
    //     SetBackgroundColor(Color::Red),
    //     Print("Styled text here."),
    //     ResetColor
    // )?;

    loop {
        match event::read()? {
            event::Event::Key(key_event) => {
                // TODO: data structure with all these
                if let event::KeyCode::Char(c) = key_event.code {
                    match c {
                        'h' => execute!(stdout(), cursor::MoveLeft(1))?,
                        'j' => execute!(stdout(), cursor::MoveDown(1))?,
                        'k' => execute!(stdout(), cursor::MoveUp(1))?,
                        'l' => execute!(stdout(), cursor::MoveRight(1))?,
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

    execute!(stdout(), terminal::LeaveAlternateScreen)
}
