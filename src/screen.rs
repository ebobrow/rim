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
    windows: Vec<Window>,
    cur_window: usize,

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
            windows: vec![Window::new(Screen::usable_rows(), Screen::cols(), (0, 0))],
            cur_window: 0,
            command_mode_cursor: None,
            message: String::new(),
            message_is_error: false,
        };

        screen.draw()?;
        Ok(screen)
    }

    pub fn active_window(&mut self) -> &mut Window {
        // TODO: error handling
        &mut self.windows[self.cur_window]
    }

    pub fn new_vertical_split(&mut self) -> Result<()> {
        let half_width = self.active_window().width() / 2;
        let (width_a, width_b) = if self.active_window().width() % 2 == 0 {
            (half_width, half_width)
        } else {
            (half_width, half_width + 1)
        };
        self.active_window().set_width(width_a);
        let new_window = Window::new(
            self.active_window().height(),
            // TODO: vertical bar; `new_width - 1`
            width_b,
            (
                self.active_window().loc().0,
                self.active_window().loc().1 + width_a,
            ),
        );
        self.windows.push(new_window);
        self.cur_window = self.windows.len() - 1;
        self.draw()
    }

    pub fn new_horizontal_split(&mut self) -> Result<()> {
        let half_height = self.active_window().height() / 2;
        let (height_a, height_b) = if self.active_window().height() % 2 == 0 {
            (half_height, half_height)
        } else {
            (half_height, half_height + 1)
        };
        self.active_window().set_height(height_a);
        let new_window = Window::new(
            height_b - 1,
            self.active_window().width(),
            (
                self.active_window().loc().0 + height_a + 1,
                self.active_window().loc().1,
            ),
        );
        self.windows.push(new_window);
        self.cur_window = self.windows.len() - 1;
        self.draw()
    }

    // TODO: lots of redundant logic here
    pub fn move_to_left_window(&mut self) -> Result<()> {
        let active_loc = self.active_window().loc();
        if let Some((i, _)) = self
            .windows
            .iter()
            .enumerate()
            .filter(|(_, window)| window.loc().1 + window.width() == active_loc.1)
            .min_by_key(|(_, window)| window.loc().0.abs_diff(active_loc.0))
        {
            self.cur_window = i;
            self.reprint_cursor()?;
        }
        Ok(())
    }

    pub fn move_to_right_window(&mut self) -> Result<()> {
        let active_loc = self.active_window().loc();
        let active_width = self.active_window().width();
        if let Some((i, _)) = self
            .windows
            .iter()
            .enumerate()
            .filter(|(_, window)| window.loc().1 == active_loc.1 + active_width)
            .min_by_key(|(_, window)| window.loc().0.abs_diff(active_loc.0))
        {
            self.cur_window = i;
            self.reprint_cursor()?;
        }
        Ok(())
    }

    pub fn move_to_up_window(&mut self) -> Result<()> {
        let active_loc = self.active_window().loc();
        if let Some((i, _)) = self
            .windows
            .iter()
            .enumerate()
            .filter(|(_, window)| window.loc().0 + window.height() + 1 == active_loc.0)
            .min_by_key(|(_, window)| window.loc().1.abs_diff(active_loc.1))
        {
            self.cur_window = i;
            self.reprint_cursor()?;
        }
        Ok(())
    }

    pub fn move_to_down_window(&mut self) -> Result<()> {
        let active_loc = self.active_window().loc();
        let active_height = self.active_window().height() + 1;
        if let Some((i, _)) = self
            .windows
            .iter()
            .enumerate()
            .filter(|(_, window)| window.loc().0 == active_loc.0 + active_height)
            .min_by_key(|(_, window)| window.loc().1.abs_diff(active_loc.1))
        {
            self.cur_window = i;
            self.reprint_cursor()?;
        }
        Ok(())
    }

    pub fn set_cursor_shape(&mut self, shape: SetCursorStyle) -> Result<()> {
        execute!(stdout(), shape)
    }

    fn draw(&mut self) -> Result<()> {
        for window in &mut self.windows {
            window.draw()?;
        }
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
            self.active_window().reprint_cursor()
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
        match self.active_window().write() {
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
