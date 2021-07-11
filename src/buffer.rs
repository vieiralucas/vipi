use std::cmp::Ordering;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;

use crate::write_debug;
use crate::Vec2;

#[derive(Debug)]
pub struct Buffer {
    lines: Vec<String>,
    cursor: Vec2,
    offset: usize,
    pos: Vec2,
    size: Vec2,
    line_num: bool,
}

#[derive(PartialEq)]
enum MoveForwardOutcome {
    Char,
    Line,
    Noop,
}

impl Buffer {
    pub fn from_lines(lines: Vec<String>, pos: Vec2, size: Vec2, line_num: bool) -> Self {
        Self {
            lines,
            cursor: Vec2::default(),
            offset: 0,
            size,
            pos,
            line_num,
        }
    }

    pub fn from_file_path(file_path: &str, pos: Vec2, size: Vec2) -> Self {
        let lines = if let Ok(mut file) = File::open(file_path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .unwrap_or_else(|_| panic!("Failed to read lines from file: {}", file_path));
            contents.split('\n').map(|s| s.to_string()).collect()
        } else {
            File::create(file_path)
                .unwrap_or_else(|_| panic!("Could neither open or create file: {}", file_path));
            vec![]
        };

        Self::from_lines(lines, pos, size, true)
    }

    pub fn render(&self, term: &mut impl Write) {
        let line_num_size = self.lines.len().to_string().len();

        let mut row = 0;
        let mut col = 0;
        let mut cursor: Option<Vec2> = None;

        for (y, line) in self
            .lines
            .iter()
            .skip(self.offset)
            .take(self.size.y)
            .enumerate()
        {
            let line_num = y + self.offset + 1;

            let relative_line_num = match self.cursor.y.cmp(&(y + self.offset)) {
                Ordering::Greater => self.cursor.y - (y + self.offset),
                Ordering::Less => (y + self.offset) - self.cursor.y,
                Ordering::Equal => line_num,
            };

            let line_num_str = format!(
                "{:>width$}",
                relative_line_num.to_string(),
                width = line_num_size
            );

            if line.is_empty() {
                write!(
                    term,
                    "{}{}{} ",
                    termion::cursor::Goto(
                        (self.pos.x + col + 1) as u16,
                        (self.pos.y + row + 1) as u16
                    ),
                    line_num_str,
                    termion::clear::UntilNewline
                )
                .unwrap();

                if y + self.offset == self.cursor.y {
                    cursor = Some(Vec2::new(0, row));
                }
            }

            for (x, c) in line.chars().enumerate() {
                if col >= self.size.x {
                    row += 1;
                    col = 0;
                }

                if y + self.offset == self.cursor.y && x == self.cursor.x {
                    cursor = Some(Vec2::new(col, row));
                }

                if x == 0 && self.line_num {
                    write!(
                        term,
                        "{}{} ",
                        termion::cursor::Goto(
                            (self.pos.x + col + 1) as u16,
                            (self.pos.y + row + 1) as u16
                        ),
                        line_num_str
                    )
                    .unwrap();
                }

                write!(
                    term,
                    "{}{}",
                    termion::cursor::Goto(
                        ((if self.line_num { line_num_size + 1 } else { 0 }) + self.pos.x + col + 1)
                            as u16,
                        (self.pos.y + row + 1) as u16
                    ),
                    c
                )
                .unwrap();

                col += 1;
            }

            if y + self.offset == self.cursor.y && cursor.is_none() {
                cursor = Some(Vec2::new(col, row));
            }

            row += 1;
            col = 0;

            write!(term, "{}", termion::clear::UntilNewline,).unwrap();
        }

        write!(term, "{}", termion::clear::AfterCursor,).unwrap();

        if let Some(cursor) = cursor {
            write!(
                term,
                "{}",
                termion::cursor::Goto(
                    ((if self.line_num { line_num_size + 1 } else { 0 })
                        + self.pos.x
                        + cursor.x
                        + 1) as u16,
                    (self.pos.y + cursor.y + 1) as u16
                )
            )
            .unwrap();
        }
    }

    pub fn current_line(&self) -> &String {
        &self.lines[self.cursor.y]
    }

    fn move_forward(&mut self) -> MoveForwardOutcome {
        let line = self.current_line();
        if self.cursor.x + 1 < line.len() {
            self.cursor.x += 1;
            MoveForwardOutcome::Char
        } else if self.cursor.y + 1 < self.lines.len() {
            self.move_cursor_down();
            self.cursor.x = 0;
            MoveForwardOutcome::Line
        } else {
            MoveForwardOutcome::Noop
        }
    }

