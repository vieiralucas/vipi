#[derive(Debug, PartialEq)]
pub struct CursorLine {
    before: Vec<char>,
    // first item of with_cursor is the cursor position
    with_cursor: Vec<char>,
}

impl CursorLine {
    pub fn from_str(str: &str, char_pos: usize) -> Self {
        if str.is_empty() {
            return Self {
                before: vec![],
                with_cursor: vec![],
            };
        }

        let mut char_pos = char_pos;
        if char_pos >= str.len() {
            char_pos = str.len() - 1
        }

        Self {
            before: str.chars().take(char_pos).collect(),
            with_cursor: str.chars().skip(char_pos).collect(),
        }
    }

    pub fn content_before_cursor(&self) -> String {
        let mut before: String = String::with_capacity(self.before.len());
        for c in self.before.iter() {
            before.push(*c);
        }

        before
    }

    pub fn line(&self) -> String {
        let mut before: String = String::with_capacity(self.before.len());
        for c in self.before.iter() {
            before.push(*c);
        }

        let mut with_cursor: String = String::with_capacity(self.with_cursor.len());
        for c in self.with_cursor.iter() {
            with_cursor.push(*c);
        }

        format!("{}{}", before, with_cursor)
    }

    pub fn len(&self) -> usize {
        self.before.len() + self.with_cursor.len()
    }

    pub fn set_x(&mut self, x: usize) {
        if x >= self.len() {
            return;
        }

        while self.x() > x {
            self.move_left();
        }

        while self.x() < x {
            self.move_right(false);
        }
    }

    pub fn x(&self) -> usize {
        self.before.len()
    }

    pub fn move_left(&mut self) -> bool {
        if let Some(cursor_char) = self.before.pop() {
            self.with_cursor.insert(0, cursor_char);
            true
        } else {
            false
        }
    }

    pub fn move_right(&mut self, allow_one_off: bool) -> bool {
        if !allow_one_off && self.with_cursor.len() <= 1 {
            false
        } else if let Some(cursor_char) = self.with_cursor.first() {
            self.before.push(*cursor_char);
            self.with_cursor.remove(0);
            true
        } else {
            false
        }
    }

    pub fn delete_char(&mut self) {
        if self.with_cursor.is_empty() {
            return;
        }

        self.with_cursor.remove(0);

        if self.with_cursor.is_empty() {
            if let Some(last) = self.before.pop() {
                self.with_cursor.push(last)
            }
        }
    }

    pub fn is_at_whitespace(&self) -> bool {
        self.with_cursor
            .first()
            .map(|cursor_char| char::is_whitespace(*cursor_char))
            .unwrap_or(false)
    }

