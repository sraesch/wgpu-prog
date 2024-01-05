use log::{info, debug, error};
use winit::{event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, dpi::{LogicalSize, LogicalPosition}, event::{Event, WindowEvent, ElementState}};

use crate::{error::{Error, Result}, event_handler::EventHandler};

/// The options for creating the canvas.
pub struct CanvasOptions {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

pub fn create_and_run_canvas<H>(options: CanvasOptions, mut handler: H) -> Result<()>
where
    H: EventHandler
{
    info!("Creating canvas...");

    // create event loop with control flow set to Poll, i.e., the event loop will run as fast as
    // possible
    debug!("Create event loop...");
    let event_loop = EventLoop::new().map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    debug!("Create window...");
    let window = WindowBuilder::new()
            .with_title(options.title)
            .with_inner_size(LogicalSize::new(options.width, options.height))
            .build(&event_loop).map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;

    let mut is_initialized = false;
    let mut cursor_pos = [0.0, 0.0];

    event_loop
        .run(move |event, window_target| {
            let scale_factor = window.scale_factor();

            match event {
                Event::Resumed => {
                    // The context needs to be current for the Renderer to set up shaders and
                    // buffers. It also performs function loading, which needs a current context on
                    // WGL.
                    if !is_initialized {
                        if let Err(err) = handler.setup(options.width, options.height) {
                            error!("Error during setup: {}", err);
                        }

                        is_initialized = true;
                    }
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        if size.width != 0 && size.height != 0 {
                                handler.resize(size.width, size.height);
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let logical_position =
                            LogicalPosition::from_physical(position, scale_factor);

                        cursor_pos = [logical_position.x, logical_position.y];
                        handler.cursor_move(logical_position.x, logical_position.y);
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let x = cursor_pos[0];
                        let y = cursor_pos[1];

                        let pressed: bool = state == ElementState::Pressed;

                        handler.mouse_button(x, y, button, pressed);
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        let pressed = event.state == ElementState::Pressed;
                        handler.keyboard_event(event.logical_key, pressed);
                    }
                    WindowEvent::CloseRequested => window_target.exit(),
                    _ => (),
                },
                _ => (),
            }
        })
        .map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;

    Ok(())
}