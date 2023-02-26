use std::{
    cmp::{max, min},
    fs::File,
    io::Read,
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
        let mut file = File::open(path).unwrap();
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
}
