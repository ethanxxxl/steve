use std::ops::Range;
use crate::editor::fonts;
use crate::editor::buffer::Buffer;

struct Highlight {
    range: Range<usize>,
    font: fonts::Font,
}

type Line = Vec<Highlight>;

enum Method {

}

struct Highlighter {
    cache: Vec<Line>,
}

impl Highlighter {
    /// Runs the highlighter on the entire buffer
    pub fn highlight_buffer(&mut self, buffer: &Buffer) {
    }

    pub fn highlight_line(&mut self, buffer: &Buffer, line: usize) {

    }
}