    pub fn clamp_cursor(&mut self, allow_one_off: bool) {
        let line = self.current_line();

        let mut max = line.len();
        if !allow_one_off && max > 0 {
            max -= 1;
        }

        self.cursor.x = self.cursor.x.clamp(0, max);
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        let is_edge = self.cursor.y - self.offset == self.size.y - 1;
        if self.cursor.y + 1 < self.lines.len() {
            self.cursor.y += 1;

            if is_edge {
                self.offset += 1;
            }
        }

        self.clamp_cursor(false);
    }

    pub fn move_cursor_up(&mut self) {
        let is_edge = self.cursor.y - self.offset == 0;
        if self.cursor.y > 0 {
            self.cursor.y -= 1;

            if is_edge {
                self.offset -= 1;
            }
        }

        self.clamp_cursor(false);
    }

    pub fn move_cursor_right(&mut self, allow_one_off: bool) {
        self.cursor.x += 1;
        self.clamp_cursor(allow_one_off);
    }

    pub fn delete_char(&mut self) {
        let line = &mut self.lines[self.cursor.y];
        if line.len() > self.cursor.x {
            line.remove(self.cursor.x);
            self.clamp_cursor(false);
        }
    }

    pub fn move_cursor_first_character(&mut self) {
        self.cursor.x = 0;
    }

    fn is_at_whitespace(&self) -> bool {
        self.current_line()
            .chars()
            .nth(self.cursor.x)
            .map(|c| c.is_whitespace())
            .unwrap_or(false)
    }

    fn is_at_alphanumeric(&self) -> bool {
        self.current_line()
            .chars()
            .nth(self.cursor.x)
            .map(|c| c.is_alphanumeric())
            .unwrap_or(false)
    }

    pub fn word_forward(&mut self) {
        let mut moved_from_empty_line = false;
        if self.current_line().is_empty() {
            if self.move_forward() == MoveForwardOutcome::Noop {
                return;
            }

            moved_from_empty_line = true;
        }

        if !moved_from_empty_line {
            if self.is_at_whitespace() {
                while self.is_at_whitespace() {
                    if self.move_forward() != MoveForwardOutcome::Char {
                        break;
                    }
                }
            } else if self.is_at_alphanumeric() {
                while self.is_at_alphanumeric() {
                    if self.move_forward() != MoveForwardOutcome::Char {
                        break;
                    }
                }
            } else {
                while !self.is_at_alphanumeric() {
                    if self.move_forward() != MoveForwardOutcome::Char {
                        break;
                    }
                }
            }
        }

        if !self.current_line().is_empty() {
            while self.is_at_whitespace() {
                if self.move_forward() == MoveForwardOutcome::Noop {
                    break;
                }
            }
        }
    }

    pub fn join_line(&mut self) {
        if let Some(next_line) = self.lines.get(self.cursor.y + 1) {
            if self.current_line().is_empty() {
                self.lines.remove(self.cursor.y);
                self.cursor.x = self.current_line().len() - 1;
            } else {
                let current_line = self.current_line().clone();
                self.lines[self.cursor.y] = format!("{} {}", current_line, next_line);
                self.lines.remove(self.cursor.y + 1);
                self.cursor.x = current_line.len();
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor.y];
        if self.cursor.x < line.len() {
            line.insert(self.cursor.x, c);
        } else {
            line.push(c);
        }

        self.cursor.x += 1;
    }

    pub fn insert_line_after_cursor(&mut self, line: String) {
        if self.cursor.y + 1 < self.lines.len() {
            self.lines.insert(self.cursor.y + 1, line);
        } else {
            self.lines.push(line);
        }
    }

    // TODO: improve performance
    pub fn write_to_file(&self, file_path: &str) {
        let mut file_contents = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i != 0 {
                file_contents.push('\n');
            }
            file_contents.push_str(line);
        }

        // TODO: report error to user instead of panic
        std::fs::write(file_path, file_contents).unwrap();
    }

    pub fn backspace(&mut self) {
        let line = self.current_line().clone();
        let x = self.cursor.x;
        let y = self.cursor.y;

        if x == 0 && y > 0 {
            self.move_cursor_up();
            self.join_line();
        } else {
            self.move_cursor_left();
        }

        self.delete_char();

        if x >= line.len() {
            self.move_cursor_right(true);
        }
    }

    pub fn insert_new_line(&mut self) {
        let line = self.current_line().clone();
        let (before_cursor, from_cursor) = line.split_at(self.cursor.x);

        self.lines[self.cursor.y] = before_cursor.to_string();
        self.insert_line_after_cursor(from_cursor.to_string());
        self.cursor.y += 1;
        self.cursor.x = 0;
    }

