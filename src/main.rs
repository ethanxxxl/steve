use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use pixels::{PixelsBuilder, SurfaceTexture};

mod editor;
use editor::EditorState;

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

    let bookerly = ab_glyph::FontArc::try_from_slice(include_bytes!(
        "/usr/share/fonts/TTF/Bookerly-Regular.ttf"
    ))
    .expect("Error Loading Font");

    let fira_code = ab_glyph::FontArc::try_from_slice(include_bytes!(
        "/usr/share/fonts/TTF/FiraCode-Regular.ttf"
    ))
    .expect("Error Loading Font");

    let mut glyph_brush = GlyphBrushBuilder::using_fonts(vec![bookerly, fira_code])
        .build(&context.device, context.texture_format);

    let mut editor_state = EditorState::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event, // this was `ref event` for some reason...
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::ReceivedCharacter(character) => {
                editor_state.process_keystroke(character);
                window.request_redraw();
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                pixels.resize_buffer(width, height);
                pixels.resize_surface(width, height);
                window.request_redraw();
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {
            pixels.get_frame().chunks_mut(4).for_each(|p| {
                p.copy_from_slice(&[0, 0, 0, 255]);
            });

            let PhysicalSize { width, height } = window.inner_size();
            editor_state.update();

            use wgpu_glyph::FontId;
            let status_text = Text::new(editor_state.status_line.as_str())
                .with_scale(20.0)
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_font_id(FontId(1));

            glyph_brush.queue(Section {
                screen_position: (0.0, height as f32 - 20.0),
                bounds: (width as f32, 25.0),
                text: vec![status_text],
                ..Section::default()
            });

            let display_buffer = editor_state.get_display_buffer();

            glyph_brush.queue(Section {
                screen_position: (0.0, 0.0),
                bounds: (width as f32, height as f32),
                text: editor_state.get_section_text(&display_buffer),
                ..Default::default()
            });

            pixels
                .render_with(|encoder, render_target, context| {
                    context.scaling_renderer.render(encoder, render_target);

                    glyph_brush
                        .draw_queued(
                            &context.device,
                            &mut staging_belt,
                            encoder,
                            render_target,
                            width,
                            height,
                        )
                        .expect("Draw queued");

                    staging_belt.finish();
                })
                .unwrap();

            use futures::task::SpawnExt;
            local_spawner
                .spawn(staging_belt.recall())
                .expect("Recall staging belt");
        }

        Event::MainEventsCleared => {
            //window.request_redraw();
        }
        _ => {
            *control_flow = winit::event_loop::ControlFlow::Wait;
        }
    });
}
