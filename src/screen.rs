use std::{io::stdout, panic};

use crossterm::{
    cursor::{self, SetCursorStyle},
    execute,
    style::{self, Color},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    Result,
};

use crate::window::Window;

pub struct Screen {
    // TODO: once we have splits there will be multiples in some cool data structure
    windows: Window,

    command_mode_cursor: Option<usize>,

    message: String,
    message_is_error: bool,
}

impl Screen {
    fn setup() -> Result<()> {
        enable_raw_mode()?;
        panic::set_hook(Box::new(|info| {
            Self::finish().unwrap();
            eprintln!("{info}");
        }));

        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            cursor::MoveTo(0, 0),
            cursor::SetCursorStyle::SteadyBlock,
        )
    }

    pub fn finish() -> Result<()> {
        disable_raw_mode()?;
        execute!(stdout(), terminal::LeaveAlternateScreen)
    }

    pub fn new() -> Result<Self> {
        Self::setup()?;

        let mut screen = Self {
            windows: Window::new(Screen::usable_rows(), Screen::cols()),
            command_mode_cursor: None,
            message: String::new(),
            message_is_error: false,
        };

        screen.draw()?;
        Ok(screen)
    }

    pub fn active_window(&mut self) -> &mut Window {
        &mut self.windows
    }

    pub fn set_cursor_shape(&mut self, shape: SetCursorStyle) -> Result<()> {
        execute!(stdout(), shape)
    }

    fn draw(&mut self) -> Result<()> {
        self.windows.draw()?;
        self.print_messageline()?;
        self.reprint_cursor()
    }

    fn reprint_cursor(&mut self) -> Result<()> {
        if let Some(col) = self.command_mode_cursor {
            execute!(
                stdout(),
                cursor::MoveTo(col as u16, Screen::rows() as u16),
                cursor::Show
            )
        } else {
            self.windows.reprint_cursor()
        }
    }

    /// For use in `draw`
    fn print_messageline(&mut self) -> Result<()> {
        execute!(stdout(), cursor::MoveTo(0, Screen::rows() as u16))?;
        let formatted_message = if self.message.len() > Screen::cols() {
            self.message[..Screen::cols()].to_owned()
        } else {
            let padding = " ".repeat(Screen::cols() - self.message.len());
            format!("{}{}", self.message, padding)
        };
        if self.message_is_error {
            execute!(
                stdout(),
                style::ResetColor,
                style::SetForegroundColor(Color::Red),
                style::Print(formatted_message)
            )
        } else {
            execute!(stdout(), style::ResetColor, style::Print(formatted_message))
        }
    }

    /// Usable on its own
    fn reprint_messageline(&mut self) -> Result<()> {
        self.print_messageline()?;
        self.reprint_cursor()
    }

    fn cols() -> usize {
        terminal::size().unwrap().0 as usize
    }

    fn rows() -> usize {
        terminal::size().unwrap().1 as usize
    }

    fn usable_rows() -> usize {
        // Status bar and messages
        Self::rows() - 2
    }

    pub fn write(&mut self) -> Result<()> {
        match self.windows.write() {
            Ok(msg) => self.set_message(msg),
            Err(e) => self.set_error_message(e),
        }
    }

    pub fn set_message(&mut self, message: impl ToString) -> Result<()> {
        self.message = message.to_string();
        self.message_is_error = false;
        self.draw()
    }

    pub fn set_error_message(&mut self, message: impl ToString) -> Result<()> {
        self.message = message.to_string();
        self.message_is_error = true;
        self.draw()
    }

    pub fn enter_command_mode(&mut self) -> Result<()> {
        self.command_mode_cursor = Some(1);
        self.message = ":".into();
        self.message_is_error = false;
        self.draw()
    }

    pub fn leave_command_mode(&mut self) -> Result<()> {
        self.command_mode_cursor = None;
        if self.message.starts_with(':') {
            self.message = "".into();
        }
        self.draw()
    }

    pub fn command_move_cursor(&mut self, rl: isize) -> Result<()> {
        let old_col = self.command_mode_cursor.expect("is in command mode");
        let new_col = old_col as isize + rl;
        if new_col < 1 {
            self.command_mode_cursor = Some(1);
        } else if new_col as usize > self.message.len() {
            self.command_mode_cursor = Some(self.message.len());
        } else {
            self.command_mode_cursor = Some(new_col as usize);
        }
        self.reprint_cursor()
    }

    pub fn command_type_char(&mut self, c: char) -> Result<()> {
        self.message.push(c);
        self.command_move_cursor(1)?;
        self.reprint_messageline()
    }

    pub fn command_delete_char(&mut self) -> Result<()> {
        if self.message.len() == 1 {
            // TODO: leave command mode (how to change state)
        } else {
            self.message.remove(self.message.len() - 1);
            self.command_move_cursor(-1)?;
        }
        self.reprint_messageline()
    }

    pub fn get_curr_command(&self) -> &str {
        &self.message[1..]
    }
}