    pub fn write_debug(&self) {
        write_debug("##########################");
        write_debug(&format!("offset {:?}\n", self.offset));
        write_debug(&format!("cursor {:?}\n", self.cursor));
        write_debug(&format!("current_line {:?}\n", self.current_line()));
        write_debug("##########################");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_cursor_down() {
        let mut buffer = Buffer {
            lines: vec!["line1".to_string(), "line2".to_string()],
            cursor: Vec2::default(),
            offset: 0,
            pos: Vec2::default(),
            size: Vec2::new(100, 100),
        };

        buffer.move_cursor_down();

        assert_eq!(buffer.cursor, Vec2::new(0, 1));
    }

    #[test]
    fn move_cursor_down_offset() {
        let mut buffer = Buffer {
            lines: vec!["line1".to_string(), "line2".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 1),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.move_cursor_down();

        assert_eq!(buffer.cursor, Vec2::new(0, 1));
        assert_eq!(buffer.offset, 1);
    }

    #[test]
    fn move_cursor_down_clamp_x() {
        let mut buffer = Buffer {
            lines: vec!["big line".to_string(), "small".to_string()],
            cursor: Vec2::new(7, 0),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.move_cursor_down();

        assert_eq!(buffer.cursor, Vec2::new(4, 1));
    }

    #[test]
    fn move_cursor_up_clamp_x() {
        let mut buffer = Buffer {
            lines: vec!["small".to_string(), "big line".to_string()],
            cursor: Vec2::new(7, 1),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.move_cursor_up();

        assert_eq!(buffer.cursor, Vec2::new(4, 0));
    }

    #[test]
    fn move_forward() {
        let mut buffer = Buffer {
            lines: vec!["abc".to_string()],
            cursor: Vec2::default(),
            offset: 0,
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
        };

        buffer.move_forward();

        assert_eq!(buffer.cursor, Vec2::new(1, 0));
    }

    #[test]
    fn move_forward_no_op_when_end_of_buffer() {
        let mut buffer = Buffer {
            lines: vec!["abc".to_string()],
            cursor: Vec2::new(2, 0),
            offset: 0,
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
        };

        buffer.move_forward();

        assert_eq!(buffer.cursor, Vec2::new(2, 0));
    }

    #[test]
    fn move_forward_wrap_line() {
        let mut buffer = Buffer {
            lines: vec!["line!".to_string(), "line2".to_string()],
            cursor: Vec2::new(4, 0),
            offset: 0,
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
        };

        buffer.move_forward();

        assert_eq!(buffer.cursor, Vec2::new(0, 1));
    }

    #[test]
    fn move_forward_wrap_line_offset() {
        let mut buffer = Buffer {
            lines: vec!["line1".to_string(), "line2".to_string()],
            cursor: Vec2::new(4, 0),
            offset: 0,
            size: Vec2::new(100, 1),
            pos: Vec2::default(),
        };

        buffer.move_forward();

        assert_eq!(buffer.cursor, Vec2::new(0, 1));
        assert_eq!(buffer.offset, 1);
    }

    #[test]
    fn word_forward() {
        let mut buffer = Buffer {
            lines: vec!["Word Forward".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();

        assert_eq!(buffer.cursor, Vec2::new(5, 0));
    }

    #[test]
    fn word_forward_space_character() {
        let mut buffer = Buffer {
            lines: vec![" Word Forward".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(1, 0));
    }

    #[test]
    fn word_forward_multiple_space_character() {
        let mut buffer = Buffer {
            lines: vec!["  Word Forward".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(2, 0));
    }

    #[test]
    fn word_forward_non_alpha_numeric_character() {
        let mut buffer = Buffer {
            lines: vec![";Word Forward".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(1, 0));
    }

    #[test]
    fn word_forward_multiple_non_alpha_numeric_character() {
        let mut buffer = Buffer {
            lines: vec![";;Word Forward".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(2, 0));
    }

    #[test]
    fn word_forward_wrap_line() {
        let mut buffer = Buffer {
            lines: vec!["word1".to_string(), "word2".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(0, 1));

        // next line starts with space
        buffer = Buffer {
            lines: vec!["word1".to_string(), " word2".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(1, 1));

        // next line is empty
        buffer = Buffer {
            lines: vec!["word1".to_string(), "".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(0, 1));

        // when current line has trailing space
        buffer = Buffer {
            lines: vec!["word1 ".to_string(), "word2".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(0, 1));

        // when current line has trailing space and next starts with space
        buffer = Buffer {
            lines: vec!["; ".to_string(), " word2".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(1, 1));

        buffer = Buffer {
            lines: vec![
                "weird ".to_string(),
                " ".to_string(),
                " scenario".to_string(),
            ],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(1, 2));
    }

    #[test]
    fn word_forward_wrap_line_current_line_is_empty() {
        let mut buffer = Buffer {
            lines: vec!["".to_string(), "word".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(0, 1));
    }

    #[test]
    fn word_forward_starts_white_space() {
        let mut buffer = Buffer {
            lines: vec!["    }".to_string(), "}".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(4, 0));
    }

    #[test]
    fn word_forward_wrap_line_offset() {
        let mut buffer = Buffer {
            lines: vec![
                "word1".to_string(),
                "word2".to_string(),
                "word3".to_string(),
            ],
            cursor: Vec2::default(),
            size: Vec2::new(100, 1),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(0, 1));
        assert_eq!(buffer.offset, 1);
    }

    #[test]
    fn clamp_cursor() {
        let mut buffer = Buffer {
            lines: vec!["".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 1),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.insert_char('a');
        buffer.clamp_cursor(false);

        assert_eq!(buffer.cursor, Vec2::new(0, 0));
    }

    #[test]
    fn insert_char() {
        let mut buffer = Buffer {
            lines: vec!["".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 1),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.insert_char('a');

        assert_eq!(buffer.current_line(), "a");
        assert_eq!(buffer.cursor, Vec2::new(1, 0));
    }

    #[test]
    fn insert_line_after_cursor_last_line() {
        let mut buffer = Buffer {
            lines: vec!["".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 1),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.insert_line_after_cursor("hello".to_string());

        assert_eq!(buffer.lines, vec!["", "hello"]);
        assert_eq!(buffer.cursor, Vec2::default());
    }

    #[test]
    fn insert_line_after_cursor_middle_line() {
        let mut buffer = Buffer {
            lines: vec![
                "line1".to_string(),
                "line2".to_string(),
                "line3".to_string(),
            ],
            cursor: Vec2::new(2, 1),
            size: Vec2::new(100, 1),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.insert_line_after_cursor("inserted".to_string());

        assert_eq!(buffer.lines, vec!["line1", "line2", "inserted", "line3"]);
        assert_eq!(buffer.cursor, Vec2::new(2, 1));
    }

    #[test]
    fn insert_new_line_start_of_current_line() {
        let mut buffer = Buffer {
            lines: vec![
                "before".to_string(),
                "cursor_line".to_string(),
                "after".to_string(),
            ],
            cursor: Vec2::new(0, 1),
            size: Vec2::new(100, 1),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.insert_new_line();

        assert_eq!(buffer.lines, vec!["before", "", "cursor_line", "after"]);
        assert_eq!(buffer.cursor, Vec2::new(0, 2));
    }

    #[test]
    fn insert_new_line_end_of_current_line() {
        let mut buffer = Buffer {
            lines: vec!["before".to_string(), "1".to_string(), "after".to_string()],
            cursor: Vec2::new(1, 1),
            size: Vec2::new(100, 1),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.insert_new_line();

        assert_eq!(buffer.lines, vec!["before", "1", "", "after"]);
        assert_eq!(buffer.cursor, Vec2::new(0, 2));
    }

    #[test]
    fn insert_new_line_middle_of_current_line() {
        let mut buffer = Buffer {
            lines: vec![
                "before".to_string(),
                "cursor_line".to_string(),
                "after".to_string(),
            ],
            cursor: Vec2::new(6, 1),
            size: Vec2::new(100, 1),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.insert_new_line();

        assert_eq!(buffer.lines, vec!["before", "cursor", "_line", "after"]);
        assert_eq!(buffer.cursor, Vec2::new(0, 2));
    }

    #[test]
    fn join_line() {
        let mut buffer = Buffer {
            lines: vec!["line1".to_string(), "line2".to_string()],
            cursor: Vec2::new(0, 0),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.join_line();

        assert_eq!(buffer.lines, vec!["line1 line2"]);
        assert_eq!(buffer.cursor, Vec2::new(5, 0));
    }

    #[test]
    fn backspace_cursor_one_off() {
        let mut buffer = Buffer {
            lines: vec!["0123456".to_string()],
            cursor: Vec2::new(7, 0),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.backspace();

        assert_eq!(buffer.lines, vec!["012345"]);
        assert_eq!(buffer.cursor, Vec2::new(6, 0));
    }

    #[test]
    fn backspace_cursor_middle() {
        let mut buffer = Buffer {
            lines: vec!["0123456".to_string()],
            cursor: Vec2::new(5, 0),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.backspace();

        assert_eq!(buffer.lines, vec!["012356"]);
        assert_eq!(buffer.cursor, Vec2::new(4, 0));
    }

    #[test]
    fn backspace_cursor_start() {
        let mut buffer = Buffer {
            lines: vec!["0123".to_string(), "4567".to_string()],
            cursor: Vec2::new(0, 1),
            size: Vec2::new(100, 100),
            pos: Vec2::default(),
            offset: 0,
        };

        buffer.backspace();

        assert_eq!(buffer.lines, vec!["01234567"]);
        assert_eq!(buffer.cursor, Vec2::new(4, 0));
    }
}
