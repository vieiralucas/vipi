use std::fs::File;
use std::io::prelude::*;
use std::io::Write;

use crate::write_debug;
use crate::CursorLine;
use crate::Vec2;

#[derive(Debug)]
pub struct Buffer {
    before_cursor_lines: Vec<String>,
    cursor_line: CursorLine,
    after_cursor_lines: Vec<String>,
    size: Vec2,
    offset: usize,
}

#[derive(PartialEq)]
enum MoveForwardOutcome {
    Char,
    Line,
    Noop,
}

impl Buffer {
    pub fn from_lines(lines: Vec<String>) -> Self {
        let mut size: Vec2 = termion::terminal_size().unwrap().into();
        size.y -= 1;

        if let Some((first, rest)) = lines.split_first() {
            Self {
                before_cursor_lines: vec![],
                cursor_line: CursorLine::from_str(first, 0),
                after_cursor_lines: rest.to_vec(),
                size,
                offset: 0,
            }
        } else {
            Self {
                before_cursor_lines: vec![],
                cursor_line: CursorLine::from_str(&String::new(), 0),
                after_cursor_lines: vec![],
                size,
                offset: 0,
            }
        }
    }

    pub fn from_file_path(file_path: &str) -> Self {
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

        Self::from_lines(lines)
    }

    fn cursor(&self) -> Vec2 {
        Vec2::new(self.cursor_line.x(), self.before_cursor_lines.len())
    }

    pub fn render(&self, term: &mut impl Write) {
        let mut lines =
            Vec::with_capacity(self.before_cursor_lines.len() + 1 + self.after_cursor_lines.len());

        for line in self.before_cursor_lines.iter() {
            lines.push(line);
        }

        let cursor_line_line = self.cursor_line.line();
        lines.push(&cursor_line_line);

        for line in self.after_cursor_lines.iter() {
            lines.push(line);
        }

        let mut row = 0;
        let mut col = 0;
        let mut cursor = Vec2::default();
        for (y, line) in lines.iter().skip(self.offset).take(self.size.y).enumerate() {
            if line.is_empty() {
                write!(
                    term,
                    "{}{}",
                    termion::cursor::Goto((col + 1) as u16, (row + 1) as u16),
                    termion::clear::UntilNewline
                )
                .unwrap();

                if y == self.before_cursor_lines.len() {
                    cursor.x = 0;
                    cursor.y = row;
                }
            }

            for (x, c) in line.chars().enumerate() {
                if col >= self.size.x {
                    row += 1;
                    col = 0;
                }

                if y == self.before_cursor_lines.len() && x == self.cursor_line.x() {
                    cursor.x = col;
                    cursor.y = row;
                }

                write!(
                    term,
                    "{}{}",
                    termion::cursor::Goto((col + 1) as u16, (row + 1) as u16),
                    c
                )
                .unwrap();

                col += 1;
            }

            row += 1;
            col = 0;

            write!(term, "{}", termion::clear::UntilNewline,).unwrap();
        }

        write!(
            term,
            "{}{}",
            termion::clear::AfterCursor,
            termion::cursor::Goto((cursor.x + 1) as u16, (cursor.y + 1) as u16)
        )
        .unwrap();
    }

