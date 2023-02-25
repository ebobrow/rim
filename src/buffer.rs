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

    pub fn add_char(&mut self, c: char) {
        self.lines[self.cursor.0].insert(self.cursor.1, c);
    }

    pub fn delete_char(&mut self) {
        self.lines[self.cursor.0].remove(self.cursor.1 - 1);
    }

    /// Moves cursor `rl` to the right (negative goes left) and `du` down if allowed
    pub fn move_cursor(&mut self, rl: isize, du: isize) {
        self.cursor = if self.lines.is_empty() {
            (0, 0)
        } else {
            let normalize = |n| if n == 0 { 0 } else { n - 1 };

            let row = min(
                self.lines.len() - 1,
                max(0, self.cursor.0 as isize + du) as usize,
            );
            (
                row,
                min(
                    normalize(self.lines[row].len()),
                    max(0, self.cursor.1 as isize + rl) as usize,
                ),
            )
        };
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
