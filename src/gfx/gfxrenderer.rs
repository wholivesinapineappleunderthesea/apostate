use glfw::{fail_on_errors};
use std::env::current_dir;
use std::fmt::format;
use std::fs;
use glm::*;
use wgpu::util::DeviceExt;

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
    is_minimized: bool,

    temp_tribuffer: wgpu::Buffer,
    temp_pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
pub struct VtxLayoutPos3Col3
{
    pos: Vec3,
    color: Vec3,
}

impl VtxLayoutPos3Col3
{
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static>
    {
        const attribs: [wgpu::VertexAttribute;2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
        wgpu::VertexBufferLayout
        {
            array_stride: std::mem::size_of::<VtxLayoutPos3Col3>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attribs,
        }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}

fn make_tri(device: &wgpu::Device) -> wgpu::Buffer
{
    let vertices: [VtxLayoutPos3Col3; 3] = [
        VtxLayoutPos3Col3 {pos: Vec3::new(-0.75, -0.75, 0.0), color: Vec3::new(1.0, 0.0, 0.0)},
        VtxLayoutPos3Col3 {pos: Vec3::new( 0.75, -0.75, 0.0), color: Vec3::new(0.0, 1.0, 0.0)},
        VtxLayoutPos3Col3 {pos: Vec3::new(  0.0,  0.75, 0.0), color: Vec3::new(0.0, 0.0, 1.0)}
    ];
    let bytes: &[u8] = unsafe { any_as_u8_slice(&vertices) };

    let buffer_descriptor = wgpu::util::BufferInitDescriptor { 
        label: Some("Triangle vertex buffer"), 
        contents: bytes,
         usage: wgpu::BufferUsages::VERTEX };

    let vertex_buffer = device.create_buffer_init(&buffer_descriptor);

    return vertex_buffer;
}

fn create_pipeline(device: &wgpu::Device, shader_filename: &str, vtxbuf_layouts: Vec<wgpu::VertexBufferLayout<'static>>, pixel_format: wgpu::TextureFormat) -> wgpu::RenderPipeline
    {
        let mut file_path = current_dir().unwrap();
        file_path.push("src/gfx/shdrs/");
        file_path.push(shader_filename);
        let filepath = file_path.into_os_string().into_string().unwrap();

        let src = fs::read_to_string(filepath).expect("Unable to read file");

        let shader_module_label = format!("{} SHDRMOD", shader_filename);
        let shader_module_desc = wgpu::ShaderModuleDescriptor
        {
            label: Some(shader_module_label.as_str()),
            source: wgpu::ShaderSource::Wgsl(src.into()),
        };

        let shader_module = device.create_shader_module(shader_module_desc);

        let pipeline_layout_label = format!("{} PIPELINE_LAYOUT", shader_filename);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: Some(pipeline_layout_label.as_str()),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_targets = [Some(wgpu::ColorTargetState
        {
            format: pixel_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let render_pipeline_label = format!("{} PIPELINE", shader_filename);
        let render_pipeline_desc = wgpu::RenderPipelineDescriptor
        {
            label: Some(render_pipeline_label.as_str()),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState
            {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &vtxbuf_layouts,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState
            {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &render_targets,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState
            {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: None,    
            multisample: wgpu::MultisampleState
            {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        };

        device.create_render_pipeline(&render_pipeline_desc)
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

        let tri_buffer = make_tri(&device);
        let tri_pipeline = create_pipeline(&device, "test.wgsl", vec![VtxLayoutPos3Col3::get_layout()], surface_config.format);

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
            is_minimized: false,

            temp_tribuffer: tri_buffer,
            temp_pipeline: tri_pipeline,
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
            self.is_minimized = false;
            self.window_surface_size = size;
            self.window_surface_config.width = size.0 as u32;
            self.window_surface_config.height = size.1 as u32;
            self.window_surface.configure(&self.gpu_device, &self.window_surface_config);
        }
        else {
            self.is_minimized = true;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.is_minimized {
            return Ok(());
        }
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
            pass.set_pipeline(&self.temp_pipeline);
            pass.set_vertex_buffer(0, self.temp_tribuffer.slice(..));
            pass.draw(0..3, 0..1);
        }

		

		self.gpu_queue.submit(std::iter::once(command_encoder.finish()));
		drawable.present();

		Ok(())
	}

    
}