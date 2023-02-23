use std::{fs::File, io::Read};

// also store file handle
pub type Buffer = Vec<String>;

// struct Buffer<'a> {
//     lines: Vec<Line<'a>>,
//     // len: usize,
// }

// struct Line<'a> {
//     raw: &'a str,
//     num: usize,
//     // len: usize,
// }

pub fn parse_text(text: &str) -> Buffer {
    text.to_owned().split('\n').map(String::from).collect()
}

pub fn parse_file(filename: String) -> Buffer {
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents.split('\n').map(String::from).collect()
}
