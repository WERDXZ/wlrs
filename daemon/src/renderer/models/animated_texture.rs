use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use wgpu::{BindGroup, BindGroupLayout, Device, Queue, RenderPipeline};

use crate::{
    asset::animated::AnimatedTexture,
    renderer::{manager::Manager, models::ModelBuilder, pipeline::Render},
};

/// A model that renders an animated texture
#[derive(Debug)]
pub struct AnimatedTextureModel {
    /// The animated texture to render
    texture: AnimatedTexture,
    /// The render pipeline
    render_pipeline: Arc<RenderPipeline>,
    /// The bind group for the model
    bind_group: Arc<BindGroup>,
    /// Layout for the bind group
    bind_group_layout: Arc<BindGroupLayout>,
}

impl AnimatedTextureModel {
    pub fn new(
        texture: AnimatedTexture,
        render_pipeline: Arc<RenderPipeline>,
        bind_group: Arc<BindGroup>,
        bind_group_layout: Arc<BindGroupLayout>,
    ) -> Self {
        Self {
            texture,
            render_pipeline,
            bind_group,
            bind_group_layout,
        }
    }
}

impl Render for AnimatedTextureModel {
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

    fn pre_render(&mut self, device: &Device, dt: Duration) {
        // Print the dt value for debugging
        println!("Animation pre_render dt: {dt:?}");

        // Update the animated texture and track if the frame changed
        let frame_changed = self.texture.update(dt);

        // Debug print frame change status
        println!("Frame changed: {frame_changed}");

        // Flag for bind group update if frame changed
        if !frame_changed {
            return;
        }
        // If the frame has changed, we need to update the bind group
        // Create a new bind group with the current frame
        self.bind_group = Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(self.texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(self.texture.sampler()),
                },
            ],
            label: Some("animated_texture_bind_group"),
        }));
    }
}

/// Builder for animated texture models
pub struct AnimatedTextureModelBuilder {
    path: Box<Path>,
    label: String,
    looping: bool,
}

impl AnimatedTextureModelBuilder {
    pub fn new(path: impl AsRef<Path>, label: impl Into<String>) -> Self {
        Self {
            path: path.as_ref().into(),
            label: label.into(),
            looping: true,
        }
    }

    /// Set whether the animation should loop
    pub fn looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
}

impl ModelBuilder for AnimatedTextureModelBuilder {
    type Target = AnimatedTextureModel;

    fn build(
        &self,
        device: &Device,
        queue: &Queue,
        bindgroup_layout_manager: Arc<Mutex<Manager<BindGroupLayout>>>,
        pipeline_manager: Arc<Mutex<Manager<RenderPipeline>>>,
    ) -> Self::Target {
        // Load the animated texture
        let texture =
            AnimatedTexture::from_path(device, queue, &self.path, &self.label, self.looping)
                .expect("Failed to load animated texture");

        // Get or create the bind group layout
        let bind_group_layout = bindgroup_layout_manager.lock().unwrap().get_or_init(
            "texture_bind_group_layout",
            || {
                Arc::new(
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                        ],
                        label: Some("texture_bind_group_layout"),
                    }),
                )
            },
        );

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Animated Texture Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Get or create the pipeline
        let pipeline =
            pipeline_manager
                .lock()
                .unwrap()
                .get_or_init("texture_render_pipeline", || {
                    let shader = device.create_shader_module(crate::shaders::TEXTURE_SHADER);

                    Arc::new(
                        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some("Texture Render Pipeline"),
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
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(texture.sampler()),
                },
            ],
            label: Some(&format!("animated_texture_bind_group_{}", self.label)),
        });

        AnimatedTextureModel::new(
            texture,
            pipeline.clone(),
            Arc::new(bind_group),
            bind_group_layout,
        )
    }
}
