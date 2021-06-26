extern crate termion;

use termion::event::{Key, Event};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};

struct Cursor {
    x: u16,
    y: u16
}
struct State {
    lines: Vec<String>,
    cursor: Cursor
}

impl State {
    fn render(&self, term: &mut impl Write) {
        let mut y = 1;
        write!(term, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
        for line in &self.lines {
            write!(term, "{}", line).unwrap();
            y += 1;
            write!(term, "{}", termion::cursor::Goto(1, y)).unwrap();
        }
        write!(term, "{}", termion::cursor::Goto(self.cursor.x, self.cursor.y)).unwrap();

        term.flush().unwrap();
    }
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut state = State {
        lines: vec!["Hello, World".to_string(), "Line below".to_string(), "Line 3".to_string()],
        cursor: Cursor { x: 1, y: 1 }
    };


    state.render(&mut stdout);

    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(Key::Char('q')) => break,
            Event::Key(Key::Char('h')) => {
                state.cursor.x -= 1;
            },
            Event::Key(Key::Char('j')) => {
                state.cursor.y += 1;
            },
            Event::Key(Key::Char('k')) => {
                state.cursor.y -= 1;
            },
            Event::Key(Key::Char('l')) => {
                state.cursor.x += 1;
            },
            _ => {}
        }

        state.render(&mut stdout);
    }
}
