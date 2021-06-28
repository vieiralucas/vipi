extern crate termion;

use std::env;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

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

#[derive(Debug)]
struct Buffer {
    lines: Vec<String>,
    cursor: Vec2,
    size: Vec2,
    offset: usize,
}

impl Buffer {
    fn render(&self, term: &mut impl Write) {
        let lines = &self.lines[self.offset..];
        let mut y = 1;
        write!(
            term,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )
        .unwrap();
        for line in lines {
            if y > self.size.y {
                break;
            }

            write!(term, "{}", line).unwrap();
            y += 1;
            write!(term, "{}", termion::cursor::Goto(1, y as u16)).unwrap();
        }
        write!(
            term,
            "{}",
            termion::cursor::Goto(
                (self.cursor.x + 1) as u16,
                (self.cursor.y - self.offset + 1) as u16
            )
        )
        .unwrap();

        term.flush().unwrap();
    }

    fn move_cursor_left(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor.y + 1 == self.lines.len() {
            return;
        }

        let is_edge = self.cursor.y - self.offset == self.size.y - 1;
        self.cursor.y += 1;
        if is_edge {
            self.offset += 1;
        }

        let line = &self.lines[self.cursor.y];
        if self.cursor.x >= line.len() {
            self.cursor.x = if line.is_empty() { 0 } else { line.len() - 1 };
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor.y == 0 {
            return;
        }

        let is_edge = self.cursor.y - self.offset == 0;
        self.cursor.y -= 1;
        if is_edge {
            self.offset -= 1;
        }

        let line = &self.lines[self.cursor.y];
        if self.cursor.x >= line.len() {
            self.cursor.x = if line.is_empty() { 0 } else { line.len() - 1 };
        }
    }

    fn move_cursor_right(&mut self) {
        let line = &self.lines[self.cursor.y];
        if self.cursor.x + 1 < line.len() {
            self.cursor.x += 1;
        }
    }

    fn delete_char(&mut self) {
        let line = &mut self.lines[self.cursor.y];
        if self.cursor.x < line.len() {
            line.remove(self.cursor.x);
        }

        if !line.is_empty() && self.cursor.x >= line.len() {
            self.cursor.x = line.len() - 1;
        }
    }

    fn move_cursor_first_character(&mut self) {
        self.cursor.x = 0;
    }

    fn word_forward(&mut self) {
        if self.lines.is_empty() {
            return;
        }

        let mut x = self.cursor.x;
        let mut line: Vec<char> = self.lines[self.cursor.y].chars().collect();

        let mut starts_in_empty_line = false;
        if line.is_empty() && self.cursor.y + 1 < self.lines.len() {
            self.cursor.x = 0;
            self.cursor.y += 1;
            x = 0;
            line = self.lines[self.cursor.y].chars().collect();
            starts_in_empty_line = true;
        }

        if !starts_in_empty_line {
            let starts_in_alphanumeric = char::is_alphanumeric(line[x]);

            while (starts_in_alphanumeric && char::is_alphanumeric(line[x]))
                || (!starts_in_alphanumeric && !char::is_alphanumeric(line[x]))
            {
                if x + 1 < line.len() {
                    x += 1;
                } else {
                    if self.cursor.y + 1 < self.lines.len() {
                        self.cursor.y += 1;
                        x = 0;
                        line = self.lines[self.cursor.y].chars().collect();
                    }
                    break;
                }
            }
        }

        if !line.is_empty() {
            while char::is_whitespace(line[x]) {
                if x + 1 < line.len() {
                    x += 1;
                } else {
                    if self.cursor.y + 1 < self.lines.len() {
                        self.cursor.y += 1;
                        x = 0;
                        line = self.lines[self.cursor.y].chars().collect();
                    }

                    if line.is_empty() {
                        break;
                    }
                }
            }
        }

        self.cursor.x = x;
    }

    fn join_line(&mut self) {
        if self.cursor.y + 1 < self.lines.len() {
            if !self.lines[self.cursor.y + 1].is_empty() {
                let line = self.lines[self.cursor.y].clone();
                let next_line = self.lines[self.cursor.y + 1].clone();

                if line.is_empty() {
                    self.lines[self.cursor.y] = next_line;
                } else {
                    self.lines[self.cursor.y] = format!("{} {}", line, next_line);
                }

                self.cursor.x = line.len()
            }

            self.lines.remove(self.cursor.y + 1);
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

    fn update(&mut self, evt: Event) -> bool {
        match evt {
            Event::Key(Key::Char('q')) => return true,
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

        false
    }
}

fn write_debug(str: &str) {
    let mut debug_file = File::create("/tmp/vipi.debug").expect("Failed to open debug file");
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

    let buffer = Buffer {
        lines: if let Some(file_path) = env::args().nth(1) {
            lines_from_file(&file_path)
        } else {
            vec![]
        },
        cursor: Vec2::default(),
        size: termion::terminal_size().unwrap().into(),
        offset: 0,
    };

    let mut state = State { buffer };

    state.render(&mut stdout);
    for c in stdin.events() {
        let evt = c.unwrap();
        write_debug(&format!("{:?}\n", evt));

        let quit = state.update(evt);
        if quit {
            break;
        }

        state.render(&mut stdout);

        write_debug(&format!("cursor {:?}\n", state.buffer.cursor));
        write_debug(&format!("size {:?}\n", state.buffer.size));
        write_debug(&format!("offset {:?}\n", state.buffer.offset));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_forward() {
        let mut buffer = Buffer {
            lines: vec!["Word Forward".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
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
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(0, 1));

        // next line starts with space
        buffer = Buffer {
            lines: vec!["word1".to_string(), " word2".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(1, 1));

        // next line is empty
        buffer = Buffer {
            lines: vec!["word1".to_string(), "".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(0, 1));

        // when current line has trailing space
        buffer = Buffer {
            lines: vec!["word1 ".to_string(), "word2".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(0, 1));

        // when current line has trailing space and next starts with space
        buffer = Buffer {
            lines: vec!["; ".to_string(), " word2".to_string()],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(1, 1));

        // when current line has trailing space and next starts with space
        buffer = Buffer {
            lines: vec![
                "weird ".to_string(),
                " ".to_string(),
                " scenario".to_string(),
            ],
            cursor: Vec2::default(),
            size: Vec2::new(100, 100),
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
            offset: 0,
        };

        buffer.word_forward();
        assert_eq!(buffer.cursor, Vec2::new(0, 1));
    }
}
