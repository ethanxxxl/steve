use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::PhysicalSize
};
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};

use pixels::{SurfaceTexture, PixelsBuilder};

struct Buffer {
    // lines : columns
    data: Vec<Vec<char>>,
    cursor_pos: (usize, usize),
}

impl Buffer {
    fn new() -> Self {
        Self {
            data: vec![vec![]],
            cursor_pos: (1, 0),
        }
    }

    /// inserts `text` at the position of the cursor
    fn insert(&mut self, text: String) {
        // cursor_pos holds a line number and column index. lines start at 1.
        let (mut line_index, mut column_index) = self.cursor_pos;
        line_index -= 1;

        for t in text.chars() {
        println!("{}:{}\t\t'{}' ({})", line_index+1, column_index, t, t as u32);
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
                    column_index = self.data[line_index].len();
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

    fn flatten(&self) -> String {
        self.data
            .iter()
            .flatten()
            .collect::<String>()
    }
}

//enum EditMode {
//    Normal,
//    Insert,
//    Visual,
//}
//
//struct EditorState {
//    mode: EditMode,
//    buffer: Buffer,
//    status_line: Buffer,
//}
//
//impl EditorState {
//
//}

//fn insert_mode(_buffer: &mut Buffer, _key_event: &KeyboardInput) {
//
//}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let PhysicalSize { width, height } = window.inner_size();

    let mut staging_belt = wgpu::util::StagingBelt::new(1024);
    let local_pool = futures::executor::LocalPool::new();
    let local_spawner = local_pool.spawner();

    let surface_texture = SurfaceTexture::new(width, height, &window);
    let mut pixels = PixelsBuilder::new(width, height, surface_texture)
        .texture_format(wgpu::TextureFormat::Bgra8UnormSrgb)
        .build()
        .expect("Error Creating Context");

    let context = pixels.context();

    let ubuntu = ab_glyph::FontArc::try_from_slice(include_bytes!(
        //"/usr/share/fonts/TTF/FiraCode-Regular.ttf"
        "/usr/share/fonts/ubuntu/Ubuntu-R.ttf"
    )).expect("Error Loading Font");

    let mut glyph_brush = GlyphBrushBuilder::using_font(ubuntu)
        .build(&context.device, context.texture_format);

    let mut main_buffer = Buffer::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event,// this was `ref event` for some reason...
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            WindowEvent::ReceivedCharacter(character) =>{
                main_buffer.insert(character.to_string());
                window.request_redraw();
            },
            WindowEvent::Resized(PhysicalSize { width, height } ) => {
                pixels.resize_buffer(width, height);
                pixels.resize_surface(width, height);
                window.request_redraw();
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {
            pixels.get_frame().chunks_mut(4).for_each(|p| {
                p.copy_from_slice(&[255, 255, 255, 255]);
            });

            let PhysicalSize { width, height } = window.inner_size();

            let text = main_buffer.flatten();
            let text = Text::new(text.as_str())
                .with_scale(20.0)
                .with_color([0.0, 0.0, 0.0, 1.0]);

            glyph_brush.queue(Section {
                screen_position: (0.0, 0.0),
                bounds: (width as f32, 0.5*height as f32),
                text: vec![text],
                ..Section::default()
            });

            //let (line, col) = main_buffer.cursor_pos;

            pixels.render_with(|encoder, render_target, context| {
                context.scaling_renderer.render(encoder, render_target);

                glyph_brush.draw_queued(
                    &context.device,
                    &mut staging_belt,
                    encoder,
                    render_target,
                    width,
                    height,
                ).expect("Draw queued");

                staging_belt.finish();
            }).unwrap();

            use futures::task::SpawnExt;
                local_spawner
                    .spawn(staging_belt.recall())
                    .expect("Recall staging belt");

        },

        Event::MainEventsCleared => {
            //window.request_redraw();
        }
        _ => {
            *control_flow = winit::event_loop::ControlFlow::Wait;
        }
    });
}
