use std::{
    fs::File,
    io::{Read, Seek, Write},
};

pub struct Buffer {
    lines: Vec<String>,
    handle: Option<File>,
    filename: String,

    // TODO: don't trigger if you typed j and hten triggered jk macro
    unsaved_changes: bool,
    terminal_newline: bool,
}

impl Buffer {
    pub fn from_filepath(path: impl ToString) -> Self {
        let mut file = File::options()
            .write(true)
            .read(true)
            .open(path.to_string())
            .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let mut lines: Vec<String> = contents.split('\n').map(String::from).collect();
        let mut terminal_newline = false;
        if let Some(end) = lines.last() {
            if end.is_empty() {
                lines = lines[..lines.len() - 1].to_vec();
                terminal_newline = true;
            }
        }
        Self {
            lines,
            handle: Some(file),
            filename: path.to_string(),
            unsaved_changes: false,
            terminal_newline,
        }
    }

    pub fn from_string(s: String) -> Self {
        Self {
            lines: s.split('\n').map(String::from).collect(),
            handle: None,
            filename: String::from("[No Name]"),
            unsaved_changes: false,
            terminal_newline: false,
        }
    }

    pub fn add_char(&mut self, c: char, cursor: (usize, usize)) {
        self.lines[cursor.0].insert(cursor.1, c);
        self.unsaved_changes = true;
    }

    pub fn add_line_break(&mut self, cursor: (usize, usize)) {
        let line = &mut self.lines[cursor.0];
        let new_line = if cursor.1 == line.len() {
            String::new()
        } else {
            line.split_off(cursor.1)
        };
        self.lines.insert(cursor.0 + 1, new_line);
        self.unsaved_changes = true;
    }

    pub fn new_line_below(&mut self, cursor: (usize, usize)) {
        self.lines.insert(cursor.0 + 1, String::new());
    }

    pub fn new_line_above(&mut self, cursor: (usize, usize)) {
        self.lines.insert(cursor.0, String::new());
    }

    pub fn delete_char(&mut self, cursor: (usize, usize)) {
        self.lines[cursor.0].remove(cursor.1 - 1);
        self.unsaved_changes = true;
    }

    pub fn delete_line(&mut self, cursor: (usize, usize)) {
        self.lines.remove(cursor.0);
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
    }

    pub fn change_line(&mut self, cursor: (usize, usize)) {
        self.lines[cursor.0].clear();
    }

    pub fn delete_line_break(&mut self, cursor: (usize, usize)) {
        let old_row = self.lines.remove(cursor.0);
        self.lines[cursor.0 - 1].push_str(&old_row);
        self.unsaved_changes = true;
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn nth_line(&self, n: usize) -> &str {
        &self.lines[n]
    }

    pub fn write(&mut self) -> Result<(), String> {
        if let Some(mut handle) = self.handle.as_ref() {
            handle.rewind().map_err(|_| "Internal error")?;
            handle
                .write_all(self.lines.join("\n").as_bytes())
                .map_err(|_| "Internal error")?;
            if self.terminal_newline {
                handle.write(b"\n").map_err(|_| "Internal error")?;
            }
        } else {
            return Err("No filename".to_string());
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
