use canvas::create_and_run_canvas;
use event_handler::EventHandler;
use log::{info, trace, debug};

mod canvas;
mod error;
mod event_handler;

struct Handler {

}

impl EventHandler for Handler {
    fn setup(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        info!("Setup canvas with width {} and height {}", width, height);

        Ok(())
    }

    fn stop(&mut self) {
        info!("Stop canvas");
    }

    fn next_frame(&mut self) {
        trace!("Render Frame");
    }

    fn resize(&mut self, w: u32, h: u32) {
        debug!("Resize canvas to width {} and height {}", w, h);
    }

    fn cursor_move(&mut self, x: f64, y: f64) {
        trace!("Cursor moved to x {} and y {}", x, y);
    }

    fn mouse_button(&mut self, x: f64, y: f64, button: winit::event::MouseButton, pressed: bool) {
        debug!("Mouse button {:?} at x {} and y {} was {}", button, x, y, if pressed { "pressed" } else { "released" });
    }

    fn keyboard_event(&mut self, key: winit::keyboard::Key, pressed: bool) {
        debug!("Key {:?} was {}", key, if pressed { "pressed" } else { "released" });
    }
}

fn main() {
    env_logger::init();

    let options = canvas::CanvasOptions {
        width: 800,
        height: 600,
        title: "Hello World".to_string(),
    };

    let handler = Handler {};
    create_and_run_canvas(options, handler).unwrap();
}
