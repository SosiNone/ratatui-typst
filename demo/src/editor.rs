use ratatui::{crossterm::event::KeyCode, text::Line};

/// A small multiline editor. The cursor is always a UTF-8 byte boundary.
pub struct Editor {
    pub text: String,
    pub cursor: usize,
}

impl Editor {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            cursor: text.len(),
        }
    }

    pub fn position(&self) -> (usize, usize) {
        let before = &self.text[..self.cursor];
        let row = before.bytes().filter(|&byte| byte == b'\n').count();
        let column = Line::from(before.rsplit('\n').next().unwrap_or_default()).width();
        (row, column)
    }

    pub fn insert(&mut self, text: &str) {
        self.text.insert_str(self.cursor, text);
        self.cursor += text.len();
    }

    pub fn handle(&mut self, code: KeyCode) {
        let previous = || {
            self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map_or(0, |(i, _)| i)
        };
        let next = || {
            self.cursor
                + self.text[self.cursor..]
                    .chars()
                    .next()
                    .map_or(0, char::len_utf8)
        };
        match code {
            KeyCode::Char(character) => self.insert(&character.to_string()),
            KeyCode::Left => self.cursor = previous(),
            KeyCode::Right => self.cursor = next(),
            KeyCode::Backspace if self.cursor > 0 => {
                let start = previous();
                self.text.drain(start..self.cursor);
                self.cursor = start;
            }
            KeyCode::Delete if self.cursor < self.text.len() => {
                self.text.drain(self.cursor..next());
            }
            KeyCode::Home => {
                self.cursor = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1)
            }
            KeyCode::End => {
                self.cursor += self.text[self.cursor..]
                    .find('\n')
                    .unwrap_or(self.text.len() - self.cursor)
            }
            KeyCode::Up | KeyCode::Down => self.move_line(code == KeyCode::Down),
            _ => {}
        }
    }

    fn move_line(&mut self, down: bool) {
        let start = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1);
        let column = self.text[start..self.cursor].chars().count();
        let target = if down {
            let Some(end) = self.text[self.cursor..].find('\n') else {
                return;
            };
            self.cursor + end + 1
        } else {
            if start == 0 {
                return;
            }
            self.text[..start - 1].rfind('\n').map_or(0, |i| i + 1)
        };
        let line = self.text[target..].split('\n').next().unwrap_or_default();
        self.cursor = target
            + line
                .char_indices()
                .nth(column)
                .map_or(line.len(), |(i, _)| i);
    }
}
