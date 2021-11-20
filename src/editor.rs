use std::{collections::HashMap, rc::Rc};
use std::fmt;

mod graphics;
mod keymaps;
mod highlighter;
mod buffer;
mod fonts;

use buffer::Buffer;
use fonts::{Font, FontDefinition};

use keymaps::*;

/// This will need to be changed to handle L/R in the future.
#[derive(Clone, Copy)]
pub enum EditMode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl fmt::Display for EditMode {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Normal => write!(formatter, "NORMAL"),
            Self::Insert => write!(formatter, "INSERT"),
            Self::Visual => write!(formatter, "VISUAL"),
            Self::Command => write!(formatter, "COMMAND"),
        }
    }
}

pub struct EditorState {
    pub theme: HashMap<Font, FontDefinition>,
    buffers: Vec<Box<Buffer>>,
    normal_chain: Rc<Chain>,
    visual_chain: Rc<Chain>,
    insert_chain: Rc<Chain>,

    next_id: u32,
    pub active_buffer: Box<Buffer>,
    pub mode: EditMode,
    pub status_line: String,

    cur_subchain: Rc<Chain>,
}

// The problem with this whole approach is that you have this EditorState struct, which is trying to manipulate itself.

impl EditorState {
    pub fn new() -> Self {
        let mut theme = HashMap::new();
        theme.insert(Font::Normal, FontDefinition::default());

        let normal_chain = Rc::new(Chain::new());
        let insert_chain = Rc::new(Chain::new());
        let visual_chain = Rc::new(Chain::new());

        (*normal_chain).insert('i'.into(), (|s: &mut Self| s.set_insert_mode()).into());
        (*insert_chain).insert('\x1b'.into(), (|s: &mut Self| s.set_normal_mode()).into());

        Self {
            normal_chain,
            insert_chain,
            visual_chain,
            buffers: Vec::new(),
            active_buffer: Box::new(Buffer::new(0)),
            mode: EditMode::Normal,
            next_id: 1,
            theme,
            status_line: String::new(),

            cur_subchain: normal_chain,
        }
    }

    /// takes a keystroke, processes it, and alters state according to internal state and
    /// the keystroke.
    pub fn process_keystroke(&mut self, key: char) {
        match self.mode {
            EditMode::Normal => {
                self.validate_chain(key.into(), self.normal_chain);
            }
            EditMode::Insert => {
                if !self.validate_chain(key.into(), self.insert_chain) {
                    self.active_buffer.insert_at_cursor(key);
                }
            }
            EditMode::Visual => {}
            EditMode::Command => {}
        }
    }

    // returns true if key was a valid input for chain.
    fn validate_chain(&mut self, key: KeyPress, root_chain: Rc<Chain>) -> bool {
        if let Some(entry) = self.cur_subchain.get(&key) {
            match entry {
                ChainLink::Func(func) => {
                    (*func)(self);
                    self.cur_subchain = root_chain.clone();
                },
                ChainLink::SubChain(subchain) => self.cur_subchain = (*subchain).clone()
            };

            true
        } else {
            false
        }
    }

    /// Creates a new empty buffer and returns its ID
    pub fn create_empty_buffer(&mut self) -> u32 {
        let new_buffer = Buffer::new(self.next_id);
        self.next_id += 1;

        let new_buffer = Box::new(new_buffer);
        self.buffers.push(new_buffer);
        self.buffers.sort_by_key(|buf| buf.get_id());

        self.next_id - 1
    }

    pub fn change_buffer(&mut self, buffer_id: u32) -> Result<(), String> {
        self.buffers.sort_by_key(|buf| buf.get_id());

        let index = self
            .buffers
            .binary_search_by_key(&buffer_id, |buf| buf.get_id());
        match index {
            Err(_) => {
                let msg = format!("buffer with id {} does not exist.", buffer_id);
                Err(msg.to_string())
            }
            Ok(i) => {
                std::mem::swap(&mut self.active_buffer, &mut self.buffers[i]);
                Ok(())
            }
        }
    }

    pub fn get_buffer_list(&mut self) -> Vec<(u32, Option<String>)> {
        self.buffers
            .iter()
            .map(|buffer| (buffer.get_id(), None))
            .collect()
    }

    pub fn set_mode(&mut self, new_mode: EditMode) {
        self.mode = new_mode
    }
    pub fn set_insert_mode(&mut self) {
        self.mode = EditMode::Insert
    }
    pub fn set_normal_mode(&mut self) {
        self.mode = EditMode::Normal
    }
    pub fn set_visual_mode(&mut self) {
        self.mode = EditMode::Visual
    }
    pub fn set_command_mode(&mut self) {
        self.mode = EditMode::Command
    }

    pub fn get_mode(&mut self) -> EditMode {
        self.mode
    }

    pub fn update(&mut self) {
        let (line, col) = self.active_buffer.get_cursor_pos();
        self.status_line = format!("[{}] [{}:{}]", self.mode, line, col);
    }
}
