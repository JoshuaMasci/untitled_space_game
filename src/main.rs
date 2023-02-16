use crate::app::App;
use crate::renderer::{Renderer, Vertex};

mod app;
mod entity;
mod module;
mod perspective_camera;
mod physics;
mod renderer;
mod space_craft;
mod transform;
mod world;

fn main() {
    pretty_env_logger::init_timed();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("untitled_space_game")
        .with_resizable(true)
        .build(&event_loop)
        .unwrap();

    window.set_maximized(true);

    let mut app = App::new(&window);

    let mut frame_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent {
            event: winit::event::WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => control_flow.set_exit(),
        winit::event::Event::WindowEvent {
            event: winit::event::WindowEvent::Resized(new_size),
            window_id,
        } if window_id == window.id() => {
            app.resize(new_size);
        }
        winit::event::Event::WindowEvent {
            event: winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. },
            window_id,
        } if window_id == window.id() => {
            app.resize(*new_inner_size);
        }
        winit::event::Event::WindowEvent {
            event: winit::event::WindowEvent::KeyboardInput { input, .. },
            window_id,
        } if window_id == window.id() => {
            app.keyboard_event(&input.scancode, &input.state);
        }
        winit::event::Event::MainEventsCleared => {
            app.update(frame_time.elapsed().as_secs_f32());
            app.render();
            frame_time = std::time::Instant::now();
        }
        _ => (),
    });
}
