use crate::app::App;
use crate::renderer::Renderer;

use log::warn;

mod app;
mod camera;
mod module;
mod physics;
mod player;
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

    let mut fps_frame_count: u16 = 0;
    let mut fps_frame_time: f32 = 0.0;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        app.input.update(&event);
        match event {
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
            winit::event::Event::MainEventsCleared => {
                let delta_time = frame_time.elapsed().as_secs_f32();
                frame_time = std::time::Instant::now();

                app.update(delta_time);
                app.render();

                fps_frame_count += 1;
                fps_frame_time += delta_time;

                if fps_frame_time >= 1.0 {
                    warn!("FPS: {fps_frame_count}");
                    fps_frame_count = 0;
                    fps_frame_time = 0.0;
                }
            }
            _ => (),
        }
    });
}
