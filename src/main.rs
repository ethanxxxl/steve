use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Text, Section, OwnedSection, OwnedText};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::rc::Rc;
use std::borrow::Cow;

use pixels::{PixelsBuilder, SurfaceTexture};

mod editor;
use editor::EditorState;

// when do we do things?
// during the event loop.
// what does the event loop need?
// static ownership of everything that it captures.
// solutions?
//  - everything to do with anything gets moved inside of the event loop
//  - somehow segregate anything to do with the event handler from the editor logic.
//    + separate threads?
//
// what exactly do you want to do?
// you want to store a plain reference to a Chain, and another reference to to sub chains in that chain.
//  - these chains will need to be mutable.
//  - because of this, you probably shouldn't actually store these as references.
//  - how do you know what the current active subchain is then?
//  - you can't use a reference if the struct owns the chain (the reason this is a problem in the first place)
//  - you can't clone the subchain, because of the Boxed values
//
//  - what should EditorState functions be able to do?
//  - modify Chains?
//  - modify the buffer?
//  - modify anything?
//  - Editor Struct, which holds editor variables, keymaps, etc.
//  - EditorState holds references to the keymaps held in the Editor struct.
//  - ^ this doesn't work, would have to pass the lifetime up, and since the keympas are mut, the references would be garbage
//
//  - when traversing down a keymap, you pop each subchain, store the parent in a vector. when you reach the end of a chain, you just go back through
//    the vector, and add everything back where it should go.
//

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let PhysicalSize { width, height } = window.inner_size();

    let window = winit::window::WindowBuilder::new()
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    // Initialize GPU
    let (device, queue) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Request adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Request device")
    });

    // Create staging belt and a local pool
    let mut staging_belt = wgpu::util::StagingBelt::new(1024);
    let mut local_pool = futures::executor::LocalPool::new();
    let local_spawner = local_pool.spawner();

    // Prepare swap chain
    let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let mut size = window.inner_size();

    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: render_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        },
    );

    let surface_texture = SurfaceTexture::new(width, height, &window);
    let mut pixels = PixelsBuilder::new(width, height, surface_texture)
        .texture_format(wgpu::TextureFormat::Bgra8UnormSrgb)
        .build()
        .expect("Error Creating Context");

    let context = pixels.context();

    let bookerly = ab_glyph::FontArc::try_from_slice(include_bytes!(
        "/System/Library/Fonts/Menlo.ttc"
    ))
    .expect("Error Loading Font");

    let fira_code = ab_glyph::FontArc::try_from_slice(include_bytes!(
        "/System/Library/Fonts/Helvetica.ttc"
    ))
    .expect("Error Loading Font");

    let mut glyph_brush = GlyphBrushBuilder::using_fonts(vec![bookerly, fira_code])
        .build(&context.device, context.texture_format);


    let mut editor_state = EditorState::new();

    event_loop.run(move |event, _, control_flow| { match event {
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
            fn pixel_buffer_thing(p: &mut [u8]) {
                p.copy_from_slice(&[0, 0, 0, 255]);
            }
            pixels.get_frame().chunks_mut(4).for_each(pixel_buffer_thing);

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

            let buffer_section = OwnedSection {
                screen_position: (0.0, 0.0),
                bounds: (width as f32, height as f32),
                text: editor_state.get_section_text(&display_buffer),
                ..Default::default()
            };
            println!("{:?}", buffer_section.text);

            glyph_brush.queue(buffer_section.to_borrowed());

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
                    Result::Ok(())
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
    }});
}
