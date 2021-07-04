extern crate termion;

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod cursor_line {
    #[derive(Debug, PartialEq)]
    pub struct CursorLine {
        before: Vec<char>,
        // first item of rest is the cursor position
        rest: Vec<char>,
    }

    impl CursorLine {
        pub fn from_str(str: &str, char_pos: usize) -> Self {
            if str.is_empty() {
                return Self {
                    before: vec![],
                    rest: vec![],
                };
            }

            let mut char_pos = char_pos;
            if char_pos >= str.len() {
                char_pos = str.len() - 1
            }

            Self {
                before: str.chars().take(char_pos).collect(),
                rest: str.chars().skip(char_pos).collect(),
            }
        }

        pub fn line(&self) -> String {
            let mut before: String = String::with_capacity(self.before.len());
            for c in self.before.iter() {
                before.push(*c);
            }

            let mut rest: String = String::with_capacity(self.rest.len());
            for c in self.rest.iter() {
                rest.push(*c);
            }

            format!("{}{}", before, rest)
        }

        pub fn len(&self) -> usize {
            self.before.len() + self.rest.len()
        }

        pub fn set_x(&mut self, x: usize) {
            if x >= self.len() {
                return;
            }

            while self.x() > x {
                self.move_left();
            }

            while self.x() < x {
                self.move_right();
            }
        }

        pub fn x(&self) -> usize {
            self.before.len()
        }

        pub fn move_left(&mut self) -> bool {
            if let Some(cursor_char) = self.before.pop() {
                self.rest.insert(0, cursor_char);
                true
            } else {
                false
            }
        }

        pub fn move_right(&mut self) -> bool {
            if self.rest.len() < 2 {
                false
            } else if let Some(cursor_char) = self.rest.first() {
                self.before.push(*cursor_char);
                self.rest.remove(0);
                true
            } else {
                false
            }
        }

        pub fn delete_char(&mut self) {
            if self.rest.is_empty() {
                return;
            }

            self.rest.remove(0);

            if self.rest.is_empty() {
                if let Some(last) = self.before.pop() {
                    self.rest.push(last)
                }
            }
        }

        pub fn is_at_whitespace(&self) -> bool {
            self.rest
                .first()
                .map(|cursor_char| char::is_whitespace(*cursor_char))
                .unwrap_or(false)
        }

        pub fn is_at_alphanumeric(&self) -> bool {
            self.rest
                .first()
                .map(|cursor_char| char::is_alphanumeric(*cursor_char))
                .unwrap_or(false)
        }

        pub fn is_empty(&self) -> bool {
            self.rest.is_empty()
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::CursorLine;

        #[test]
        fn from_str_empty() {
            let cursor_line = CursorLine::from_str("", 0);

            assert_eq!(cursor_line.before, vec![]);
            assert_eq!(cursor_line.rest, vec![]);
        }

        #[test]
        fn from_str_start() {
            let cursor_line = CursorLine::from_str("012", 0);

            assert_eq!(cursor_line.before, vec![]);
            assert_eq!(cursor_line.rest, vec!['0', '1', '2']);
        }

        #[test]
        fn from_str_middle() {
            let cursor_line = CursorLine::from_str("012", 1);

            assert_eq!(cursor_line.before, vec!['0']);
            assert_eq!(cursor_line.rest, vec!['1', '2']);
        }

        #[test]
        fn from_str_end() {
            let cursor_line = CursorLine::from_str("012", 2);

            assert_eq!(cursor_line.before, vec!['0', '1']);
            assert_eq!(cursor_line.rest, vec!['2']);
        }

        #[test]
        fn from_str_overflowed() {
            let cursor_line = CursorLine::from_str("012", 3);

            assert_eq!(cursor_line.before, vec!['0', '1']);
            assert_eq!(cursor_line.rest, vec!['2']);
        }

        #[test]
        fn is_empty_true() {
            let cursor_line = CursorLine::from_str("", 0);

            let is_empty = cursor_line.is_empty();

            assert_eq!(is_empty, true);
        }

        #[test]
        fn is_empty_false() {
            let mut cursor_line = CursorLine::from_str("01234", 0);

            let is_empty = cursor_line.is_empty();
            assert_eq!(is_empty, false);

            while cursor_line.move_right() {
                let is_empty = cursor_line.is_empty();
                assert_eq!(is_empty, false);
            }
        }

        #[test]
        fn move_right_start() {
            let mut cursor_line = CursorLine::from_str("012", 0);

            let result = cursor_line.move_right();

            assert_eq!(result, true);
            assert_eq!(cursor_line, CursorLine::from_str("012", 1));
        }

        #[test]
        fn move_right_middle() {
            let mut cursor_line = CursorLine::from_str("012", 1);

            let result = cursor_line.move_right();

            assert_eq!(result, true);
            assert_eq!(cursor_line, CursorLine::from_str("012", 2));
        }

        #[test]
        fn move_right_end() {
            let mut cursor_line = CursorLine::from_str("012", 2);

            let result = cursor_line.move_right();

            assert_eq!(result, false);
            assert_eq!(cursor_line, CursorLine::from_str("012", 2));
        }

        #[test]
        fn set_x_to_left() {
            let mut cursor_line = CursorLine::from_str("012", 2);

            cursor_line.set_x(0);

            assert_eq!(cursor_line, CursorLine::from_str("012", 0));
        }

        #[test]
        fn set_x_to_right() {
            let mut cursor_line = CursorLine::from_str("012", 0);

            cursor_line.set_x(2);

            assert_eq!(cursor_line, CursorLine::from_str("012", 2));
        }

        #[test]
        fn set_x_to_invalid() {
            let mut cursor_line = CursorLine::from_str("012", 0);

            cursor_line.set_x(3);

            assert_eq!(cursor_line, CursorLine::from_str("012", 0));
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
struct Vec2 {
    x: usize,
    y: usize,
}

impl Vec2 {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<(u16, u16)> for Vec2 {
    fn from(tuple: (u16, u16)) -> Self {
        Self::new(tuple.0 as usize, tuple.1 as usize)
    }
}

use cursor_line::CursorLine;

#[derive(Debug)]
struct Buffer {
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
    fn from_lines(lines: Vec<String>) -> Buffer {
        if let Some((first, rest)) = lines.split_first() {
            Buffer {
                before_cursor_lines: vec![],
                cursor_line: CursorLine::from_str(first, 0),
                after_cursor_lines: rest.to_vec(),
                size: termion::terminal_size().unwrap().into(),
                offset: 0,
            }
        } else {
            Buffer {
                before_cursor_lines: vec![],
                cursor_line: CursorLine::from_str(&String::new(), 0),
                after_cursor_lines: vec![],
                size: termion::terminal_size().unwrap().into(),
                offset: 0,
            }
        }
    }

    fn cursor(&self) -> Vec2 {
        Vec2::new(self.cursor_line.x(), self.before_cursor_lines.len())
    }

    // TODO: do not use self.lines, we can do much better
    fn render(&self, term: &mut impl Write) {
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

        write!(
            term,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )
        .unwrap();
        for (i, line) in lines.iter().skip(self.offset).take(self.size.y).enumerate() {
            write!(term, "{}", line).unwrap();
            write!(term, "{}", termion::cursor::Goto(1, (i + 2) as u16)).unwrap();
        }

        let cursor = self.cursor();
        write!(
            term,
            "{}",
            termion::cursor::Goto((cursor.x + 1) as u16, (cursor.y - self.offset + 1) as u16)
        )
        .unwrap();

        term.flush().unwrap();
    }

    fn move_forward(&mut self) -> MoveForwardOutcome {
        if self.cursor_line.move_right() {
            MoveForwardOutcome::Char
        } else if !self.after_cursor_lines.is_empty() {
            self.move_cursor_down();
            self.cursor_line.set_x(0);

            MoveForwardOutcome::Line
        } else {
            MoveForwardOutcome::Noop
        }
    }

    fn move_cursor_left(&mut self) {
        self.cursor_line.move_left();
    }

    fn move_cursor_down(&mut self) {
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

    fn move_cursor_up(&mut self) {
        if let Some(previous_line) = self.before_cursor_lines.pop() {
            let current_line = self.cursor_line.line();

            self.after_cursor_lines.insert(0, current_line);
            self.cursor_line = CursorLine::from_str(&previous_line, self.cursor_line.x());
        }
    }

    fn move_cursor_right(&mut self) {
        self.cursor_line.move_right();
    }

    fn delete_char(&mut self) {
        self.cursor_line.delete_char();
    }

    fn move_cursor_first_character(&mut self) {
        self.cursor_line.set_x(0);
    }

    fn word_forward(&mut self) {
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

    fn join_line(&mut self) {
        if let Some(next_line) = self.after_cursor_lines.first() {
            if self.cursor_line.is_empty() {
                self.cursor_line = CursorLine::from_str(&next_line, 0);
            } else {
                self.cursor_line = CursorLine::from_str(
                    &format!("{} {}", self.cursor_line.line(), next_line),
                    self.cursor_line.len(),
                );
            }

            self.after_cursor_lines.remove(0);
        }
    }
}

#[derive(Debug)]
struct State {
    buffer: Buffer,
}

impl State {
    fn render(&self, term: &mut impl Write) {
        self.buffer.render(term);
    }

    fn update(&mut self, evt: Event) {
        match evt {
            Event::Key(Key::Char('h')) => {
                self.buffer.move_cursor_left();
            }
            Event::Key(Key::Char('j')) => {
                self.buffer.move_cursor_down();
            }
            Event::Key(Key::Char('k')) => {
                self.buffer.move_cursor_up();
            }
            Event::Key(Key::Char('l')) => {
                self.buffer.move_cursor_right();
            }
            Event::Key(Key::Char('x')) => {
                self.buffer.delete_char();
            }
            Event::Key(Key::Char('0')) => {
                self.buffer.move_cursor_first_character();
            }
            Event::Key(Key::Char('w')) => {
                self.buffer.word_forward();
            }
            Event::Key(Key::Char('J')) => {
                self.buffer.join_line();
            }
            _ => {}
        }
    }
}

fn write_debug(str: &str) {
    let mut debug_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/tmp/vipi.debug")
        .expect("Failed to open debug file");

    debug_file
        .write_all(&format!("{}\n", str).into_bytes())
        .expect("Failed to write debug file");
}

fn lines_from_file(file_path: &str) -> Vec<String> {
    if let Ok(file) = File::open(file_path) {
        let lines: Result<Vec<_>, _> = BufReader::new(file).lines().collect();
        lines.unwrap_or_else(|_| panic!("Failed to read lines from file: {}", file_path))
    } else {
        File::create(file_path)
            .unwrap_or_else(|_| panic!("Could neither open or create file: {}", file_path));
        vec![]
    }
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let lines = if let Some(file_path) = env::args().nth(1) {
        lines_from_file(&file_path)
    } else {
        vec![]
    };

    let buffer = Buffer::from_lines(lines);

    let mut state = State { buffer };

    write_debug("##########################");
    write_debug(&format!(
        "before_cursor {:?}\n",
        state.buffer.before_cursor_lines.last()
    ));
    write_debug(&format!("cursor {:?}\n", state.buffer.cursor_line));
    write_debug(&format!("cursor_vec {:?}\n", state.buffer.cursor()));
    write_debug(&format!(
        "after_cursor {:?}\n",
        state.buffer.after_cursor_lines.first()
    ));
    write_debug(&format!("size {:?}\n", state.buffer.size));
    write_debug(&format!("offset {:?}\n", state.buffer.offset));
    write_debug("##########################");

    state.render(&mut stdout);
    for c in stdin.events() {
        let evt = c.unwrap();
        write_debug(&format!("{:?}\n", evt));

        if evt == Event::Key(Key::Char('q')) {
            break;
        }

        write_debug("##########################");
        write_debug(&format!(
            "before_cursor {:?}\n",
            state.buffer.before_cursor_lines.last()
        ));
        write_debug(&format!("cursor {:?}\n", state.buffer.cursor_line));
        write_debug(&format!("cursor_vec {:?}\n", state.buffer.cursor()));
        write_debug(&format!(
            "after_cursor {:?}\n",
            state.buffer.after_cursor_lines.first()
        ));
        write_debug(&format!("size {:?}\n", state.buffer.size));
        write_debug(&format!("offset {:?}\n", state.buffer.offset));
        write_debug("##########################");

        state.update(evt);
        state.render(&mut stdout);

        write_debug("##########################");
        write_debug(&format!(
            "before_cursor {:?}\n",
            state.buffer.before_cursor_lines.last()
        ));
        write_debug(&format!("cursor {:?}\n", state.buffer.cursor_line));
        write_debug(&format!("cursor_vec {:?}\n", state.buffer.cursor()));
        write_debug(&format!(
            "after_cursor {:?}\n",
            state.buffer.after_cursor_lines.first()
        ));
        write_debug(&format!("size {:?}\n", state.buffer.size));
        write_debug(&format!("offset {:?}\n", state.buffer.offset));
        write_debug("##########################");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cursor_line::CursorLine;

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
}
