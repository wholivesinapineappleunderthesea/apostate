use glfw::{fail_on_errors};

pub struct GfxRenderer<'a> 
{
    glfw_backend: glfw::Glfw,
    glfw_window: glfw::PWindow,
    glfw_receiver: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    gpu_instance: wgpu::Instance,
    gpu_device: wgpu::Device,
    gpu_queue: wgpu::Queue,
    window_surface: wgpu::Surface<'a>,
    window_surface_config: wgpu::SurfaceConfiguration,
    window_surface_size: (i32, i32),
}

impl<'a> GfxRenderer<'a> 
{
    pub async fn new() -> Self
    {
        let mut glfw = glfw::init(fail_on_errors!()).unwrap();
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        let (mut glfw_window, events) = glfw.create_window(800, 600, "Hello, World!", glfw::WindowMode::Windowed).unwrap();
        glfw_window.set_key_polling(true);
        glfw_window.set_framebuffer_size_polling(true);
        glfw_window.set_mouse_button_polling(true);
        glfw_window.set_cursor_pos_polling(true);
        glfw_window.set_scroll_polling(true);
        glfw_window.set_pos_polling(true);

        let size = glfw_window.get_framebuffer_size();

        let instance_desc = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), 
            ..Default::default()
        };

        let instance = wgpu::Instance::new(&instance_desc);

        let surface = instance.create_surface(glfw_window.render_context()).unwrap();

        let adapter_desc = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            ..Default::default()
        };

        let adapter = instance.request_adapter(&adapter_desc).await.unwrap();

        let device_desc = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
			required_limits: wgpu::Limits::default(),
			memory_hints: wgpu::MemoryHints::default(),
			label: Some("WGPU Device"),
        };

        let (device, queue) = adapter.request_device(&device_desc, None).await.unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.iter().copied().filter(|format| {format.is_srgb()}).next().unwrap_or(surface_capabilities.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.0 as u32,
			height: size.1 as u32,
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: surface_capabilities.alpha_modes[0],
			view_formats: vec![],
			desired_maximum_frame_latency: 2
        };

        surface.configure(&device, &surface_config);

        Self {
            glfw_backend: glfw,
            glfw_window: glfw_window,
            glfw_receiver: events,
            gpu_instance: instance,
            gpu_device: device,
            gpu_queue: queue,
            window_surface: surface,
            window_surface_config: surface_config,
            window_surface_size: size,
            
        }
    }

    pub fn poll(&mut self) -> bool
    {
        self.glfw_backend.poll_events();

        let events: Vec<_> = glfw::flush_messages(&self.glfw_receiver).collect();
        for (_, event) in events {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    self.glfw_window.set_should_close(true)
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    self.update_surface();
                    self.resize((width, height));
                }
                _ => {}
            }
        }
        self.glfw_window.should_close() == false
    }

    pub fn frame(&mut self) 
    {
        match self.render()
        {
            Ok(_) => {},
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.update_surface();
				self.resize(self.glfw_window.get_framebuffer_size());
            }
            Err(e) => {
                eprintln!("Error rendering frame: {:?}", e);
            }
        }
    }

    pub fn update_surface(&mut self)
    {
        self.window_surface = self.gpu_instance.create_surface(self.glfw_window.render_context()).unwrap();
    }

    pub fn resize(&mut self, size: (i32, i32))
    {
        if size.0 > 0 && size.1 > 0 {
            self.window_surface_size = size;
            self.window_surface_config.width = size.0 as u32;
            self.window_surface_config.height = size.1 as u32;
            self.window_surface.configure(&self.gpu_device, &self.window_surface_config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let drawable = self.window_surface.get_current_texture()?;
		let image_view_descriptor = wgpu::TextureViewDescriptor::default();
		let image_view = drawable.texture.create_view(&image_view_descriptor);
		let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder")
		};

		let mut command_encoder = self.gpu_device.create_command_encoder(&command_encoder_descriptor);

		let colour_attachment = wgpu::RenderPassColorAttachment {
			view: &image_view,
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Clear(wgpu::Color::RED),
				store: wgpu::StoreOp::Store,
			}
		};

		let render_pass_descriptor = wgpu::RenderPassDescriptor {
			label: Some("Render Pass"),
			color_attachments: &[Some(colour_attachment)],
			depth_stencil_attachment: None,
			occlusion_query_set: None,
			timestamp_writes: None
		};

        {
            let mut pass = command_encoder.begin_render_pass(&render_pass_descriptor);
        }

		

		self.gpu_queue.submit(std::iter::once(command_encoder.finish()));
		drawable.present();

		Ok(())
	}
}