struct Color(f32, f32, f32);
impl From<(f32, f32, f32)> for Color {
    fn from(o: (f32, f32, f32)) -> Self {
        Self(o.0, o.1, o.2)
    }
}

#[derive(Clone, Copy)]
pub struct FontDefinition {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,

    pub typeface: TypeFace,
    pub color: [f32; 4],
    pub size: f32,
}

impl Default for FontDefinition {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,

            typeface: TypeFace::Monospace,
            color: [1.0, 1.0, 1.0, 1.0],
            size: 15.0,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TypeFace {
    Monospace,
    Serif,
    SansSerif,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Font {
    Normal,
    Bold,
    Italic,
    BoldItalic,
    Comment,
    Number,
    String,
    Keyword,
    Variable,
    Function,
    Structure,
    // ... etc.
}
