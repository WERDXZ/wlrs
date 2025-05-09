use std::sync::{Arc, Mutex};

use image::DynamicImage;
use wgpu::{BindGroup, BindGroupLayout, Device, Queue, RenderPipeline};

use crate::{
    asset::image::ImageTexture,
    renderer::{manager::Manager, models::ModelBuilder, pipeline::Render},
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct TextureModel {
    texture: ImageTexture,
    render_pipeline: Arc<RenderPipeline>,
    bind_group: Arc<BindGroup>,
}

impl TextureModel {
    pub fn new(
        texture: ImageTexture,
        render_pipeline: Arc<RenderPipeline>,
        bind_group: Arc<BindGroup>,
    ) -> Self {
        Self {
            texture,
            render_pipeline,
            bind_group,
        }
    }
}

impl Render for TextureModel {
    fn pipeline(&self) -> Arc<RenderPipeline> {
        self.render_pipeline.clone()
    }

    fn bindgroup(&self) -> Arc<BindGroup> {
        self.bind_group.clone()
    }
}

pub struct TextureModelBuilder {
    image: DynamicImage,
    label: String,
}

impl TextureModelBuilder {
    pub fn new(image: DynamicImage, label: impl Into<String>) -> Self {
        Self {
            image,
            label: label.into(),
        }
    }
}

impl ModelBuilder for TextureModelBuilder {
    type Target = TextureModel;

    fn build(
        &self,
        device: &Device,
        queue: &Queue,
        bindgroup_layout_manager: Arc<Mutex<Manager<BindGroupLayout>>>,
        pipeline_manager: Arc<Mutex<Manager<RenderPipeline>>>,
    ) -> Self::Target {
        // Create texture from image using the provided queue
        let texture = ImageTexture::from_image(device, queue, &self.image, &self.label);

        // Get or create the bind group layout and pipeline
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Texture Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipeline if it doesn't exist yet
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
                                    format: wgpu::TextureFormat::Bgra8UnormSrgb, // Use your preferred format
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
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some(&format!("texture_bind_group_{}", self.label)),
        });

        TextureModel::new(texture, pipeline.clone(), Arc::new(bind_group))
    }
}
