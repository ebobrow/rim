use std::{
    fs::File,
    io::{self, Read, Seek, Write},
    path::Path,
};

pub struct Buffer {
    lines: Vec<String>,

    /// (row, col)
    cursor: (usize, usize),

    handle: Option<File>,
}

impl Buffer {
    pub fn from_filepath<P: AsRef<Path>>(path: P) -> Self {
        // TODO: handle the implicit blank line at the end. which is to say, don't print it, print
        // something if it doesn't exist, retain it on save.
        let mut file = File::options().write(true).read(true).open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        Self {
            lines: contents.split('\n').map(String::from).collect(),
            cursor: (0, 0),
            handle: Some(file),
        }
    }

    pub fn from_string(s: String) -> Self {
        Self {
            lines: s.split('\n').map(String::from).collect(),
            cursor: (0, 0),
            handle: None,
        }
    }

    pub fn add_char(&mut self, c: char, offset: usize) {
        self.lines[self.cursor.0 + offset].insert(self.cursor.1, c);
    }

    pub fn delete_char(&mut self, offset: usize) {
        self.lines[self.cursor.0 + offset].remove(self.cursor.1 - 1);
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

        Ok(())
    }
}
