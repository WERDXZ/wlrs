use std::sync::{Arc, Mutex};

use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use wgpu::{BindGroup, BindGroupLayout, Device, Queue, RenderPipeline};

use crate::{
    asset::image::ImageTexture,
    renderer::{manager::Manager, models::ModelBuilder, pipeline::Render},
};

/// Base effect model that can render image-based effects
#[derive(Debug)]
pub struct EffectModel {
    /// The mask texture used for the effect
    texture: ImageTexture,
    /// The render pipeline for this effect
    render_pipeline: Arc<RenderPipeline>,
    /// The bind group containing our texture and any effect parameters
    bind_group: Arc<BindGroup>,
}

impl EffectModel {
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

impl Render for EffectModel {
    fn pipeline(&self) -> Arc<RenderPipeline> {
        self.render_pipeline.clone()
    }

    fn bindgroup(&self) -> Arc<BindGroup> {
        self.bind_group.clone()
    }
}

/// Builder for creating static effect models
pub struct EffectModelBuilder {
    /// The mask image used for the effect
    image: DynamicImage,
    /// Optional alpha mask (if not provided, the alpha channel of the image is used)
    mask: Option<DynamicImage>,
    /// The label for this effect
    label: String,
    /// Whether to pre-multiply the RGB values by the alpha value
    premultiply_alpha: bool,

    shader: wgpu::ShaderModuleDescriptor<'static>,
}

impl EffectModelBuilder {
    /// Create a new effect builder with the given image as both color and mask
    pub fn new(
        image: DynamicImage,
        shader: wgpu::ShaderModuleDescriptor<'static>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            image,
            mask: None,
            label: label.into(),
            premultiply_alpha: true,
            shader,
        }
    }

    /// Use a separate image as mask (grayscale will be used as alpha)
    pub fn with_mask(mut self, mask: DynamicImage) -> Self {
        self.mask = Some(mask);
        self
    }

    /// Set whether to premultiply alpha
    pub fn with_premultiply_alpha(mut self, premultiply: bool) -> Self {
        self.premultiply_alpha = premultiply;
        self
    }

    /// Process the image with the mask if provided
    fn process_image(&self) -> DynamicImage {
        let mut processed = self.image.clone();

        // If a separate mask is provided, apply it
        if let Some(mask) = &self.mask {
            let (width, height) = processed.dimensions();
            let mask_resized =
                mask.resize_exact(width, height, image::imageops::FilterType::Lanczos3);

            // Apply mask to each pixel
            for y in 0..height {
                for x in 0..width {
                    let mask_pixel = mask_resized.get_pixel(x, y);
                    let mask_alpha = (0.299 * mask_pixel[0] as f32
                        + 0.587 * mask_pixel[1] as f32
                        + 0.114 * mask_pixel[2] as f32) as u8;

                    let mut pixel = processed.get_pixel(x, y);

                    // Use the original alpha or the mask, whichever is lower
                    let final_alpha = u8::min(pixel[3], mask_alpha);

                    // Apply alpha
                    pixel[3] = final_alpha;

                    // Pre-multiply RGB by alpha if requested
                    if self.premultiply_alpha {
                        let alpha_factor = final_alpha as f32 / 255.0;
                        pixel[0] = (pixel[0] as f32 * alpha_factor) as u8;
                        pixel[1] = (pixel[1] as f32 * alpha_factor) as u8;
                        pixel[2] = (pixel[2] as f32 * alpha_factor) as u8;
                    }

                    processed.put_pixel(x, y, pixel);
                }
            }
        }

        processed
    }
}

impl ModelBuilder for EffectModelBuilder {
    type Target = EffectModel;

    fn build(
        &self,
        device: &Device,
        queue: &Queue,
        bindgroup_layout_manager: Arc<Mutex<Manager<BindGroupLayout>>>,
        pipeline_manager: Arc<Mutex<Manager<RenderPipeline>>>,
    ) -> Self::Target {
        // Process the image using any mask provided
        let processed_image = self.process_image();

        // Create texture from the processed image
        let texture = ImageTexture::from_image(device, queue, &processed_image, &self.label);

        // Get or create the bind group layout
        let bind_group_layout = bindgroup_layout_manager.lock().unwrap().get_or_init(
            "effect_bind_group_layout",
            || {
                Arc::new(
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            // Texture binding
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
                            // Sampler binding
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                            // Time uniform binding (needed for effect shaders)
                            wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                        label: Some("effect_bind_group_layout"),
                    }),
                )
            },
        );

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Effect Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Get or create pipeline
        let pipeline =
            pipeline_manager
                .lock()
                .unwrap()
                .get_or_init("effect_render_pipeline", || {
                    // Use the specialized effect shader
                    let shader = device.create_shader_module(self.shader.clone());

                    Arc::new(
                        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some("Effect Render Pipeline"),
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

        // Create the time uniform buffer (with default time = 0)
        let time_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Static Time Buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Initialize the buffer with zero
        queue.write_buffer(&time_buffer, 0, bytemuck::cast_slice(&[0.0f32]));

        // Create bind group for this specific texture
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: time_buffer.as_entire_binding(),
                },
            ],
            label: Some(&format!("effect_bind_group_{}", self.label)),
        });

        EffectModel::new(texture, pipeline.clone(), Arc::new(bind_group))
    }
}

