use std::sync::{Arc, Mutex};

use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Device, Queue, RenderPipeline};

use crate::renderer::{manager::Manager, models::ModelBuilder, pipeline::Render};

/// Represents a solid color to render
#[derive(Debug)]
pub struct ColorModel {
    color_buffer: wgpu::Buffer,
    render_pipeline: Arc<RenderPipeline>,
    bind_group: Arc<BindGroup>,
}

impl ColorModel {
    pub fn new(
        color_buffer: wgpu::Buffer,
        render_pipeline: Arc<RenderPipeline>,
        bind_group: Arc<BindGroup>,
    ) -> Self {
        Self {
            color_buffer,
            render_pipeline,
            bind_group,
        }
    }
}

impl Render for ColorModel {
    fn pipeline(&self) -> Arc<RenderPipeline> {
        self.render_pipeline.clone()
    }

    fn bindgroup(&self) -> Arc<BindGroup> {
        self.bind_group.clone()
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builds a model for rendering a solid color background
pub struct ColorModelBuilder {
    color: [f32; 4],
    label: String,
}

impl ColorModelBuilder {
    /// Create a new builder with the specified RGBA color
    pub fn new(color: [f32; 4], label: impl Into<String>) -> Self {
        Self {
            color,
            label: label.into(),
        }
    }

    /// Parse a hex color string (#RRGGBB) and create a builder
    pub fn from_hex_color(hex_color: &str, label: impl Into<String>) -> Self {
        let rgba = parse_hex_color(hex_color);
        Self::new(rgba, label)
    }
}

impl ModelBuilder for ColorModelBuilder {
    type Target = ColorModel;

    fn build(
        &self,
        device: &Device,
        queue: &Queue,
        bindgroup_layout_manager: Arc<Mutex<Manager<BindGroupLayout>>>,
        pipeline_manager: Arc<Mutex<Manager<RenderPipeline>>>,
    ) -> Self::Target {
        // Create a buffer for the color uniform
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Color Buffer: {}", self.label)),
            contents: bytemuck::cast_slice(&[ColorUniform { color: self.color }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Get or create the bind group layout
        let bind_group_layout =
            bindgroup_layout_manager
                .lock()
                .unwrap()
                .get_or_init("color_bind_group_layout", || {
                    Arc::new(
                        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            entries: &[wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            }],
                            label: Some("color_bind_group_layout"),
                        }),
                    )
                });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Color Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipeline if it doesn't exist yet
        let pipeline =
            pipeline_manager
                .lock()
                .unwrap()
                .get_or_init("color_render_pipeline", || {
                    let shader = device.create_shader_module(crate::shaders::COLOR_SHADER);

                    Arc::new(
                        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some("Color Render Pipeline"),
                            layout: Some(&pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &shader,
                                entry_point: Some("vs_main"),
                                buffers: &[],
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &shader,
                                entry_point: Some("fs_main"),
                                targets: &[Some(wgpu::ColorTargetState {
                                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                    write_mask: wgpu::ColorWrites::ALL,
                                })],
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            }),
                            primitive: wgpu::PrimitiveState {
                                topology: wgpu::PrimitiveTopology::TriangleList,
                                strip_index_format: None,
                                front_face: wgpu::FrontFace::Ccw,
                                cull_mode: None,
                                polygon_mode: wgpu::PolygonMode::Fill,
                                unclipped_depth: false,
                                conservative: false,
                            },
                            depth_stencil: None,
                            multisample: wgpu::MultisampleState {
                                count: 1,
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                            multiview: None,
                            cache: None,
                        }),
                    )
                });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
            label: Some(&format!("color_bind_group_{}", self.label)),
        });

        ColorModel::new(color_buffer, pipeline.clone(), Arc::new(bind_group))
    }
}

// Uniform structure matching the shader's expected format
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ColorUniform {
    color: [f32; 4],
}

/// Parse a hex color string to RGBA [f32; 4] values
/// Supports #RRGGBB format
fn parse_hex_color(hex: &str) -> [f32; 4] {
    // Default to opaque black
    let mut rgba = [0.0, 0.0, 0.0, 1.0];

    // Check if the string starts with '#' and has the right length
    if hex.starts_with('#') && hex.len() == 7 {
        let hex = &hex[1..]; // Remove the # prefix

        // Try to parse the hex values
        if let (Some(r), Some(g), Some(b)) = (
            u8::from_str_radix(&hex[0..2], 16).ok(),
            u8::from_str_radix(&hex[2..4], 16).ok(),
            u8::from_str_radix(&hex[4..6], 16).ok(),
        ) {
            // Convert to normalized float values (0.0 - 1.0)
            rgba[0] = r as f32 / 255.0;
            rgba[1] = g as f32 / 255.0;
            rgba[2] = b as f32 / 255.0;
        }
    }

    rgba
}

