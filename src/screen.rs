use std::{
    cmp::{max, min},
    io::stdout,
};

use crossterm::{
    cursor, execute, style,
    terminal::{self, enable_raw_mode},
    Result,
};

use crate::buffer::{self, Buffer};

pub struct Screen {
    // TODO: this isn't really true right b/c of splits? implement `active_buf_idx` and `buffers`
    // as Vec? Or repurpose this as window and move screen logic to new class that only does the
    // setup stuff
    buffer: Buffer,

    /// (row, col)
    cursor: (usize, usize),
}

impl Screen {
    fn setup() -> Result<()> {
        enable_raw_mode()?;

        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            cursor::MoveTo(0, 0)
        )
    }

    pub fn finish(&self) -> Result<()> {
        execute!(stdout(), terminal::LeaveAlternateScreen)
    }

    pub fn new() -> Result<Self> {
        Self::setup()?;

        Ok(Self {
            // TODO: centered info screen
            buffer: buffer::parse_text(""),
            cursor: (0, 0),
        })
    }

    /// Does not check bounds; use `move_cursor` for user input
    fn set_cursor(&mut self, r: usize, c: usize) -> Result<()> {
        self.cursor = (r, c);
        execute!(stdout(), cursor::MoveTo(c as u16, r as u16))
    }

    /// Moves cursor `rl` to the right (negative goes left) and `du` down if allowed
    pub fn move_cursor(&mut self, rl: isize, du: isize) -> Result<()> {
        let (row, col) = if self.buffer.is_empty() {
            (0, 0)
        } else {
            let normalize = |n| if n == 0 { 0 } else { n - 1 };

            let row = min(
                self.buffer.len() - 1,
                max(0, self.cursor.0 as isize + du) as usize,
            );
            (
                row,
                min(
                    normalize(self.buffer[row].len()),
                    max(0, self.cursor.1 as isize + rl) as usize,
                ),
            )
        };
        self.set_cursor(row, col)
    }

    fn write_buffer(&mut self) -> Result<()> {
        for line in &self.buffer {
            execute!(
                stdout(),
                style::Print(line),
                cursor::MoveToColumn(0),
                cursor::MoveDown(1)
            )?;
        }
        Ok(())
    }

    pub fn load_file(&mut self, filename: String) -> Result<()> {
        self.buffer = buffer::parse_file(filename);
        self.write_buffer()?;
        self.set_cursor(0, 0)
    }
}
