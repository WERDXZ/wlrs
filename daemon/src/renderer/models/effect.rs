use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use image::{DynamicImage, GenericImage, GenericImageView};
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
    /// Current animation time for animated effects
    pub current_time: f32,
    /// Whether this effect needs time updates
    animated: bool,
    /// Parameters buffer (for updating time)
    params_buffer: Option<wgpu::Buffer>,
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
            current_time: 0.0,
            animated: false,
            params_buffer: None,
        }
    }

    /// Create an animated effect model
    pub fn new_animated(
        texture: ImageTexture,
        render_pipeline: Arc<RenderPipeline>,
        bind_group: Arc<BindGroup>,
        params_buffer: wgpu::Buffer,
    ) -> Self {
        Self {
            texture,
            render_pipeline,
            bind_group,
            current_time: 0.0,
            animated: true,
            params_buffer: Some(params_buffer),
        }
    }

    /// Update effect time for animations
    pub fn update_time(&mut self, dt: Duration, queue: &Queue) {
        if !self.animated || self.params_buffer.is_none() {
            // No debug output to reduce noise
            return;
        }

        // Update time with a larger multiplier to make animations move faster for the demo
        // This makes the animations more noticeable for testing
        let time_scale = 5.0; // 5x faster animations to make effects more obvious
        self.current_time += dt.as_secs_f32() * time_scale;

        // Avoid precision issues by keeping time in reasonable range
        if self.current_time > 1000.0 {
            self.current_time -= 1000.0;
        }

        // Print debug time update more frequently for debugging
        if self.current_time < 0.2 || (self.current_time % 2.0 < 0.1) {
            println!(
                "Updating effect shader time: {:.2} (dt: {:?}, scaled: {:?})",
                self.current_time,
                dt,
                dt.as_secs_f32() * time_scale
            );
        }

        // Write new time to params buffer at the appropriate offset
        // For the new parameter layout:
        // [param1, param2, strength, time] (each f32 = 4 bytes)
        // So time is at offset 12 (3 x 4 bytes)
        queue.write_buffer(
            self.params_buffer.as_ref().unwrap(),
            12, // Offset of 12 bytes (3 x f32)
            bytemuck::cast_slice(&[self.current_time]),
        );

        // Force more frequent updates to prevent animation stalling
        // This is a debug measure to ensure time updates are happening
        println!("Time updated for shader: {:.2}", self.current_time);
    }
}

impl EffectModel {
    /// Check if this effect is animated
    pub fn is_animated(&self) -> bool {
        self.animated
    }
}

impl Render for EffectModel {
    fn pipeline(&self) -> Arc<RenderPipeline> {
        self.render_pipeline.clone()
    }

    fn bindgroup(&self) -> Arc<BindGroup> {
        self.bind_group.clone()
    }

