use std::{collections::HashMap, rc::Rc};
/// There are two ways of controlling the editor: **Series** and **Chords**
/// Series have two varieties:
/// - Compositions
/// - Movements
///
/// Chords are a standalone variety.
///
/// # Compositions
/// series that are initiated by the leader key in either normal
/// or visual mode (ie not insert). You are navigating your way to a
/// specific command. Each keypress will either: 1) Open up a "menu" of
/// more valid keys. 2) Terminate the command (invalid key). or 3) execute
/// a command.
///
/// # Movements
/// Special series which modify the buffer state. they are
/// akin to Vim motions. 100% compatibility with Vim motions
/// is not a requirement here. movements are unique in that they can be repeated
/// when preceded a number. These cannot be executed in insert mode.
///
/// # Chords
/// Short key combinations which can be executed in any mode. ex)
/// <C-c>, <C-v>, etc. Unlike in Emacs, these cannot initiate a composition.
/// a key with a modifier which appears in a composition is not considered
/// a chord. Unfortunately, some of the movement shortcuts utilize modifier keys,
/// so there are some exceptions to this definition.
///
/// ^^^ This will be deprecated! ^^^
/// Everything will be a series. renaming series to chain?

use super::EditorState;

#[derive(Eq, Clone, Copy, Hash)]
pub struct Modifiers {
    pub control: bool,
    pub alt: bool,
    pub logo: bool,
}

impl Modifiers {
    pub fn with_none() -> Self {
        Self {
            control: false,
            alt: false,
            logo: false,
        }
    }
    pub fn with_control() -> Self {
        Self {
            control: true,
            alt: false,
            logo: false,
        }
    }
    pub fn with_alt() -> Self {
        Self {
            control: false,
            alt: true,
            logo: false,
        }
    }
    pub fn with_logo() -> Self {
        Self {
            control: false,
            alt: false,
            logo: true,
        }
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::with_none()
    }
}

impl PartialEq for Modifiers {
    #[rustfmt::skip]
    fn eq(&self, other: &Self) -> bool {
        self.control == other.control
        && self.alt  == other.alt
        && self.logo == other.logo
    }
}

/// A key press which also tracks modifier state.
#[derive(Eq, Clone, Copy, Hash)]
pub struct KeyPress {
    pub key: char,
    pub modifiers: Modifiers,
}

impl From<char> for KeyPress {
    fn from(key: char) -> Self {
        Self {
            key,
            modifiers: Modifiers::with_none(),
        }
    }
}

impl PartialEq for KeyPress {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.modifiers == other.modifiers
    }
}

pub type Chain = HashMap<KeyPress, ChainLink>;

pub enum ChainLink {
    SubChain(Rc<Chain>),
    Func(Box<dyn Fn(&mut EditorState)>),
}

impl<F: Fn(&mut EditorState) + 'static> From<F> for ChainLink {
    fn from(f: F) -> Self {
        Self::Func(Box::new(f))
    }
}
