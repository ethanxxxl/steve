use crate::editor::*;
use wgpu_glyph::{Text, ab_glyph::PxScale};

// always assume the text buffer starts at 0,0 in the pixel buffer.
//pub fn draw_cursor<'a>(editor_state: EditorState, window_buffer: &'a [u8], window_size: PhysicalSize<u32>) -> &'a[u8]{
//    let style_marker = editor_state.active_buffer.get_style_at_cursor();
//    let style = editor_state.theme.get(&style_marker).expect("invalid style marker");
//
//    // TODO make this match the theme color
//    let cursor = vec![1.0 ; (style.size * 3.0 * 4.0) as usize ];
//}

impl EditorState{
    // returns a buffer of visible text to which will be displayed on the screen.
    pub fn get_display_buffer(&self) -> Buffer {

        // other plugins / systems will have a chance to alter the text before it is displayed.

        let mut display_buffer = self.active_buffer.clone();
        // draw the cursor
        //let cursor_pos = display_buffer.get_cursor_pos();
        //let current_line = display_buffer.get_current_line_mut();
        //if current_line.len() == cursor_pos.1 {
        //    current_line.push('\u{2588}');
        //} else {
        //    let byte_pos = current_line.char_indices().skip(cursor_pos.1).next().unwrap().0;
        //    current_line.replace_range(byte_pos..byte_pos, "\u{2588}");
        //}

        display_buffer
    }

    pub fn get_section_text<'a>(&self, display_buffer: &'a Buffer) -> Vec<Text<'a>> {
        // now I have a vector with all the strings, and their styles.
        let editor_theme = self.theme.clone();

        let mut v = Vec::new();

        for line in display_buffer.get_lines() {
            let t = Text::new(line.as_str())
                .with_color([1.0,1.0,1.0,1.0])
                .with_scale(30.0)
                .with_font_id(wgpu_glyph::FontId(0));

            v.push(t);

        }

        v

        //display_buffer
            //.get_lines()
            //.iter()
            //.fold(Vec::new(), |mut v, line| {
                //let t = Text::new(line.as_str())
                    //.with_scale(30.0)
                    //.with_color([1.0, 1.0, 1.0, 1.0])
                    //.with_font_id(wgpu_glyph::FontId(0));
//
                //v.push(t);
                //v
        //})
    }
}
