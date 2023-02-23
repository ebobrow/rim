pub type Buffer<'a> = Vec<&'a str>;

// struct Buffer<'a> {
//     lines: Vec<Line<'a>>,
//     // len: usize,
// }

// struct Line<'a> {
//     raw: &'a str,
//     num: usize,
//     // len: usize,
// }

pub fn parse_text(text: &str) -> Buffer<'_> {
    text.lines().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text() {
        assert_eq!(parse_text("1\n2\n3"), vec!["1", "2", "3"]);
    }
}
