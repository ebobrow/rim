use std::{cmp::min, io::stdout, panic};

use crossterm::{
    cursor::{self, SetCursorStyle},
    execute,
    style::{self, Color},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    Result,
};

const SIDEBAR_LEN: usize = 4;

use crate::buffer::Buffer;

pub struct Screen {
    // TODO: this isn't really true right b/c of splits? implement `active_buf_idx` and `buffers`
    // as Vec? Or repurpose this as window and move screen logic to new class that only does the
    // setup stuff
    buffer: Buffer,

    /// (row, col) relative to screen
    cursor: (usize, usize),
    offset: (usize, usize),

    message: String,
    message_is_error: bool,
    // TODO: this is weird right?
    command_mode_cached_cursor: Option<(usize, usize)>,
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
            // TODO: centered info screen
            buffer: Buffer::from_string(String::new()),
            cursor: (0, 0),
            offset: (0, 0),
            message: String::new(),
            message_is_error: false,
            command_mode_cached_cursor: None,
        };

        screen.draw()?;
        Ok(screen)
    }

    fn cursor_row(&self) -> usize {
        self.cursor.0
    }

    fn cursor_col(&self) -> usize {
        self.cursor.1
    }

    fn offset_row(&self) -> usize {
        self.offset.0
    }

    fn offset_col(&self) -> usize {
        self.offset.1
    }

    fn adjusetd_cursor(&self) -> (usize, usize) {
        (
            self.cursor_row() + self.offset_row(),
            self.cursor_col() + self.offset_col(),
        )
    }

    pub fn set_cursor_shape(&mut self, shape: SetCursorStyle) -> Result<()> {
        execute!(stdout(), shape)
    }

    fn reprint_cursor(&mut self) -> Result<()> {
        let col = if self.command_mode_cached_cursor.is_some() {
            self.cursor_col()
        } else {
            self.cursor_col() + SIDEBAR_LEN + 1
        };
        execute!(
            stdout(),
            cursor::MoveTo(col as u16, self.cursor_row() as u16),
            cursor::Show
        )
    }

    /// Moves cursor `du` down (negative goes up) if allowed
    pub fn move_cursor_row(&mut self, du: isize) -> Result<()> {
        let old_row = self.cursor_row();
        let old_offset = self.offset_row();
        let mut new_row = if self.buffer.lines().is_empty() {
            0
        } else {
            min(
                self.buffer.lines().len() as isize - 1,
                self.cursor_row() as isize + du,
            )
        };
        let term_height = Screen::usable_rows() - 1;
        if new_row < 0 {
            if self.offset_row() > 0 {
                let amt_under = (-new_row) as usize;
                if self.offset_row() < amt_under {
                    self.offset.0 = 0;
                } else {
                    self.offset.0 -= amt_under;
                }
            }
            new_row = 0;
        } else if new_row as usize > term_height {
            let amt_over = new_row as usize - term_height;
            self.offset.0 += amt_over;
            new_row -= amt_over as isize;
        }
        if new_row as usize != old_row || self.offset_row() != old_offset {
            self.set_cursor_row(new_row as usize)?;
            self.draw()?;
        }
        self.move_cursor_col(0)
    }

    /// Moves cursor `rl` to the right (negative goes left)
    pub fn move_cursor_col(&mut self, rl: isize) -> Result<()> {
        let old_col = self.cursor_col();
        let old_offset = self.offset_col();
        let mut new_col = if self.buffer.lines().is_empty() {
            0
        } else {
            let line_len = self
                .buffer
                .nth_line(self.cursor_row() + self.offset_row())
                .len() as isize
                - self.offset_col() as isize;
            // TODO: subtract 1 from n if we're in normal mode but we are allowed to go one further
            // if we are in insert mode
            min(line_len, self.cursor_col() as isize + rl)
        };
        let term_width = Screen::usable_cols() - 1;
        if new_col < 0 {
            if self.offset_col() > 0 {
                let amt_under = (-new_col) as usize;
                if self.offset_col() < amt_under {
                    self.offset.1 = 0;
                } else {
                    self.offset.1 -= amt_under;
                }
            }
            new_col = 0;
        } else if new_col as usize > term_width {
            let amt_over = new_col as usize - term_width;
            self.offset.1 += amt_over;
            new_col -= amt_over as isize;
        }
        if new_col as usize != old_col || self.offset_col() != old_offset {
            self.set_cursor_col(new_col as usize)?;
            self.draw()?;
        }
        Ok(())
    }

    fn validate_cursor(&mut self) -> Result<()> {
        self.move_cursor_row(0)
    }

    pub fn set_cursor_row(&mut self, row: usize) -> Result<()> {
        self.cursor.0 = row;
        self.validate_cursor()?;
        self.draw()
    }

    pub fn zero_cursor_col(&mut self) -> Result<()> {
        self.move_cursor_col(0 - self.offset_col() as isize - self.cursor_col() as isize)
    }

    pub fn set_cursor_col(&mut self, col: usize) -> Result<()> {
        self.cursor.1 = col;
        self.validate_cursor()?;
        self.draw()
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) -> Result<()> {
        self.cursor = (row, col);
        self.validate_cursor()?;
        self.draw()
    }

    pub fn move_cursor_end_of_line(&mut self) -> Result<()> {
        self.cursor.1 = self
            .buffer
            .nth_line(self.cursor_row() + self.offset_row())
            .len();
        self.validate_cursor()
    }

    pub fn new_line_below(&mut self) -> Result<()> {
        self.buffer.new_line_below(self.adjusetd_cursor());
        self.move_cursor_row(1)
    }

    pub fn new_line_above(&mut self) -> Result<()> {
        self.buffer.new_line_above(self.adjusetd_cursor());
        self.move_cursor_row(-1)
    }

    pub fn delete_line(&mut self) -> Result<()> {
        self.buffer.delete_line(self.adjusetd_cursor());
        self.validate_cursor()?;
        self.draw()
    }

    pub fn change_line(&mut self) -> Result<()> {
        self.buffer.change_line(self.adjusetd_cursor());
        self.validate_cursor()?;
        self.draw()
    }

    fn draw(&mut self) -> Result<()> {
        execute!(
            stdout(),
            cursor::Hide,
            cursor::MoveTo(0, 0),
            style::ResetColor,
        )?;
        let mut num_lines = 0;
        for line in self
            .buffer
            .lines()
            .iter()
            .skip(self.offset_row())
            .take(Screen::usable_rows())
        {
            num_lines += 1;
            let formatted_line = if line.len() < self.offset_col() {
                " ".repeat(Screen::usable_cols())
            } else if line[self.offset_col()..].len() > Screen::usable_cols() {
                line[self.offset_col()..Screen::usable_cols()].to_owned()
            } else {
                let padding = " ".repeat(Screen::usable_cols() - line[self.offset_col()..].len());
                format!("{}{padding}", &line[self.offset_col()..])
            };
            let linenum = format!("{}", self.offset_row() + num_lines);
            let linenum_padding = " ".repeat(SIDEBAR_LEN - linenum.len());
            execute!(
                stdout(),
                style::SetForegroundColor(Color::DarkGrey),
                style::Print(format!("{linenum_padding}{linenum} ")),
                style::ResetColor,
                style::Print(formatted_line),
                cursor::MoveToColumn(0),
                cursor::MoveDown(1)
            )?;
        }
        for _ in 0..(Screen::usable_rows() - num_lines) {
            execute!(
                stdout(),
                style::SetForegroundColor(Color::DarkGrey),
                style::Print(format!("~{}", " ".repeat(Screen::cols() - 1))),
                cursor::MoveToColumn(0),
                cursor::MoveDown(1)
            )?;
        }
        self.print_statusline()?;
        self.print_messageline()?;
        self.reprint_cursor()
    }

    fn print_statusline(&mut self) -> Result<()> {
        let left_side = format!(
            "{name}{save_marker}",
            name = self.buffer.filename(),
            save_marker = if self.buffer.unsaved_changes() {
                " [+]"
            } else {
                ""
            }
        );
        let right_side = format!(
            "{cursor_loc}",
            cursor_loc = format!(
                "{}:{}",
                self.cursor_row() + self.offset_row() + 1,
                self.cursor_col() + self.offset_row() + 1
            )
        );

        let padding = " ".repeat(Screen::cols() - (left_side.len() + right_side.len()));
        execute!(
            stdout(),
            style::ResetColor,
            style::SetBackgroundColor(Color::DarkGrey),
            style::Print(format!("{left_side}{padding}{right_side}"))
        )
    }

    /// For use in `draw`
    fn print_messageline(&mut self) -> Result<()> {
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
        execute!(stdout(), cursor::MoveTo(0, Screen::rows() as u16))?;
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

    fn usable_cols() -> usize {
        Self::cols() - SIDEBAR_LEN - 1
    }

    pub fn load_file(&mut self, filename: String) -> Result<()> {
        self.buffer = Buffer::from_filepath(filename);
        self.draw()?;
        self.cursor = (0, 0);
        self.offset = (0, 0);
        self.reprint_cursor()
    }

    pub fn type_char(&mut self, c: char) -> Result<()> {
        if c == '\n' {
            self.buffer.add_line_break(self.adjusetd_cursor());
            self.move_cursor_row(1)?;
            self.zero_cursor_col()?;
            self.draw()?;
        } else {
            self.buffer.add_char(c, self.adjusetd_cursor());
            self.move_cursor_col(1)?;
            self.draw()?;
        }

        Ok(())
    }

    pub fn delete_chars(&mut self, n: usize) -> Result<()> {
        for _ in 0..n {
            if self.cursor_col() == 0 {
                if self.cursor_row() + self.offset_row() != 0 {
                    let new_row = self.cursor_row() + self.offset_row() - 1;
                    let new_col = self.buffer.nth_line(new_row).len();
                    self.buffer.delete_line_break(self.adjusetd_cursor());
                    self.move_cursor_row(-1)?;
                    self.set_cursor_col(new_col)?;
                    // TODO: technically only have to reprint all lines below the current one--is
                    // that faster or anything worthwhile?
                    self.draw()?;
                }
            } else {
                self.buffer.delete_char(self.adjusetd_cursor());
                self.move_cursor_col(-1)?;
            }
        }
        self.draw()
    }

    pub fn write(&mut self) -> Result<()> {
        match self.buffer.write() {
            Ok(()) => self.set_message(format!("\"{}\" written", self.buffer.filename())),
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
        self.command_mode_cached_cursor = Some((self.cursor_row(), self.cursor_col()));
        self.cursor = (Screen::rows(), 1);
        self.message = ":".into();
        self.message_is_error = false;
        self.draw()
    }

    pub fn leave_command_mode(&mut self) -> Result<()> {
        let (r, c) = self.command_mode_cached_cursor.unwrap();
        self.command_mode_cached_cursor = None;
        self.set_cursor(r, c)?;
        if self.message.starts_with(':') {
            self.message = "".into();
        }
        self.draw()
    }

    pub fn command_move_cursor(&mut self, rl: isize) -> Result<()> {
        let new_col = self.cursor_col() as isize + rl;
        if new_col < 1 {
            self.cursor.1 = 1;
        } else if new_col as usize > self.message.len() {
            self.cursor.1 = self.message.len();
        } else {
            self.cursor.1 = new_col as usize;
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
