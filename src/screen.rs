use std::{
    cmp::{max, min},
    io::stdout,
    panic,
};

use crossterm::{
    cursor::{self, SetCursorStyle},
    execute, style,
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
}

impl Screen {
    fn setup() -> Result<()> {
        enable_raw_mode()?;
        panic::set_hook(Box::new(|info| {
            Self::finish().unwrap();
            println!("{info}");
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

        Ok(Self {
            // TODO: centered info screen
            buffer: Buffer::from_string(String::new()),
            offset: 0,
        })
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
        let term_height = Screen::rows() - 1;
        let mut scrolled = false;
        if row < 0 {
            if self.offset > 0 {
                let amt_under = (-row) as usize;
                if self.offset < amt_under {
                    self.offset = 0;
                } else {
                    self.offset -= amt_under;
                }
                scrolled = true;
            }
            row = 0;
        } else if row as usize > term_height {
            let amt_over = row as usize - term_height;
            self.offset += amt_over;
            row -= amt_over as isize;
            scrolled = true;
        }
        self.buffer.set_cursor(row as usize, col);
        self.reprint_cursor()?;
        if scrolled {
            self.print_buffer()?;
        }
        Ok(())
    }

    /// Be absolutely positive this is a valid position!!
    fn set_cursor(&mut self, row: usize, col: usize) -> Result<()> {
        self.buffer.set_cursor(row, col);
        self.reprint_cursor()
    }

    fn print_buffer(&mut self) -> Result<()> {
        execute!(stdout(), cursor::Hide, cursor::MoveTo(0, 0))?;
        for line in self
            .buffer
            .lines()
            .iter()
            .skip(self.offset)
            .take(Screen::rows())
        {
            let padding = " ".repeat(Screen::cols() - line.len());
            execute!(
                stdout(),
                style::Print(format!("{line}{padding}")),
                cursor::MoveToColumn(0),
                cursor::MoveDown(1)
            )?;
        }
        self.reprint_cursor()
    }

    fn reprint_line(&mut self) -> Result<()> {
        let line = self.buffer.nth_line(self.buffer.cursor_row() + self.offset);
        let padding = " ".repeat(Screen::cols() - line.len());
        execute!(
            stdout(),
            cursor::MoveToColumn(0),
            style::Print(format!("{line}{padding}")),
            cursor::MoveToColumn(self.buffer.cursor_col() as u16),
        )
    }

    fn cols() -> usize {
        terminal::size().unwrap().0 as usize
    }

    fn rows() -> usize {
        terminal::size().unwrap().1 as usize
    }

    pub fn load_file(&mut self, filename: String) -> Result<()> {
        self.buffer = Buffer::from_filepath(filename);
        self.print_buffer()?;
        self.buffer.zero_cursor();
        self.offset = 0;
        self.reprint_cursor()
    }

    // TODO: one char at a time is definitely not right
    pub fn type_char(&mut self, c: char) -> Result<()> {
        self.buffer.add_char(c, self.offset);
        self.move_cursor(1, 0)?;
        self.reprint_line()
    }

    pub fn delete_chars(&mut self, n: usize) -> Result<()> {
        for _ in 0..n {
            if self.buffer.cursor_col() == 0 {
                if self.buffer.cursor_row() != 0 {
                    let new_row = self.buffer.cursor_row() - 1;
                    let new_col = self.buffer.nth_line(new_row).len();
                    self.buffer.delete_line_break(self.offset);
                    self.set_cursor(new_row, new_col)?;
                    // TODO: technically only have to reprint all lines below the current one--is
                    // that faster or anything worthwhile?
                    self.print_buffer()?;
                }
            } else {
                self.buffer.delete_char(self.offset);
                self.move_cursor(-1, 0)?;
            }
        }
        self.reprint_line()
    }

    pub fn write(&mut self) {
        // TODO: print errors in status bar
        self.buffer.write().unwrap();
    }
}
