use std::{io::stdout, panic};

use crossterm::{
    cursor::{self, SetCursorStyle},
    execute, style,
    terminal::{self, disable_raw_mode, enable_raw_mode},
    Result,
};

use crate::buffer::{self, Buffer};

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
                (self.buffer.cursor_row() + self.offset) as u16
            )
        )
    }

    /// Moves cursor `rl` to the right (negative goes left) and `du` down if allowed
    pub fn move_cursor(&mut self, rl: isize, du: isize) -> Result<()> {
        self.buffer.move_cursor(rl, du);
        self.reprint_cursor()
    }

    fn write_buffer(&mut self) -> Result<()> {
        for line in self.buffer.lines() {
            execute!(
                stdout(),
                style::Print(line),
                cursor::MoveToColumn(0),
                cursor::MoveDown(1)
            )?;
        }
        Ok(())
    }

    fn reprint_line(&mut self) -> Result<()> {
        execute!(
            stdout(),
            cursor::MoveToColumn(0),
            style::Print(&self.buffer.nth_line(self.buffer.cursor_row())),
            cursor::MoveToColumn(self.buffer.cursor_col() as u16),
        )
    }

    pub fn load_file(&mut self, filename: String) -> Result<()> {
        self.buffer = Buffer::from_filepath(filename);
        self.write_buffer()?;
        self.buffer.zero_cursor();
        self.offset = 0;
        self.reprint_cursor()
    }

    // TODO: one char at a time is definitely not right
    pub fn type_char(&mut self, c: char) -> Result<()> {
        // buffer::add_char(&mut self.buffer, c, self.cursor.0, self.cursor.1);
        self.buffer.add_char(c);
        self.move_cursor(1, 0)?;
        self.reprint_line()
    }
}
