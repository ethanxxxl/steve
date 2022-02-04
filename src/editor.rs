use std::{collections::HashMap, rc::Rc};
use std::fmt;

mod graphics;
pub mod keymaps;
mod highlighter;
pub mod buffer;
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
    pub normal_chain: Vec<(KeyPress, Chain)>,
    pub visual_chain: Vec<(KeyPress, Chain)>,
    pub insert_chain: Vec<(KeyPress, Chain)>,

    pub next_id: u32,
    pub active_buffer: Buffer,
    pub mode: EditMode,
    pub status_line: String,
}

// The problem with this whole approach is that you have this EditorState struct, which is trying to manipulate itself.

impl EditorState {
    pub fn new() -> Self {
        let mut default_buffer = Buffer::new(0);
        let mut normal_chain = Chain::new();
        let mut insert_chain = Chain::new();
        let mut visual_chain = Chain::new();

        normal_chain.insert('i'.into(), (|s: &mut EditorState| s.set_insert_mode()).into());
        insert_chain.insert('\x1b'.into(), (|s: &mut EditorState| s.set_normal_mode()).into());


        let mut theme: HashMap<Font, FontDefinition> = HashMap::new();

        theme.insert(Font::Normal, Default::default());

        EditorState {
            theme,
            normal_chain: vec![(' '.into(), normal_chain)],
            visual_chain: vec![(' '.into(), visual_chain)],
            insert_chain: vec![(' '.into(), insert_chain)],
            next_id: 1,
            active_buffer: Buffer::new(0),
            mode: EditMode::Normal,
            status_line: String::new(),
        }
    }
    /// takes a keystroke, processes it, and alters state according to internal state and
    /// the keystroke.
    pub fn process_keystroke(&mut self, key: char) {
        match self.mode {
            EditMode::Normal => {
                if let Some(func) = Self::validate_chain(key.into(), &mut self.normal_chain) {
                    (*func)(self);
                }
            }
            EditMode::Insert => {
                if let Some(func) = Self::validate_chain(key.into(), &mut self.insert_chain) {
                    (*func)(self);
                } else {
                    self.active_buffer.insert_at_cursor(key);
                }
            }
            EditMode::Visual => {}
            EditMode::Command => {}
        }
    }

    // returns true if key was a valid input for chain.
    fn validate_chain(key: KeyPress, chain: &mut Vec<(KeyPress, Chain)>)
                      -> Option<Box<dyn Fn(&mut EditorState)>> {
        let end = chain.len()-1;
        if let Some((key, entry)) = chain[end].1.remove_entry(&key) {
            match entry {
                ChainLink::Func(func) => {
                    // fold the chain back into the root.
                    for i in (1..=end).rev() {
                        let subchain = chain.pop().expect("premature end of chain");
                        chain[i-1].1.insert(subchain.0, ChainLink::SubChain(subchain.1));
                    }

                    Some(func)
                },
                // not a function, so add this layer to the vector.
                ChainLink::SubChain(subchain) => {
                    chain.push((key, subchain));
                    None
                },
            }
        } else {
            None
        }
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
