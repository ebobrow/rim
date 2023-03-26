use std::{cmp::min, io::stdout};

use crossterm::{
    cursor, execute,
    style::{self, Color},
    terminal, Result as CResult,
};

use crate::buffer::Buffer;

const SIDEBAR_LEN: usize = 4;

pub struct Window {
    buffer: Buffer,

    /// (row, col) relative to screen
    cursor: (usize, usize),
    offset: (usize, usize),

    /// top left corner
    loc: (usize, usize),
    height: usize,
    width: usize,
}

impl Window {
    pub fn new(height: usize, width: usize, loc: (usize, usize)) -> Self {
        Self {
            // TODO: centered info screen
            buffer: Buffer::from_string(String::new()),
            cursor: (0, 0),
            offset: (0, 0),
            height,
            width,
            loc,
        }
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

    fn usable_cols(&self) -> usize {
        self.width - SIDEBAR_LEN - 1
    }

    pub fn reprint_cursor(&self) -> CResult<()> {
        let row = self.cursor_row() + self.loc.0;
        let col = self.cursor_col() + self.loc.1 + SIDEBAR_LEN + 1;
        execute!(
            stdout(),
            cursor::MoveTo(col as u16, row as u16),
            cursor::Show
        )
    }

    /// Moves cursor `du` down (negative goes up) if allowed
    pub fn move_cursor_row(&mut self, du: isize) -> CResult<()> {
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
        let term_height = self.height - 1;
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
            self.redraw()?;
        }
        self.move_cursor_col(0)
    }

    /// Moves cursor `rl` to the right (negative goes left)
    pub fn move_cursor_col(&mut self, rl: isize) -> CResult<()> {
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
        let term_width = self.usable_cols() - 1;
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
            self.redraw()?;
        }
        Ok(())
    }

    fn validate_cursor(&mut self) -> CResult<()> {
        self.move_cursor_row(0)
    }

    pub fn set_cursor_row(&mut self, row: usize) -> CResult<()> {
        self.cursor.0 = row;
        self.validate_cursor()?;
        self.redraw()
    }

    pub fn zero_cursor_col(&mut self) -> CResult<()> {
        self.move_cursor_col(0 - self.offset_col() as isize - self.cursor_col() as isize)
    }

    pub fn set_cursor_col(&mut self, col: usize) -> CResult<()> {
        self.cursor.1 = col;
        self.validate_cursor()?;
        self.redraw()
    }

    pub fn move_cursor_end_of_line(&mut self) -> CResult<()> {
        self.cursor.1 = self
            .buffer
            .nth_line(self.cursor_row() + self.offset_row())
            .len();
        self.validate_cursor()?;
        self.redraw()
    }

    pub fn new_line_below(&mut self) -> CResult<()> {
        self.buffer.new_line_below(self.adjusetd_cursor());
        self.move_cursor_row(1)
    }

    pub fn new_line_above(&mut self) -> CResult<()> {
        self.buffer.new_line_above(self.adjusetd_cursor());
        self.move_cursor_row(-1)
    }

    pub fn delete_line(&mut self) -> CResult<()> {
        self.buffer.delete_line(self.adjusetd_cursor());
        self.validate_cursor()?;
        self.redraw()
    }

    pub fn change_line(&mut self) -> CResult<()> {
        self.buffer.change_line(self.adjusetd_cursor());
        self.validate_cursor()?;
        self.redraw()
    }

    pub fn draw(&self) -> CResult<()> {
        execute!(
            stdout(),
            cursor::Hide,
            cursor::MoveTo(self.loc.1 as u16, self.loc.0 as u16),
            style::ResetColor,
        )?;
        let mut num_lines = 0;
        for line in self
            .buffer
            .lines()
            .iter()
            .skip(self.offset_row())
            .take(self.height)
        {
            num_lines += 1;
            let formatted_line = if line.len() < self.offset_col() {
                " ".repeat(self.usable_cols() - 1)
            } else if line[self.offset_col()..].len() > self.usable_cols() {
                line[self.offset_col()..self.usable_cols() - 1].to_owned()
            } else {
                let padding = " ".repeat(self.usable_cols() - 1 - line[self.offset_col()..].len());
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
                cursor::MoveToColumn(self.loc.1 as u16),
                cursor::MoveDown(1)
            )?;
        }
        for _ in 0..(self.height - num_lines) {
            execute!(
                stdout(),
                style::SetForegroundColor(Color::DarkGrey),
                style::Print(format!("~{}", " ".repeat(self.width - 2))),
                cursor::MoveToColumn(self.loc.1 as u16),
                cursor::MoveDown(1)
            )?;
        }
        self.print_statusline()
    }

    fn redraw(&self) -> CResult<()> {
        self.draw()?;
        self.reprint_cursor()
    }

    fn print_statusline(&self) -> CResult<()> {
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

        let padding = " ".repeat(self.width - (left_side.len() + right_side.len()));
        execute!(
            stdout(),
            style::ResetColor,
            style::SetBackgroundColor(Color::DarkGrey),
            style::Print(format!("{left_side}{padding}{right_side}"))
        )
    }

    pub fn print_divider(&self) -> CResult<()> {
        execute!(
            stdout(),
            cursor::MoveTo((self.loc.1 + self.width - 1) as u16, self.loc.0 as u16),
            style::SetForegroundColor(Color::Black),
            style::SetBackgroundColor(Color::DarkGrey)
        )?;
        for _ in 0..self.height {
            execute!(
                stdout(),
                style::Print("|"),
                cursor::MoveDown(1),
                cursor::MoveLeft(1)
            )?;
        }
        Ok(())
    }

    pub fn load_file(&mut self, filename: String) -> CResult<()> {
        self.buffer = Buffer::from_filepath(filename);
        self.redraw()?;
        self.cursor = (0, 0);
        self.offset = (0, 0);
        self.reprint_cursor()
    }

    pub fn type_char(&mut self, c: char) -> CResult<()> {
        if c == '\n' {
            self.buffer.add_line_break(self.adjusetd_cursor());
            self.move_cursor_row(1)?;
            self.zero_cursor_col()?;
            self.redraw()?;
        } else {
            self.buffer.add_char(c, self.adjusetd_cursor());
            self.move_cursor_col(1)?;
            self.redraw()?;
        }

        Ok(())
    }

    pub fn delete_chars(&mut self, n: usize) -> CResult<()> {
        for _ in 0..n {
            if self.cursor_col() == 0 {
                if self.cursor_row() + self.offset_row() != 0 {
                    let new_row = self.cursor_row() + self.offset_row() - 1;
                    let new_col = self.buffer.nth_line(new_row).len();
                    self.buffer.delete_line_break(self.adjusetd_cursor());
                    self.move_cursor_row(-1)?;
                    self.set_cursor_col(new_col)?;
                    self.redraw()?;
                }
            } else {
                self.buffer.delete_char(self.adjusetd_cursor());
                self.move_cursor_col(-1)?;
            }
        }
        self.redraw()
    }

    pub fn write(&mut self) -> Result<String, String> {
        match self.buffer.write() {
            Ok(()) => Ok(format!("\"{}\" written", self.buffer.filename())),
            Err(e) => Err(e),
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn set_height(&mut self, height: usize) {
        self.height = height;
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;
    }

    pub fn loc(&self) -> (usize, usize) {
        self.loc
    }
}
