use super::fonts::Font;

#[derive(Clone)]
pub struct Token {
    text: std::ops::Range<usize>,
    font: Font,
    tags: Option<Vec<String>>,
}

/// once the line has been edited, the edited flag becomes true.
/// this marks the line to be processed by systems again.
///
#[derive(Clone)]
pub struct Line {
    text: String,
    edited: bool,
}

impl Line {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            edited: false,
        }
    }
    /// inserts `c` at index. index is the character column. (not the byte position)
    /// panics if index is out of range
    pub fn insert(&mut self, index: usize, c: char) {
        self.edited = true;
        self.text.insert(self.get_byte_position(index), c);
    }

    /// inserts `s` at index. index is the character column. (not the byte position)
    /// panics if index is out of range
    pub fn insert_str(&mut self, index: usize, s: &str) {
        self.edited = true;
        self.text.insert_str(self.get_byte_position(index), s);
    }

    pub fn push(&mut self, c: char) {
        self.edited = true;
        self.text.push(c);
    }

    pub fn pop(&mut self) -> Option<char> {
        self.edited = true;
        self.text.pop()
    }

    pub fn push_str(&mut self, s: &str) {
        self.edited = true;
        self.text.push_str(s);
    }

    pub fn split_off(&mut self, index: usize) -> Line {
        self.edited = true;
        Line {
            text: self.text.split_off(self.get_byte_position(index)),
            edited: true,
        }
    }

    /// removes character at index, and returns it. index is the character column (not the byte position)
    /// panics if index is out of range
    pub fn remove(&mut self, index: usize) -> char {
        self.edited = true;
        self.text.remove(self.get_byte_position(index))
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }

    /// returns a mutable reference to the text string enclosed. Marks the line as edited.
    pub fn get_text_mut(&mut self) -> &mut String {
        self.edited = true;
        &mut self.text
    }

    /// internal helper function to get the byte position of a character position in the string.
    pub fn get_byte_position(&self, index: usize) -> usize {
        self.text.char_indices().nth(index).expect(&format!(
            "ERROR: index out of bounds, index was {}, but len is {}",
            index,
            self.text.len()
        )).0
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl From<String> for Line {
    fn from(text: String) -> Line {
        Line {
            text,
            edited: true,
        }
    }
}

impl From<&str> for Line {
    fn from(text: &str) -> Line {
        Line {
            text: text.to_string(),
            edited: true,
        }
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
    id: u32,
    cursor_pos: (usize, usize),

    lines: Vec<Line>
}

impl Buffer {
    pub fn new(id: u32) -> Self {
        Self {
            lines: vec![Line::new()],
            cursor_pos: (1, 0),
            id,
        }
    }

    pub fn insert_at_cursor(&mut self, c: char) {
        // cursor_pos holds a line number and column index. lines start at 1.
        let (mut line_index, mut column_index) = self.cursor_pos;
        line_index -= 1;

        match c {
            '\n' | '\r' => {
                self.lines[line_index].insert(column_index, '\n');
                column_index += 1;

                let newline = self.lines[line_index].split_off(column_index);
                self.lines.insert(line_index + 1, newline);
                column_index = 0;
                line_index += 1;
            }

            '\x08' => {
                if self.lines[line_index].get_text().is_empty() && self.lines.len() > 1 {
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
                    self.lines[line_index].push(' ');
                    column_index += 1;
                }
            }

            _ => {
                self.lines[line_index].insert(column_index, c);
                column_index += 1;
            }
        }
        self.lines[line_index].insert(column_index, c);
    }
    pub fn insert_line_above(&mut self, line: Line) {
        self.lines.insert(self.cursor_pos.0-1, line);
        self.cursor_pos.1 += 1;
    }
    pub fn insert_line_below(&mut self, line: Line) {
        self.lines.insert(self.cursor_pos.0, line);
    }

    pub fn delete_at_cursor(&mut self) {
        self.lines[self.cursor_pos.0 - 1].remove(self.cursor_pos.1);
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_current_line(&self) -> &Line {
        &self.lines[self.cursor_pos.0 - 1]
    }

    pub fn get_current_line_mut(&mut self) -> &mut Line {
        &mut self.lines[self.cursor_pos.0 - 1]
    }

    pub fn get_cursor_pos(&self) -> (usize, usize) {
        self.cursor_pos
    }

    pub fn get_lines(&self) -> &Vec<Line> {
        &self.lines
    }
}