    fn move_forward(&mut self) -> MoveForwardOutcome {
        if self.cursor_line.move_right(false) {
            MoveForwardOutcome::Char
        } else if !self.after_cursor_lines.is_empty() {
            self.move_cursor_down();
            self.cursor_line.set_x(0);

            MoveForwardOutcome::Line
        } else {
            MoveForwardOutcome::Noop
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_line.move_left();
    }

    pub fn move_cursor_down(&mut self) {
        if let Some(next_line) = self.after_cursor_lines.first() {
            let was_edge = self.cursor().y - self.offset == self.size.y - 1;
            let current_line = self.cursor_line.line();

            self.before_cursor_lines.push(current_line);
            self.cursor_line = CursorLine::from_str(next_line, self.cursor_line.x());
            self.after_cursor_lines.remove(0);

            if was_edge {
                self.offset += 1;
            }
        }
    }

    pub fn move_cursor_up(&mut self) {
        let was_edge = self.cursor().y - self.offset == 0;
        if let Some(previous_line) = self.before_cursor_lines.pop() {
            let current_line = self.cursor_line.line();

            self.after_cursor_lines.insert(0, current_line);
            self.cursor_line = CursorLine::from_str(&previous_line, self.cursor_line.x());

            if was_edge && self.offset > 0 {
                self.offset -= 1;
            }
        }
    }

    pub fn move_cursor_right(&mut self, allow_one_off: bool) {
        self.cursor_line.move_right(allow_one_off);
    }

    pub fn delete_char(&mut self) {
        self.cursor_line.delete_char();
    }

    pub fn move_cursor_first_character(&mut self) {
        self.cursor_line.set_x(0);
    }

    pub fn word_forward(&mut self) {
        let mut moved_from_empty_line = false;
        if self.cursor_line.is_empty() {
            if self.move_forward() == MoveForwardOutcome::Noop {
                return;
            }

            moved_from_empty_line = true;
        }

        if !moved_from_empty_line {
            if self.cursor_line.is_at_whitespace() {
                while self.cursor_line.is_at_whitespace() {
                    if self.move_forward() != MoveForwardOutcome::Char {
                        break;
                    }
                }
            } else if self.cursor_line.is_at_alphanumeric() {
                while self.cursor_line.is_at_alphanumeric() {
                    if self.move_forward() != MoveForwardOutcome::Char {
                        break;
                    }
                }
            } else {
                while !self.cursor_line.is_at_alphanumeric() {
                    if self.move_forward() != MoveForwardOutcome::Char {
                        break;
                    }
                }
            }
        }

        if !self.cursor_line.is_empty() {
            while self.cursor_line.is_at_whitespace() {
                if self.move_forward() == MoveForwardOutcome::Noop {
                    break;
                }
            }
        }
    }

    pub fn join_line(&mut self) {
        if let Some(next_line) = self.after_cursor_lines.first() {
            if self.cursor_line.is_empty() {
                self.cursor_line = CursorLine::from_str(next_line, 0);
            } else {
                self.cursor_line = CursorLine::from_str(
                    &format!("{} {}", self.cursor_line.line(), next_line),
                    self.cursor_line.len(),
                );
            }

            self.after_cursor_lines.remove(0);
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.cursor_line.insert_char(c);
    }

    pub fn clamp_cursor(&mut self) {
        self.cursor_line.clamp();
    }

    pub fn insert_line_after_cursor(&mut self, line: String) {
        self.after_cursor_lines.insert(0, line);
    }

    pub fn write_to_file(&self, file_path: &str) {
        let mut file_contents = String::new();
        for (i, line) in self.before_cursor_lines.iter().enumerate() {
            if i != 0 {
                file_contents.push('\n');
            }
            file_contents.push_str(line);
        }

        let cursor_line_line = self.cursor_line.line();
        file_contents.push('\n');
        file_contents.push_str(&cursor_line_line);

        for line in self.after_cursor_lines.iter() {
            file_contents.push('\n');
            file_contents.push_str(line);
        }

        // TODO: report error to user instead of panic
        std::fs::write(file_path, file_contents).unwrap();
    }

    pub fn insert_new_line(&mut self) {
        let content_before_cursor = self.cursor_line.content_before_cursor();

        self.before_cursor_lines.push(content_before_cursor);
        loop {
            if !self.cursor_line.backspace() {
                break;
            }
        }
    }

    pub fn write_debug(&self) {
        write_debug("##########################");
        write_debug(&format!(
            "before_cursor {:?}\n",
            self.before_cursor_lines.last()
        ));
        write_debug(&format!("cursor {:?}\n", self.cursor_line));
        write_debug(&format!("cursor_vec {:?}\n", self.cursor()));
        write_debug(&format!(
            "after_cursor {:?}\n",
            self.after_cursor_lines.first()
        ));
        write_debug(&format!("size {:?}\n", self.size));
        write_debug(&format!("offset {:?}\n", self.offset));
        write_debug("##########################");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CursorLine;

    #[test]
    fn move_forward() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("abc", 0),
            after_cursor_lines: vec![],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.move_forward();

        assert_eq!(buffer.cursor(), Vec2::new(1, 0));
    }

    #[test]
    fn move_forward_no_op_when_end_of_buffer() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("abc", 2),
            after_cursor_lines: vec![],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.move_forward();

        assert_eq!(buffer.cursor(), Vec2::new(2, 0));
    }

    #[test]
    fn move_forward_wrap_line() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("line1", 4),
            after_cursor_lines: vec!["line2".to_string()],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.move_forward();

        assert_eq!(buffer.cursor(), Vec2::new(0, 1));
    }

