//! Rebit Desktop — 原生 BIM/CAD 应用入口点。
//!
//! 构建墙体、楼板和柱子的 BIM 实体，提取三角形网格，并通过 wgpu 在窗口中渲染。

use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use geometry_kernel::{
    Point3D, Tolerance,
    mesh::{GpuMesh, extract_mesh},
    topology::{Edge, Face, Loop, Shell, Solid, Vertex},
};

// ---------------------------------------------------------------------------
// 构建示例 BIM 场景
// ---------------------------------------------------------------------------

/// 构建一个盒状实体（用于墙体、楼板、柱子）。
fn make_box_solid(face_id_start: u64, cx: f64, cy: f64, cz: f64, w: f64, d: f64, h: f64) -> Solid {
    let hw = w * 0.5;
    let hd = d * 0.5;
    let hh = h * 0.5;
    let v = [
        Point3D::new(cx - hw, cy - hd, cz - hh),
        Point3D::new(cx + hw, cy - hd, cz - hh),
        Point3D::new(cx + hw, cy + hd, cz - hh),
        Point3D::new(cx - hw, cy + hd, cz - hh),
        Point3D::new(cx - hw, cy - hd, cz + hh),
        Point3D::new(cx + hw, cy - hd, cz + hh),
        Point3D::new(cx + hw, cy + hd, cz + hh),
        Point3D::new(cx - hw, cy + hd, cz + hh),
    ];
    let verts: Vec<Vertex> = v
        .iter()
        .enumerate()
        .map(|(i, p)| Vertex::new(i as u64, *p))
        .collect();
    fn quad(v0: &Vertex, v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Loop {
        Loop::new(
            0,
            vec![
                Edge::new(0, v0.clone(), v1.clone()),
                Edge::new(1, v1.clone(), v2.clone()),
                Edge::new(2, v2.clone(), v3.clone()),
                Edge::new(3, v3.clone(), v0.clone()),
            ],
        )
    }
    let faces = vec![
        Face::new(
            face_id_start,
            quad(&verts[0], &verts[1], &verts[2], &verts[3]),
            vec![],
        ),
        Face::new(
            face_id_start + 1,
            quad(&verts[4], &verts[5], &verts[6], &verts[7]),
            vec![],
        ),
        Face::new(
            face_id_start + 2,
            quad(&verts[0], &verts[1], &verts[5], &verts[4]),
            vec![],
        ),
        Face::new(
            face_id_start + 3,
            quad(&verts[2], &verts[3], &verts[7], &verts[6]),
            vec![],
        ),
        Face::new(
            face_id_start + 4,
            quad(&verts[0], &verts[3], &verts[7], &verts[4]),
            vec![],
        ),
        Face::new(
            face_id_start + 5,
            quad(&verts[1], &verts[2], &verts[6], &verts[5]),
            vec![],
        ),
    ];
    Solid::new(0, Shell::new(0, faces))
}

/// 将所有实体的网格合并为一个 GpuMesh。
fn merge_meshes(meshes: &[GpuMesh]) -> GpuMesh {
    let mut merged = GpuMesh::new();
    for m in meshes {
        let base = (merged.vertices.len() / 9) as u32;
        merged.vertices.extend_from_slice(&m.vertices);
        merged.indices.extend(m.indices.iter().map(|i| i + base));
    }
    merged
}

// ---------------------------------------------------------------------------
// GPU 状态
// ---------------------------------------------------------------------------

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl GpuState {
    async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();
        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        // 构建 BIM 场景。
        let tol = Tolerance::default();
        let wall = make_box_solid(0, 0.0, 2.0, 1.5, 5.0, 0.3, 3.0); // 墙体 X
        let wall2 = make_box_solid(6, 2.5, 0.0, 1.5, 0.3, 4.0, 3.0); // 墙体 Y
        let slab = make_box_solid(12, 2.5, 2.0, 3.0, 5.0, 4.0, 0.2); // 楼板
        let column = make_box_solid(18, 0.0, 0.0, 1.5, 0.4, 0.4, 3.0); // 柱子

        let mesh = merge_meshes(&[
            extract_mesh(&wall, &tol),
            extract_mesh(&wall2, &tol),
            extract_mesh(&slab, &tol),
            extract_mesh(&column, &tol),
        ]);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                struct VertexInput {
                    @location(0) pos: vec3<f32>,
                    @location(1) normal: vec3<f32>,
                    @location(2) color: vec3<f32>,
                };
                struct VertexOutput {
                    @builtin(position) pos: vec4<f32>,
                    @location(0) color: vec3<f32>,
                };
                @vertex
                fn vs_main(in: VertexInput) -> VertexOutput {
                    var out: VertexOutput;
                    // Scale world coords (±5m) to clip space ±1 for viewing.
                    out.pos = vec4<f32>(in.pos * 0.15 - vec3<f32>(0.35, 0.2, 0.3), 1.0);
                    out.color = in.color;
                    return out;
                }
                @fragment
                fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                    return vec4<f32>(in.color, 1.0);
                }
                "#
                .into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let vb_layout = wgpu::VertexBufferLayout {
            array_stride: GpuMesh::STRIDE_BYTES as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 24,
                    shader_location: 2,
                },
            ],
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vb_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("failed to acquire surface texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.15,
                            g: 0.15,
                            b: 0.18,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.render_pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..self.index_count, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}

// ---------------------------------------------------------------------------
// 应用状态
// ---------------------------------------------------------------------------

struct App {
    gpu: Option<GpuState>,
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Rebit — BIM/CAD Platform")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720)),
                )
                .unwrap(),
        );
        let gpu = pollster::block_on(GpuState::new(Arc::clone(&window)));
        self.gpu = Some(gpu);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(new_size.width, new_size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.render();
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        gpu: None,
        window: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
