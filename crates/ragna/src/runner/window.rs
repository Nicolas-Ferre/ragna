use crate::runner::common::{Runner, TargetSpecialized};
use crate::App;
use wgpu::Color;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

// coverage: off (window cannot be tested)

#[derive(Debug)]
pub(crate) struct WindowRunner {
    app: App,
    background_color: Color,
    runner: Option<Runner>,
}

impl ApplicationHandler for WindowRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.refresh_surface(event_loop);
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => self.update(),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.update_window_size(size),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(runner) = &mut self.runner {
            match &runner.target.inner {
                TargetSpecialized::Window(target) => target.window.request_redraw(),
                TargetSpecialized::Texture(_) => {
                    unreachable!("internal error: uninitialized window")
                }
            }
        }
    }
}

impl WindowRunner {
    pub(crate) fn new(app: App, background_color: Color) -> Self {
        Self {
            app,
            background_color,
            runner: None,
        }
    }

    fn refresh_surface(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(runner) = &mut self.runner {
            runner.refresh_surface();
        } else {
            self.runner = Some(Runner::new_window(
                &self.app,
                event_loop,
                self.background_color,
            ));
        }
    }

    fn update(&mut self) {
        if let Some(runner) = &mut self.runner {
            runner.run_step();
        }
    }

    fn update_window_size(&mut self, size: PhysicalSize<u32>) {
        if let Some(runner) = &mut self.runner {
            runner.update_surface_size(size);
        }
    }
}
