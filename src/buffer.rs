use std::{
    fs::File,
    io::{self, Read, Seek, Write},
};

pub struct Buffer {
    lines: Vec<String>,

    /// (row, col)
    cursor: (usize, usize),

    handle: Option<File>,
    filename: String,
    unsaved_changes: bool,
}

impl Buffer {
    pub fn from_filepath(path: impl ToString) -> Self {
        // TODO: handle the implicit blank line at the end. which is to say, don't print it, print
        // something if it doesn't exist, retain it on save.
        let mut file = File::options()
            .write(true)
            .read(true)
            .open(path.to_string())
            .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        Self {
            lines: contents.split('\n').map(String::from).collect(),
            cursor: (0, 0),
            handle: Some(file),
            filename: path.to_string(),
            unsaved_changes: false,
        }
    }

    pub fn from_string(s: String) -> Self {
        Self {
            lines: s.split('\n').map(String::from).collect(),
            cursor: (0, 0),
            handle: None,
            filename: String::from("[scratch]"),
            unsaved_changes: false,
        }
    }

    pub fn add_char(&mut self, c: char, offset: usize) {
        self.lines[self.cursor.0 + offset].insert(self.cursor.1, c);
        self.unsaved_changes = true;
    }

    pub fn add_line_break(&mut self, offset: usize) {
        let line = &mut self.lines[self.cursor.0 + offset];
        let new_line = if self.cursor.1 == line.len() {
            String::new()
        } else {
            line.split_off(self.cursor.1)
        };
        self.lines.insert(self.cursor.0 + 1 + offset, new_line);
        self.unsaved_changes = true;
    }

    pub fn delete_char(&mut self, offset: usize) {
        self.lines[self.cursor.0 + offset].remove(self.cursor.1 - 1);
        self.unsaved_changes = true;
    }

    pub fn delete_line_break(&mut self, offset: usize) {
        let old_row = self.lines.remove(self.cursor.0 + offset);
        self.lines[self.cursor.0 + offset - 1].push_str(&old_row);
        self.unsaved_changes = true;
    }

    pub fn set_cursor(&mut self, r: usize, c: usize) {
        self.cursor = (r, c);
    }

    pub fn zero_cursor(&mut self) {
        self.cursor = (0, 0);
    }

    pub fn cursor_row(&self) -> usize {
        self.cursor.0
    }

    pub fn cursor_col(&self) -> usize {
        self.cursor.1
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn nth_line(&self, n: usize) -> &str {
        &self.lines[n]
    }

    pub fn write(&mut self) -> io::Result<()> {
        if let Some(mut handle) = self.handle.as_ref() {
            handle.rewind()?;
            handle.write_all(self.lines.join("\n").as_bytes())?;
        } else {
            todo!("error: no filename")
        }
        self.unsaved_changes = false;

        Ok(())
    }

    pub fn filename(&self) -> &str {
        self.filename.as_ref()
    }

    pub fn unsaved_changes(&self) -> bool {
        self.unsaved_changes
    }
}
