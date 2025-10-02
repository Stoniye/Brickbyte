use winit::event::{WindowEvent};
use winit::event_loop::{EventLoopBuilder, ActiveEventLoop};
use winit::window::{Window, WindowId};

struct Brickbyte{
    window: Option<Window>,
}

impl Brickbyte {
    fn new() -> Self {
        Brickbyte{window: None}
    }
}

impl winit::application::ApplicationHandler for Brickbyte {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Brickbyte");
        self.window = Some(event_loop.create_window(window_attributes).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if let WindowEvent::CloseRequested = event {
            event_loop.exit();
        }
    }
}

fn main() {
    let event_loop = EventLoopBuilder::default().build().unwrap();
    let mut brickbyte = Brickbyte::new();
    event_loop.run_app(&mut brickbyte).unwrap();
}