    fn pre_render(&mut self, _device: &Device, _dt: Duration) {
        // Time updates are handled by the layer's draw method with direct queue access
        // No special handling needed here
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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
    /// Effect parameters from manifest
    params: HashMap<String, toml::Value>,
    /// Layer opacity from manifest (0.0 to 1.0)
    opacity: f32,
    /// The shader to use
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
            params: HashMap::new(),
            opacity: 1.0, // Default opacity is 1.0 (fully opaque)
            shader,
        }
    }

    /// Set the layer opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set effect parameters from a parameters map (from manifest)
    pub fn with_params(mut self, params: HashMap<String, toml::Value>) -> Self {
        self.params = params;
        self
    }

    /// Parse a floating point parameter from the params map with a default value
    fn parse_f32_param(&self, param_name: &str, default_value: f32) -> f32 {
        match self.params.get(param_name) {
            Some(value) => {
                // Try to parse the value as a float
                if let Some(float_val) = value.as_float() {
                    float_val as f32
                } else if let Some(int_val) = value.as_integer() {
                    int_val as f32
                } else {
                    println!(
                        "Warning: Parameter '{param_name}' has invalid type, using default: {default_value}"
                    );
                    default_value
                }
            }
            None => {
                println!("Parameter '{param_name}' not found, using default: {default_value}");
                default_value
            }
        }
    }

    /// Parse an integer parameter from the params map with a default value
    fn parse_i32_param(&self, param_name: &str, default_value: i32) -> i32 {
        match self.params.get(param_name) {
            Some(value) => {
                if let Some(int_val) = value.as_integer() {
                    int_val as i32
                } else if let Some(float_val) = value.as_float() {
                    float_val as i32
                } else {
                    println!(
                        "Warning: Parameter '{param_name}' has invalid type, using default: {default_value}"
                    );
                    default_value
                }
            }
            None => {
                println!("Parameter '{param_name}' not found, using default: {default_value}");
                default_value
            }
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

    /// Process the image with the mask if provided and apply opacity
    fn process_image(&self) -> DynamicImage {
        let mut processed = self.image.clone();
        let (width, height) = processed.dimensions();

        // If a separate mask is provided, apply it
        let mask_present = self.mask.is_some();
        let mask_resized = self
            .mask
            .as_ref()
            .map(|mask| mask.resize_exact(width, height, image::imageops::FilterType::Lanczos3));

        // Apply mask and/or opacity to each pixel
        for y in 0..height {
            for x in 0..width {
                let mut pixel = processed.get_pixel(x, y);

                // Start with original alpha
                let mut final_alpha = pixel[3];

                // Apply mask if present
                if let Some(ref mask) = mask_resized {
                    let mask_pixel = mask.get_pixel(x, y);
                    let mask_alpha = (0.299 * mask_pixel[0] as f32
                        + 0.587 * mask_pixel[1] as f32
                        + 0.114 * mask_pixel[2] as f32) as u8;

                    // Use the original alpha or the mask, whichever is lower
                    final_alpha = u8::min(final_alpha, mask_alpha);
                }

                // Apply layer opacity
                final_alpha = (final_alpha as f32 * self.opacity) as u8;

                // Apply final alpha
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
                                    blend: Some(wgpu::BlendState {
                                        color: wgpu::BlendComponent {
                                            src_factor: wgpu::BlendFactor::SrcAlpha,
                                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                            operation: wgpu::BlendOperation::Add,
                                        },
                                        alpha: wgpu::BlendComponent {
                                            src_factor: wgpu::BlendFactor::One,
                                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                            operation: wgpu::BlendOperation::Add,
                                        },
                                    }),
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

        // Create uniform buffer for shader parameters
        // For Gaussian blur, we pass radius and time
        let is_gaussian = matches!(self.shader.label, Some("gaussian.effect.wgsl"));

        // Buffer will contain radius, time, and padding
        let buffer_size = std::mem::size_of::<f32>() * 4; // 16 bytes for alignment
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Effect Parameters Buffer"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Get shader type
        let shader_label = self.shader.label.as_ref().map(|&s| s).unwrap_or("");
        println!("Shader type: {shader_label}");

        // Prepare parameters based on shader type
        let initial_data = if shader_label == "gaussian.effect.wgsl" {
            // Gaussian blur parameters
            println!("Setting up Gaussian blur parameters for {}", self.label);

            // Parse radius from manifest or use default
            let radius = self.parse_f32_param("radius", 3.5f32);

            // Use layer opacity to scale the effect intensity
            let effect_strength = self.opacity;
            let actual_radius = radius * effect_strength;

            println!("Using blur radius: {radius} scaled by opacity: {effect_strength} = {actual_radius}");

            // Parameters: radius, time, opacity (for intensity scaling), padding
            [actual_radius, 0.0f32, effect_strength, 0.0f32]
        } else if shader_label == "glitch.effect.wgsl" {
            // Glitch effect parameters
            println!("Setting up Glitch effect parameters for {}", self.label);

            // Parse parameters from manifest or use defaults
            let intensity = self.parse_f32_param("intensity", 0.5f32); // Strength of the glitch
            let frequency = self.parse_f32_param("frequency", 0.3f32); // How often glitches occur

            // Use layer opacity to scale the effect intensity
            let effect_strength = self.opacity;
            let actual_intensity = intensity * effect_strength;

            println!("Using glitch intensity: {intensity} scaled by opacity: {effect_strength} = {actual_intensity}, frequency: {frequency}");

            // Parameters: intensity, frequency, opacity (for intensity scaling), time
            [actual_intensity, frequency, effect_strength, 0.0f32]
        } else if shader_label == "wave.effect.wgsl" {
            // Wave effect parameters
            println!("Setting up Wave effect parameters for {}", self.label);

            // Parse parameters from manifest or use defaults
            let amplitude = self.parse_f32_param("amplitude", 0.2f32); // Wave height/strength
            let frequency = self.parse_f32_param("frequency", 0.5f32); // Wave density

            // Parse additional wave parameters (these will be ignored by the shader but kept for future expansion)
            let _speed = self.parse_f32_param("speed", 1.0f32); // Animation speed multiplier
            let _complexity = self.parse_f32_param("complexity", 1.0f32); // Wave complexity multiplier
            let _direction = self.parse_f32_param("direction", 0.0f32); // Wave direction (0-360 degrees)

            // Use layer opacity to scale the effect intensity
            let effect_strength = self.opacity;
            let actual_amplitude = amplitude * effect_strength;

            println!("Using wave amplitude: {amplitude} scaled by opacity: {effect_strength} = {actual_amplitude}, frequency: {frequency}");

            // Parameters: amplitude, frequency, opacity (for intensity scaling), time
            [actual_amplitude, frequency, effect_strength, 0.0f32]
        } else {
            // Default parameters for other shaders
            // Include opacity as the third parameter
            [0.0f32, 0.0f32, self.opacity, 0.0f32]
        };

        // Initialize the buffer with the appropriate parameters
        queue.write_buffer(&params_buffer, 0, bytemuck::cast_slice(&initial_data));

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
                    resource: params_buffer.as_entire_binding(),
                },
            ],
            label: Some(&format!("effect_bind_group_{}", self.label)),
        });

        // All shader effects should be animated by default
        // This ensures they all receive time updates for potential animation
        let is_animated = true;

        // Print animation status
        println!("Effect {} is animated: {}", self.label, is_animated);

        if is_animated {
            println!("Effect {} requires time updates for animation", self.label);
            EffectModel::new_animated(
                texture,
                pipeline.clone(),
                Arc::new(bind_group),
                params_buffer,
            )
        } else {
            EffectModel::new(texture, pipeline.clone(), Arc::new(bind_group))
        }
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
    ctx: mlua::Lua,
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
            ctx: mlua::Lua::new(),
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

        // Print current time for debugging
        // println!("Animation time updated: {:.2}", self.current_time);
    }
}

impl Render for AnimatedEffectModel {
    fn pipeline(&self) -> Arc<RenderPipeline> {
        self.effect.pipeline()
    }

    fn bindgroup(&self) -> Arc<BindGroup> {
        self.animated_bind_group.clone()
    }

    fn pre_render(&mut self, _device: &Device, dt: Duration) {
        // In the animated effect model, we update the time directly here
        if self.current_time < 0.1 {
            println!("AnimatedEffectModel pre_render called, will update time");
        }
        // We can't update the time here because we don't have access to queue
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for animated effect models
pub struct AnimatedEffectModelBuilder {
    /// The base effect builder
    effect_builder: EffectModelBuilder,
    /// Animation speed multiplier
    speed: f32,
    /// script
    script: Option<String>,
}

impl AnimatedEffectModelBuilder {
    pub fn new(
        image: DynamicImage,
        shader: wgpu::ShaderModuleDescriptor<'static>,
        label: impl Into<String>,
        speed: f32,
        script: Option<String>,
    ) -> Self {
        Self {
            effect_builder: EffectModelBuilder::new(image, shader, label),
            speed,
            script,
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
        println!(
            "Building animated effect model for {}",
            self.effect_builder.label
        );
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
