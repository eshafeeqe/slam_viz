mod app;
mod data;
mod renderer;
mod state;
mod ui;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{WindowAttributes, WindowId};

use app::App;

enum AppState {
    Uninitialized,
    Running(App),
}

struct AppHandler {
    state: AppState,
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if matches!(self.state, AppState::Uninitialized) {
            let window = Arc::new(
                event_loop
                    .create_window(
                        WindowAttributes::default()
                            .with_title("SLAM Visualizer")
                            .with_inner_size(winit::dpi::LogicalSize::new(1400, 900)),
                    )
                    .expect("Failed to create window"),
            );

            let app = pollster::block_on(App::new(window));
            self.state = AppState::Running(app);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let AppState::Running(app) = &mut self.state {
            app.window_event(&event, event_loop);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let AppState::Running(app) = &mut self.state {
            app.about_to_wait();
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting SLAM Visualizer");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut handler = AppHandler { state: AppState::Uninitialized };
    event_loop.run_app(&mut handler).expect("Event loop error");
}