/// Animated effect model that adds time-based animation parameters
#[derive(Debug)]
pub struct AnimatedEffectModel {
    /// The base effect model
    effect: EffectModel,
    /// Animation speed (multiplier)
    speed: f32,
    /// Uniform buffer for time and other animation parameters
    time_buffer: wgpu::Buffer,
    /// Current animation time in seconds
    current_time: f32,
    /// Custom bind group that includes animation parameters
    animated_bind_group: Arc<BindGroup>,
    // Lua script ctx
    // ctx: mlua::Lua,
}

impl AnimatedEffectModel {
    pub fn new(
        effect: EffectModel,
        speed: f32,
        device: &Device,
        time_buffer: wgpu::Buffer,
        animated_bind_group: Arc<BindGroup>,
    ) -> Self {
        Self {
            effect,
            speed,
            time_buffer,
            current_time: 0.0,
            animated_bind_group,
            // ctx: Arc::new(mlua::Lua::new()),
        }
    }

    /// Update the animation time
    pub fn update(&mut self, dt: f32, queue: &Queue) {
        self.current_time += dt * self.speed;

        // Keep time in a reasonable range to avoid floating point precision issues
        if self.current_time > 1000.0 {
            self.current_time -= 1000.0;
        }

        // Update the time uniform buffer
        queue.write_buffer(
            &self.time_buffer,
            0,
            bytemuck::cast_slice(&[self.current_time]),
        );
    }
}

impl Render for AnimatedEffectModel {
    fn pipeline(&self) -> Arc<RenderPipeline> {
        self.effect.pipeline()
    }

    fn bindgroup(&self) -> Arc<BindGroup> {
        self.animated_bind_group.clone()
    }
}

/// Builder for animated effect models
pub struct AnimatedEffectModelBuilder {
    /// The base effect builder
    effect_builder: EffectModelBuilder,
    /// Animation speed multiplier
    speed: f32,
}

impl AnimatedEffectModelBuilder {
    pub fn new(
        image: DynamicImage,
        shader: wgpu::ShaderModuleDescriptor<'static>,
        label: impl Into<String>,
        speed: f32,
    ) -> Self {
        Self {
            effect_builder: EffectModelBuilder::new(image, shader, label),
            speed,
        }
    }

    /// Use a separate image as mask (grayscale will be used as alpha)
    pub fn with_mask(mut self, mask: DynamicImage) -> Self {
        self.effect_builder = self.effect_builder.with_mask(mask);
        self
    }
}

impl ModelBuilder for AnimatedEffectModelBuilder {
    type Target = AnimatedEffectModel;

    fn build(
        &self,
        device: &Device,
        queue: &Queue,
        bindgroup_layout_manager: Arc<Mutex<Manager<BindGroupLayout>>>,
        pipeline_manager: Arc<Mutex<Manager<RenderPipeline>>>,
    ) -> Self::Target {
        // First, build the base effect
        let base_effect = self.effect_builder.build(
            device,
            queue,
            bindgroup_layout_manager.clone(),
            pipeline_manager.clone(),
        );

        // Create the time uniform buffer
        let time_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Animation Time Buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Initialize the buffer with zero
        queue.write_buffer(&time_buffer, 0, bytemuck::cast_slice(&[0.0f32]));

        // Create a bind group layout that includes the time uniform
        let animated_bind_group_layout = bindgroup_layout_manager.lock().unwrap().get_or_init(
            "animated_effect_bind_group_layout",
            || {
                Arc::new(
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            // Texture binding
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
                            // Sampler binding
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                            // Time uniform binding
                            wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                        label: Some("animated_effect_bind_group_layout"),
                    }),
                )
            },
        );

        // Create the animated bind group
        let animated_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &animated_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&base_effect.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&base_effect.texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: time_buffer.as_entire_binding(),
                },
            ],
            label: Some("animated_effect_bind_group"),
        });

        AnimatedEffectModel::new(
            base_effect,
            self.speed,
            device,
            time_buffer,
            Arc::new(animated_bind_group),
        )
    }
}
