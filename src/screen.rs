use std::{
    cmp::{max, min},
    io::stdout,
    panic,
};

use crossterm::{
    cursor::{self, SetCursorStyle},
    execute,
    style::{self, Color},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    Result,
};

use crate::buffer::Buffer;

pub struct Screen {
    // TODO: this isn't really true right b/c of splits? implement `active_buf_idx` and `buffers`
    // as Vec? Or repurpose this as window and move screen logic to new class that only does the
    // setup stuff
    buffer: Buffer,

    /// instead of storing a cursor, use the buffer's cursor but with offset for scroll
    offset: usize,

    message: String,
    message_is_error: bool,
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
            offset: 0,
            message: String::new(),
            message_is_error: false,
            command_mode_cached_cursor: None,
        };

        screen.draw()?;
        Ok(screen)
    }

    pub fn set_cursor_shape(&mut self, shape: SetCursorStyle) -> Result<()> {
        execute!(stdout(), shape)
    }

    fn reprint_cursor(&mut self) -> Result<()> {
        execute!(
            stdout(),
            cursor::MoveTo(
                self.buffer.cursor_col() as u16,
                self.buffer.cursor_row() as u16
            ),
            cursor::Show
        )
    }

    /// Moves cursor `rl` to the right (negative goes left) and `du` down if allowed
    pub fn move_cursor(&mut self, rl: isize, du: isize) -> Result<()> {
        let (old_row, old_col) = (self.buffer.cursor_row(), self.buffer.cursor_col());
        let (mut row, col) = if self.buffer.lines().is_empty() {
            (0, 0)
        } else {
            // TODO: better solution--basically subtract 1 from n if we're in normal mode but we
            // are allowed to go one further if we are in insert mode
            let normalize = |n| {
                if n == 0 {
                    0
                } else {
                    n /* - 1 */
                }
            };

            let row = min(
                self.buffer.lines().len() as isize - 1,
                self.buffer.cursor_row() as isize + du,
            );
            (
                row,
                min(
                    normalize(
                        self.buffer
                            .nth_line(max(row, 0) as usize + self.offset)
                            .len(),
                    ),
                    max(0, self.buffer.cursor_col() as isize + rl) as usize,
                ),
            )
        };
        let term_height = Screen::usable_rows() - 1;
        if row < 0 {
            if self.offset > 0 {
                let amt_under = (-row) as usize;
                if self.offset < amt_under {
                    self.offset = 0;
                } else {
                    self.offset -= amt_under;
                }
            }
            row = 0;
        } else if row as usize > term_height {
            let amt_over = row as usize - term_height;
            self.offset += amt_over;
            row -= amt_over as isize;
        }
        if row as usize != old_row || col != old_col {
            self.buffer.set_cursor(row as usize, col);
            self.reprint_cursor()?;
            self.draw()?;
        }
        Ok(())
    }

    /// Be absolutely positive this is a valid position!!
    pub fn set_cursor_col(&mut self, col: usize) -> Result<()> {
        self.buffer.set_cursor(self.buffer.cursor_row(), col);
        self.reprint_cursor()
    }

    pub fn move_cursor_end_of_line(&mut self) -> Result<()> {
        self.buffer.set_cursor(
            self.buffer.cursor_row(),
            self.buffer
                .nth_line(self.buffer.cursor_row() + self.offset)
                .len(),
        );
        self.reprint_cursor()
    }

    pub fn new_line_below(&mut self) -> Result<()> {
        self.buffer.new_line_below(self.offset);
        self.move_cursor(0, 1)
    }

    pub fn new_line_above(&mut self) -> Result<()> {
        self.buffer.new_line_above(self.offset);
        self.move_cursor(0, -1)
    }

    pub fn delete_line(&mut self) -> Result<()> {
        self.buffer.delete_line(self.offset);
        self.move_cursor(0, 0)?;
        self.draw()
    }

    pub fn change_line(&mut self) -> Result<()> {
        self.buffer.change_line(self.offset);
        self.move_cursor(0, 0)?;
        self.draw()
    }

    fn draw(&mut self) -> Result<()> {
        execute!(stdout(), cursor::Hide, cursor::MoveTo(0, 0))?;
        let mut num_lines = 0;
        for line in self
            .buffer
            .lines()
            .iter()
            .skip(self.offset)
            .take(Screen::usable_rows())
        {
            num_lines += 1;
            let padding = " ".repeat(Screen::cols() - line.len());
            execute!(
                stdout(),
                style::ResetColor,
                style::Print(format!("{line}{padding}")),
                cursor::MoveToColumn(0),
                cursor::MoveDown(1)
            )?;
        }
        for _ in 0..(Screen::usable_rows() - num_lines) {
            execute!(
                stdout(),
                style::ResetColor,
                style::Print(" ".repeat(Screen::cols())),
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
                self.buffer.cursor_row() + self.offset + 1,
                self.buffer.cursor_col() + self.offset + 1
            )
        );

        let padding = " ".repeat(Screen::cols() - (left_side.len() + right_side.len()));
        execute!(
            stdout(),
            style::SetBackgroundColor(Color::DarkGrey),
            style::Print(format!("{left_side}{padding}{right_side}"))
        )
    }

    /// For use in `draw`
    fn print_messageline(&mut self) -> Result<()> {
        let padding = " ".repeat(Screen::cols() - self.message.len());
        if self.message_is_error {
            execute!(
                stdout(),
                style::ResetColor,
                style::SetForegroundColor(Color::Red),
                style::Print(format!("{}{}", self.message, padding))
            )
        } else {
            execute!(
                stdout(),
                style::ResetColor,
                style::Print(format!("{}{}", self.message, padding))
            )
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

    pub fn load_file(&mut self, filename: String) -> Result<()> {
        self.buffer = Buffer::from_filepath(filename);
        self.draw()?;
        self.buffer.zero_cursor();
        self.offset = 0;
        self.reprint_cursor()
    }

    pub fn type_char(&mut self, c: char) -> Result<()> {
        if c == '\n' {
            self.buffer.add_line_break(self.offset);
            self.move_cursor(0, 1)?;
            self.set_cursor_col(0)?;
            self.draw()?;
        } else {
            self.buffer.add_char(c, self.offset);
            self.move_cursor(1, 0)?;
            self.draw()?;
        }

        Ok(())
    }

    pub fn delete_chars(&mut self, n: usize) -> Result<()> {
        for _ in 0..n {
            if self.buffer.cursor_col() == 0 {
                if self.buffer.cursor_row() + self.offset != 0 {
                    let new_row = self.buffer.cursor_row() + self.offset - 1;
                    let new_col = self.buffer.nth_line(new_row).len();
                    self.buffer.delete_line_break(self.offset);
                    self.move_cursor(0, -1)?;
                    self.set_cursor_col(new_col)?;
                    // TODO: technically only have to reprint all lines below the current one--is
                    // that faster or anything worthwhile?
                    self.draw()?;
                }
            } else {
                self.buffer.delete_char(self.offset);
                self.move_cursor(-1, 0)?;
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
        self.command_mode_cached_cursor =
            Some((self.buffer.cursor_row(), self.buffer.cursor_col()));
        self.buffer.set_cursor(Screen::rows(), 1);
        self.message = ":".into();
        self.message_is_error = false;
        self.draw()
    }

    pub fn leave_command_mode(&mut self) -> Result<()> {
        let (r, c) = self.command_mode_cached_cursor.unwrap();
        self.command_mode_cached_cursor = None;
        self.buffer.set_cursor(r, c);
        if self.message.starts_with(':') {
            self.message = "".into();
        }
        self.draw()
    }

    // TODO: it's weird to me that cursor is still stored in buffer; if we had an additional cursor
    // object attached to screen only for use in command mode we wouldn't need to cache the old one
    // either
    // actually for that matter cursor shouldn't be attacked to a buffer but rather a window. think
    // like if there are multiple splits with the same buffer.
    pub fn command_move_cursor(&mut self, rl: isize) -> Result<()> {
        let new_col = self.buffer.cursor_col() as isize + rl;
        if new_col < 1 {
            self.buffer.set_cursor(self.buffer.cursor_row(), 1);
        } else if new_col as usize > self.message.len() {
            self.buffer
                .set_cursor(self.buffer.cursor_row(), self.message.len());
        } else {
            self.buffer
                .set_cursor(self.buffer.cursor_row(), new_col as usize);
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
