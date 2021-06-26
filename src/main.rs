extern crate termion;

use std::fs::File;
use std::io::{stdin, stdout, Write};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

#[derive(Default, Debug)]
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
            self.cursor.x = line.len() - 1;
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
            self.cursor.x = line.len() - 1;
        }
    }

    fn move_cursor_right(&mut self) {
        let line = &self.lines[self.cursor.y];
        if self.cursor.x + 1 < line.len() {
            self.cursor.x += 1;
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

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let buffer = Buffer {
        lines: vec![
            "Hello, World".to_string(),
            "Line below".to_string(),
            "Line 3".to_string(),
            "Line four".to_string(),
            "Line 5".to_string(),
            "Line 6".to_string(),
            "Line 7".to_string(),
            "Line 8".to_string(),
        ],
        cursor: Vec2::default(),
        size: termion::terminal_size().unwrap().into(),
        offset: 0,
    };

    let mut state = State { buffer };

    state.render(&mut stdout);
    for c in stdin.events() {
        let evt = c.unwrap();
        let quit = state.update(evt);
        if quit {
            break;
        }

        state.render(&mut stdout);

        write_debug(&format!("{:?}\n", state));
    }
}