    #[test]
    fn move_forward_wrap_line_offset() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("line1", 4),
            after_cursor_lines: vec!["line2".to_string()],
            size: Vec2::new(100, 1),
            offset: 0,
        };

        buffer.move_forward();

        assert_eq!(buffer.cursor(), Vec2::new(0, 1));
        assert_eq!(buffer.offset, 1);
    }

    #[test]
    fn word_forward() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("Word Forward", 0),
            after_cursor_lines: vec![],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();

        assert_eq!(buffer.cursor(), Vec2::new(5, 0));
    }

    #[test]
    fn word_forward_space_character() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str(" Word Forward", 0),
            after_cursor_lines: vec![],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(1, 0));
    }

    #[test]
    fn word_forward_multiple_space_character() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("  Word Forward", 0),
            after_cursor_lines: vec![],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(2, 0));
    }

    #[test]
    fn word_forward_non_alpha_numeric_character() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str(";Word Forward", 0),
            after_cursor_lines: vec![],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(1, 0));
    }

    #[test]
    fn word_forward_multiple_non_alpha_numeric_character() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str(";;Word Forward", 0),
            after_cursor_lines: vec![],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(2, 0));
    }

    #[test]
    fn word_forward_wrap_line() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("word1", 0),
            after_cursor_lines: vec!["word2".to_string()],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(0, 1));

        // next line starts with space
        buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("word1", 0),
            after_cursor_lines: vec![" word2".to_string()],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(1, 1));

        // next line is empty
        buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("word1", 0),
            after_cursor_lines: vec!["".to_string()],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(0, 1));

        // when current line has trailing space
        buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("word1 ", 0),
            after_cursor_lines: vec!["word2".to_string()],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(0, 1));

        // when current line has trailing space and next starts with space
        buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("; ", 0),
            after_cursor_lines: vec![" word2".to_string()],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(1, 1));

        // when current line has trailing space and next starts with space
        buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("weird ", 0),
            after_cursor_lines: vec![" ".to_string(), " scenario".to_string()],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(1, 2));
    }

    #[test]
    fn word_forward_wrap_line_current_line_is_empty() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("", 0),
            after_cursor_lines: vec!["word".to_string()],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(0, 1));
    }

    #[test]
    fn word_forward_starts_white_space() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("    }", 0),
            after_cursor_lines: vec!["}".to_string()],
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(4, 0));
    }

    #[test]
    fn word_forward_wrap_line_offset() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("word1", 0),
            after_cursor_lines: vec!["word2".to_string(), "word3".to_string()],
            size: Vec2::new(100, 1),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor(), Vec2::new(0, 1));
        assert_eq!(buffer.offset, 1);
    }

    #[test]
    fn clamp_cursor() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("", 0),
            after_cursor_lines: vec![],
            size: Vec2::new(100, 1),
            offset: 0,
        };

        buffer.insert_char('a');
        buffer.clamp_cursor();

        assert_eq!(buffer.cursor(), Vec2::new(0, 0));
    }

    #[test]
    fn insert_char() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("", 0),
            after_cursor_lines: vec![],
            size: Vec2::new(100, 1),
            offset: 0,
        };

        buffer.insert_char('a');

        assert_eq!(buffer.cursor_line.line(), "a");
        assert_eq!(buffer.cursor(), Vec2::new(1, 0));
    }

    #[test]
    fn insert_line_after_cursor() {
        let mut buffer = Buffer {
            before_cursor_lines: vec![],
            cursor_line: CursorLine::from_str("", 0),
            after_cursor_lines: vec![],
            size: Vec2::new(100, 1),
            offset: 0,
        };

        buffer.insert_line_after_cursor("hello".to_string());

        assert_eq!(buffer.after_cursor_lines, vec!["hello"]);
    }

    #[test]
    fn insert_new_line_start_of_current_line() {
        let mut buffer = Buffer {
            before_cursor_lines: vec!["before".to_string()],
            cursor_line: CursorLine::from_str("cursor_line", 0),
            after_cursor_lines: vec!["after".to_string()],
            size: Vec2::new(100, 1),
            offset: 0,
        };

        buffer.insert_new_line();

        assert_eq!(buffer.before_cursor_lines, vec!["before", ""]);
    }

    #[test]
    fn insert_new_line_end_of_current_line() {
        let mut cursor_line = CursorLine::from_str("1", 0);
        cursor_line.move_right(true);

        let mut buffer = Buffer {
            before_cursor_lines: vec!["before".to_string()],
            cursor_line,
            after_cursor_lines: vec!["after".to_string()],
            size: Vec2::new(100, 1),
            offset: 0,
        };

        buffer.insert_new_line();

        assert_eq!(buffer.before_cursor_lines, vec!["before", "1"]);
        assert_eq!(buffer.cursor_line.line(), "");
    }

    #[test]
    fn insert_new_line_middle_of_current_line() {
        let mut buffer = Buffer {
            before_cursor_lines: vec!["before".to_string()],
            cursor_line: CursorLine::from_str("cursor_line", 6),
            after_cursor_lines: vec!["after".to_string()],
            size: Vec2::new(100, 1),
            offset: 0,
        };

        buffer.insert_new_line();

        assert_eq!(buffer.before_cursor_lines, vec!["before", "cursor"]);
        assert_eq!(buffer.cursor_line.line(), "_line");
    }
}
