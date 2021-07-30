use std::fmt;

pub struct Buffer {
    /// lines : columns
    data: Vec<Vec<char>>,
    cursor_pos: (usize, usize),
    file_path: Option<String>,
    id: u32,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            data: vec![vec![]],
            cursor_pos: (1, 0),
            file_path: None,
            id: 0,
        }
    }

    /// inserts `text` at the position of the cursor
    pub fn insert(&mut self, text: String) {
        // cursor_pos holds a line number and column index. lines start at 1.
        let (mut line_index, mut column_index) = self.cursor_pos;
        line_index -= 1;

        for t in text.chars() {
        match t {
            '\n' | '\r' => {
                self.data[line_index].insert(column_index, '\n');
                column_index += 1;

                let newline = self.data[line_index].split_off(column_index);
                self.data.insert(line_index+1, newline);
                column_index = 0;
                line_index += 1;
            }

            '\x08' => {
                if self.data[line_index].is_empty() && self.data.len() > 1 {
                    self.data.remove(line_index);
                    line_index -= 1;
                    self.data[line_index].pop(); // remove trailing newline
                    column_index = self.data[line_index].len();
                } else if self.data[line_index].is_empty() {
                    // we are on the first line. and its empty.
                    // do nothing.
                } else {
                    self.data[line_index].remove(column_index-1);
                    column_index -= 1;
                }
            }

            _ => {
                self.data[line_index].insert(column_index, t);
                column_index += 1;
            }
        }}

        self.cursor_pos = (line_index+1, column_index);
    }

    pub fn flatten(&self) -> String {
        self.data
            .iter()
            .flatten()
            .collect::<String>()
    }
}

/// Pressing the leader key initiates a *sequence*. a sequence of keys
///  may be mapped to complete an action. This is not available in insert mode.
///
///  Movement commands are not preceded by the leader key.
struct Keymap {
    leader: char,
}

pub enum EditMode {
    Normal,
    Insert,
    Visual,
}

impl fmt::Display for EditMode {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Normal => write!(formatter, "NORMAL"),
            Self::Insert => write!(formatter, "INSERT"),
            Self::Visual => write!(formatter, "VISUAL"),
        }
    }
}

pub struct EditorState {
    pub active_buffer: Box<Buffer>,
    pub mode: EditMode,
    pub status_line: String,
    buffers: Vec<Box<Buffer>>,
    next_id: u32,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            active_buffer: Box::new(Buffer::new()),
            mode: EditMode::Insert,
            buffers: Vec::new(),
            next_id: 1,
            status_line: String::new(),
        }
    }
    /// Creates a new empty buffer and returns its ID
    pub fn create_empty_buffer(&mut self) -> u32 {
        let mut new_buffer = Buffer::new();
        new_buffer.id = self.next_id;
        self.next_id += 1;

        let new_buffer = Box::new(new_buffer);
        self.buffers.push(new_buffer);
        self.buffers.sort_by_key(|buf| buf.id );

        self.next_id - 1
    }

    pub fn change_buffer(&mut self, buffer_id: u32) -> Result<(), String> {
        self.buffers.sort_by_key(|buf| buf.id);

        let index = self.buffers.binary_search_by_key(&buffer_id, |buf| buf.id);
        match index {
            Err(_) => {
                let msg = format!("buffer with id {} does not exist.", buffer_id);
                Err(msg.to_string())
            },
            Ok(i) => {
                std::mem::swap(&mut self.active_buffer, &mut self.buffers[i]);
                Ok(())
            },
        }
    }

    pub fn get_buffer_list(&mut self) -> Vec<(u32, Option<String>)> {
        self.buffers
            .iter()
            .map(|buffer| {
                (buffer.id, buffer.file_path.clone())
            })
            .collect()
    }

    pub fn update(&mut self) {
        let (line, col) = self.active_buffer.cursor_pos;
        self.status_line =
            format!("[{}] [{}:{}]", self.mode, line, col);
    }
}
