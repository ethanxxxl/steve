use std::char;

use super::fonts::Font;

// when a font is inserted into the buffer, that font will be applied to any text which succeeds it.
#[derive(Clone)]
pub enum BufferEntry {
    Text(char),
    Font(Font),
}

impl From<char> for BufferEntry {
    fn from(c: char) -> Self {
        BufferEntry::Text(c)
    }
}

/// A text buffer with cached metadata.
///
/// Usage:
/// ```
/// let buffer = Buffer::new(0);
/// buffer.insert_at_cursor('a');
/// buffer.delete_at_cursor();
///
/// let line = Line::new();
/// let line.insert_str("Hello There!!!");
///
/// // put three lines into the buffer
/// buffer.insert_line_below(line.clone());
/// buffer.insert_line_below(line.clone());
/// buffer.insert_line_below(line.clone());
///
/// assert_eq!(buffer.to_string(),
/// "Hello There!!!
/// Hello There!!!
/// Hello There!!!");
///
/// assert_eq!(buffer.move_cursor_down(1), 2);
/// assert_eq!(buffer.move_cursor_down(1), 3);
/// ```
#[derive(Clone)]
pub struct Buffer {
    cursor_pos: (usize, usize),
    lines: Vec<Vec<BufferEntry>>,
}

impl Buffer {
    pub fn new(id: u32) -> Self {
        Self {
            lines: vec![vec![]],
            cursor_pos: (1, 0),
        }
    }

    pub fn insert_at_cursor(&mut self, c: char) {
        // cursor_pos holds a line number and column index. lines start at 1.
        let (mut line_index, mut column_index) = self.cursor_pos;
        line_index -= 1;

        match c {
            '\n' | '\r' => {
                self.lines[line_index].insert(column_index, '\n'.into());
                column_index += 1;

                let newline = self.lines[line_index].split_off(column_index);
                self.lines.insert(line_index + 1, newline);
                column_index = 0;
                line_index += 1;
            }

            '\x08' => {
                if self.lines[line_index].is_empty() && self.lines.len() > 1 {
                    self.lines.remove(line_index);
                    line_index -= 1;
                    self.lines[line_index].pop(); // remove trailing newline
                    column_index = self.lines[line_index].len();
                } else if self.lines[line_index].is_empty() {
                    // we are on the first line. and its empty.
                    // do nothing.
                } else {
                    self.lines[line_index].remove(column_index - 1);
                    column_index -= 1;
                }
            }

            '\t' => {
                for _ in 0..3 {
                    self.lines[line_index].push(' '.into());
                    column_index += 1;
                }
            }

            _ => {
                self.lines[line_index].insert(column_index, c.into());
                column_index += 1;
            }
        }

        self.cursor_pos = (line_index+1, column_index);
    }

    pub fn insert_line_above(&mut self, line: Vec<BufferEntry>) {
        self.lines.insert(self.cursor_pos.0-1, line);
        self.cursor_pos.1 += 1;
    }
    pub fn insert_line_below(&mut self, line: Vec<BufferEntry>) {
        self.lines.insert(self.cursor_pos.0, line);
    }

    pub fn delete_at_cursor(&mut self) {
        self.lines[self.cursor_pos.0 - 1].remove(self.cursor_pos.1);
    }

    pub fn get_current_line(&self) -> &Vec<BufferEntry> {
        &self.lines[self.cursor_pos.0 - 1]
    }

    pub fn get_current_line_mut(&mut self) -> &mut Vec<BufferEntry> {
        &mut self.lines[self.cursor_pos.0 - 1]
    }

    pub fn get_cursor_pos(&self) -> (usize, usize) {
        self.cursor_pos
    }

    pub fn get_lines(&self) -> &Vec<Vec<BufferEntry>> {
        &self.lines
    }
}
