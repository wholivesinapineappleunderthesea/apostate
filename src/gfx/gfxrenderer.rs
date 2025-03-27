use glfw::fail_on_errors;
use glm::*;
use wgpu::core::command;
use std::env::current_dir;
use std::fs;
use wgpu::include_wgsl;
use wgpu::util::DeviceExt;

pub struct GfxRenderer<'a> {
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

    uniform_buffer: wgpu::Buffer,
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    bind_groups: Vec<wgpu::BindGroup>,

    temp_quadbuffer: wgpu::Buffer,
    temp_quadidxbuffer: wgpu::Buffer,
    temp_pipeline: wgpu::RenderPipeline,

    cumulative_time: f32,
}

#[repr(C)]
pub struct VtxLayoutPos3Col3 {
    pos: Vec3,
    color: Vec3,
}

impl VtxLayoutPos3Col3 {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VtxLayoutPos3Col3>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

fn make_quad(device: &wgpu::Device) -> wgpu::Buffer {
    let vertices: [VtxLayoutPos3Col3; 4] = [
        // top-left
        VtxLayoutPos3Col3 {
            pos: vec3(-0.5, 0.5, 0.0),
            color: vec3(1.0, 0.0, 0.0),
        },
        // top-right
        VtxLayoutPos3Col3 {
            pos: vec3(0.5, 0.5, 0.0),
            color: vec3(0.0, 1.0, 0.0),
        },
        // bottom-right
        VtxLayoutPos3Col3 {
            pos: vec3(0.5, -0.5, 0.0),
            color: vec3(0.0, 0.0, 1.0),
        },
        // bottom-left
        VtxLayoutPos3Col3 {
            pos: vec3(-0.5, -0.5, 0.0),
            color: vec3(1.0, 1.0, 1.0),
        },
    ];
    let bytes: &[u8] = unsafe { any_as_u8_slice(&vertices) };

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad vertex buffer"),
        contents: bytes,
        usage: wgpu::BufferUsages::VERTEX,
    };

    device.create_buffer_init(&buffer_descriptor)
}

fn make_quad_indexbuf(device: &wgpu::Device) -> wgpu::Buffer
{
    let indices: [u16; 6] = [0, 2, 1, 0, 3, 2];
    let bytes: &[u8] = unsafe { any_as_u8_slice(&indices) };

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad index buffer"),
        contents: bytes,
        usage: wgpu::BufferUsages::INDEX,
    };

    device.create_buffer_init(&buffer_descriptor)
}

fn create_pipeline(
    device: &wgpu::Device,
    shader_module_desc: wgpu::ShaderModuleDescriptor,
    name: &str,
    vtxbuf_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    pixel_format: wgpu::TextureFormat,
    bind_group_layouts: Vec<&wgpu::BindGroupLayout>,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(shader_module_desc);

    let pipeline_layout_label = format!("{} PIPELINE_LAYOUT", name);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(pipeline_layout_label.as_str()),
        bind_group_layouts: &bind_group_layouts,
        push_constant_ranges: &[],
    });

    let render_targets = [Some(wgpu::ColorTargetState {
        format: pixel_format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    })];


    let render_pipeline_label = format!("{} PIPELINE", name);
    let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
        label: Some(render_pipeline_label.as_str()),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            buffers: &vtxbuf_layouts,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            targets: &render_targets,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
            unclipped_depth: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    };

    device.create_render_pipeline(&render_pipeline_desc)
}

impl<'a> GfxRenderer<'a> {
    pub async fn new() -> Self {
        let mut glfw = glfw::init(fail_on_errors!()).unwrap();
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        let (mut glfw_window, events) = glfw
            .create_window(800, 600, "Hello, World!", glfw::WindowMode::Windowed)
            .unwrap();
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

        let surface = instance
            .create_surface(glfw_window.render_context())
            .unwrap();

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
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|format| format.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0 as u32,
            height: size.1 as u32,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let bind_group_layouts_desc = wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        };

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<f32>() as u64 * 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layouts = vec![
            device.create_bind_group_layout(&bind_group_layouts_desc)
        ];

        let bind_groups = vec![
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: &bind_group_layouts[0],
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &uniform_buffer,
                            offset: 0,
                            size: None,
                        }),
                    }
                ],
            })
        ];

        let quad_buffer = make_quad(&device);
        let quad_indexbuf = make_quad_indexbuf(&device);
        let tri_pipeline_src = include_wgsl!("shdrs/test.wgsl");
        let tri_pipeline = create_pipeline(
            &device,
            tri_pipeline_src,
            "test.wgsl",
            vec![VtxLayoutPos3Col3::get_layout()],
            surface_config.format,
            bind_group_layouts.iter().collect(),
        );

        

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

            uniform_buffer: uniform_buffer,
            bind_group_layouts: bind_group_layouts,
            bind_groups: bind_groups,

            temp_quadbuffer: quad_buffer,
            temp_quadidxbuffer: quad_indexbuf,
            temp_pipeline: tri_pipeline,

            cumulative_time: 0.0,
        }
    }

    pub fn poll(&mut self) -> bool {
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

    pub fn frame(&mut self) {
        match self.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.update_surface();
                self.resize(self.glfw_window.get_framebuffer_size());
            }
            Err(e) => {
                eprintln!("Error rendering frame: {:?}", e);
            }
        }
    }

    pub fn update_surface(&mut self) {
        self.window_surface = self
            .gpu_instance
            .create_surface(self.glfw_window.render_context())
            .unwrap();
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        if size.0 > 0 && size.1 > 0 {
            self.is_minimized = false;
            self.window_surface_size = size;
            self.window_surface_config.width = size.0 as u32;
            self.window_surface_config.height = size.1 as u32;
            self.window_surface
                .configure(&self.gpu_device, &self.window_surface_config);
        } else {
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
            label: Some("Render Encoder"),
        };

        let time = self.cumulative_time;
        self.cumulative_time += 0.01;
        let time_bytes = unsafe { std::slice::from_raw_parts(&time as *const f32 as *const u8, 4 * 4) };

        self.gpu_queue.write_buffer(
            &self.uniform_buffer,
            0,
            time_bytes,
        );

        let mut command_encoder = self
            .gpu_device
            .create_command_encoder(&command_encoder_descriptor);

        let colour_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                store: wgpu::StoreOp::Store,
            },
        };

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(colour_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        
        


        {
            let mut pass = command_encoder.begin_render_pass(&render_pass_descriptor);
            pass.set_pipeline(&self.temp_pipeline);

            pass.set_bind_group(0, &self.bind_groups[0], &[]);
            


            pass.set_vertex_buffer(0, self.temp_quadbuffer.slice(..));
            pass.set_index_buffer(self.temp_quadidxbuffer.slice(..), wgpu::IndexFormat::Uint16);        
            pass.draw_indexed(0..6, 0, 0..1);
        }

        self.gpu_queue
            .submit(std::iter::once(command_encoder.finish()));
        drawable.present();

        Ok(())
    }
}
