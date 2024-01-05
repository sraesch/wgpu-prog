use std::fmt::format;

use log::{debug, error, info};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    canvas,
    error::{Error, Result},
    event_handler::EventHandler,
};

/// The options for creating the canvas.
pub struct CanvasOptions {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

/// The data for the canvas.
struct CanvasData<H: EventHandler> {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    handler: H,
}

impl<H: EventHandler> CanvasData<H> {
    /// Create a new canvas data object for the given window.
    /// This will also create the respective wgpu objects.
    ///
    /// # Arguments
    /// * `window` - The window to create the canvas for.
    /// * `handler` - The event handler for the canvas.
    async fn new(window: Window, handler: H) -> Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window, so this should be safe.
        debug!("Create surface...");
        let surface = unsafe { instance.create_surface(&window) }
            .map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;

        debug!("Choose adapter...");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| Error::GraphicsAPI("No suitable adapter found".to_string()))?;

        // print some infos about the adapter
        {
            let adapter_info = adapter.get_info();
            info!("Adapter name: {}", adapter_info.name);
            info!("Adapter vendor: {}", adapter_info.vendor);
            info!("Adapter driver: {}", adapter_info.driver);
            info!("Adapter driver info: {}", adapter_info.driver_info);
            info!("Adapter backend API: {}", adapter_info.backend.to_str());
        }

        // create the device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this program assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            handler,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn handler(&mut self) -> &mut H {
        &mut self.handler
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.handler.resize(new_size.width, new_size.height);
        }
    }

    /// Returns true if the event should be captured.
    ///
    /// # Arguments
    /// * `event` - The event to check.
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<()> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.handler.next_frame();

        Ok(())
    }
}

pub async fn create_and_run_canvas<H>(options: CanvasOptions, mut handler: H) -> Result<()>
where
    H: EventHandler,
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
        .build(&event_loop)
        .map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;

    let mut canvas_data = CanvasData::new(window, handler).await?;
    if let Err(err) = canvas_data.handler().setup(options.width, options.height) {
        error!("Error during setup: {}", err);
        return Err(Error::Internal(format!("Error during setup: {}", err)));
    }

    let mut cursor_pos = [0.0, 0.0];

    event_loop
        .run(move |event, window_target| {
            let scale_factor = canvas_data.window().scale_factor();

            match event {
                Event::WindowEvent { event, window_id }
                    if window_id == canvas_data.window().id() =>
                {
                    if !canvas_data.input(&event) {
                        match event {
                            WindowEvent::Resized(size) => {
                                canvas_data.resize(size);
                            }
                            WindowEvent::CursorMoved { position, .. } => {
                                let logical_position =
                                    LogicalPosition::from_physical(position, scale_factor);

                                cursor_pos = [logical_position.x, logical_position.y];
                                canvas_data
                                    .handler()
                                    .cursor_move(logical_position.x, logical_position.y);
                            }
                            WindowEvent::MouseInput { state, button, .. } => {
                                let x = cursor_pos[0];
                                let y = cursor_pos[1];

                                let pressed: bool = state == ElementState::Pressed;

                                canvas_data.handler().mouse_button(x, y, button, pressed);
                            }
                            WindowEvent::KeyboardInput { event, .. } => {
                                let pressed = event.state == ElementState::Pressed;
                                canvas_data
                                    .handler()
                                    .keyboard_event(event.logical_key, pressed);
                            }
                            WindowEvent::CloseRequested => window_target.exit(),
                            WindowEvent::RedrawRequested => {
                                canvas_data.update();
                                match canvas_data.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if lost
                                    Err(Error::ContextLost(_)) => {
                                        canvas_data.resize(canvas_data.size)
                                    }
                                    // The system is out of memory, we should probably quit
                                    Err(Error::OutOfMemory(_)) => {
                                        error!("Out of memory");
                                        window_target.exit();
                                    }
                                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                                    Err(e) => {
                                        error!("{:?}", e)
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
                Event::AboutToWait => {
                    canvas_data.window().request_redraw();
                }
                _ => (),
            }
        })
        .map_err(|e| Error::GraphicsAPI(format!("{}", e)))?;

    Ok(())
}