    pub fn is_at_alphanumeric(&self) -> bool {
        self.with_cursor
            .first()
            .map(|cursor_char| char::is_alphanumeric(*cursor_char))
            .unwrap_or(false)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert_char(&mut self, c: char) {
        self.before.push(c);
    }

    pub fn clamp(&mut self) {
        if !self.with_cursor.is_empty() {
            return;
        }

        if let Some(c) = self.before.pop() {
            self.with_cursor.insert(0, c)
        }
    }

    pub fn backspace(&mut self) -> bool {
        if self.is_empty() {
            false
        } else if self.with_cursor.is_empty() {
            self.before.pop().is_some()
        } else if self.move_left() {
            self.delete_char();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::CursorLine;

    #[test]
    fn from_str_empty() {
        let cursor_line = CursorLine::from_str("", 0);

        assert_eq!(cursor_line.before, vec![]);
        assert_eq!(cursor_line.with_cursor, vec![]);
    }

    #[test]
    fn from_str_start() {
        let cursor_line = CursorLine::from_str("012", 0);

        assert_eq!(cursor_line.before, vec![]);
        assert_eq!(cursor_line.with_cursor, vec!['0', '1', '2']);
    }

    #[test]
    fn from_str_middle() {
        let cursor_line = CursorLine::from_str("012", 1);

        assert_eq!(cursor_line.before, vec!['0']);
        assert_eq!(cursor_line.with_cursor, vec!['1', '2']);
    }

    #[test]
    fn from_str_end() {
        let cursor_line = CursorLine::from_str("012", 2);

        assert_eq!(cursor_line.before, vec!['0', '1']);
        assert_eq!(cursor_line.with_cursor, vec!['2']);
    }

    #[test]
    fn from_str_overflowed() {
        let cursor_line = CursorLine::from_str("012", 3);

        assert_eq!(cursor_line.before, vec!['0', '1']);
        assert_eq!(cursor_line.with_cursor, vec!['2']);
    }

    #[test]
    fn is_empty_true() {
        let cursor_line = CursorLine::from_str("", 0);

        let is_empty = cursor_line.is_empty();

        assert_eq!(is_empty, true);
    }

    #[test]
    fn is_empty_off_by_one() {
        let mut cursor_line = CursorLine::from_str("0", 0);
        cursor_line.move_right(true);

        let is_empty = cursor_line.is_empty();

        assert_eq!(is_empty, false);
    }

    #[test]
    fn is_empty_false() {
        let mut cursor_line = CursorLine::from_str("01234", 0);

        let is_empty = cursor_line.is_empty();
        assert_eq!(is_empty, false);

        while cursor_line.move_right(false) {
            let is_empty = cursor_line.is_empty();
            assert_eq!(is_empty, false);
        }
    }

    #[test]
    fn move_right_start() {
        let mut cursor_line = CursorLine::from_str("012", 0);

        let result = cursor_line.move_right(false);

        assert_eq!(result, true);
        assert_eq!(cursor_line, CursorLine::from_str("012", 1));
    }

    #[test]
    fn move_right_middle() {
        let mut cursor_line = CursorLine::from_str("012", 1);

        let result = cursor_line.move_right(false);

        assert_eq!(result, true);
        assert_eq!(cursor_line, CursorLine::from_str("012", 2));
    }

    #[test]
    fn move_right_end() {
        let mut cursor_line = CursorLine::from_str("012", 2);

        let result = cursor_line.move_right(false);

        assert_eq!(result, false);
        assert_eq!(cursor_line, CursorLine::from_str("012", 2));
    }

    #[test]
    fn move_right_end_allow_one_off() {
        let mut cursor_line = CursorLine::from_str("012", 2);

        let result = cursor_line.move_right(true);

        assert_eq!(result, true);
        assert_eq!(cursor_line.before, vec!['0', '1', '2']);
        assert_eq!(cursor_line.with_cursor, vec![]);
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

    #[test]
    fn insert_char() {
        let mut cursor_line = CursorLine::from_str("", 0);

        cursor_line.insert_char('a');

        assert_eq!(cursor_line.before, vec!['a']);
    }

    #[test]
    fn clamp() {
        let mut cursor_line = CursorLine::from_str("", 0);

        cursor_line.insert_char('a');
        cursor_line.clamp();

        assert_eq!(cursor_line.before, vec![]);
        assert_eq!(cursor_line.with_cursor, vec!['a']);
    }

    #[test]
    fn clamp_no_op() {
        let mut cursor_line = CursorLine::from_str("abc", 0);

        cursor_line.clamp();

        assert_eq!(cursor_line.before, vec![]);
        assert_eq!(cursor_line.with_cursor, vec!['a', 'b', 'c']);
    }

    #[test]
    fn backspace() {
        let mut cursor_line = CursorLine::from_str("abc", 2);

        assert_eq!(cursor_line.backspace(), true);

        assert_eq!(cursor_line.before, vec!['a']);
        assert_eq!(cursor_line.with_cursor, vec!['c']);
    }

    #[test]
    fn backspace_when_off_by_one() {
        let mut cursor_line = CursorLine::from_str("abc", 2);
        cursor_line.move_right(true);

        assert_eq!(cursor_line.backspace(), true);

        assert_eq!(cursor_line.before, vec!['a', 'b']);
        assert_eq!(cursor_line.with_cursor, vec![]);
    }

    #[test]
    fn backspace_start() {
        let mut cursor_line = CursorLine::from_str("abc", 0);

        assert_eq!(cursor_line.backspace(), false);
        assert_eq!(cursor_line.before, vec![]);
        assert_eq!(cursor_line.with_cursor, vec!['a', 'b', 'c']);
    }
}
