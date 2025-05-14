use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
    time::Duration,
};

use common::{manifest::ShaderType, wallpaper::Wallpaper};
use wgpu::{BindGroup, BindGroupLayout, Device, Queue, RenderPipeline};

use crate::renderer::{
    manager::Manager,
    models::{
        animated_texture::AnimatedTextureModelBuilder, color::ColorModelBuilder,
        texture::TextureModelBuilder, ModelBuilder,
    },
};

use super::models::effect::EffectModelBuilder;

pub trait Render: std::fmt::Debug + std::any::Any {
    fn pipeline(&self) -> Arc<RenderPipeline>;
    fn bindgroup(&self) -> Arc<BindGroup>;

    /// Called before rendering to update the model state if needed
    fn pre_render(&mut self, _device: &Device, _dt: Duration) {
        // Default implementation does nothing
    }

    /// Downcast to Any for runtime type checking
    fn as_any(&self) -> &dyn std::any::Any;

    /// Downcast to Any for runtime type checking (mutable)
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

#[derive(Default)]
pub struct Pipelines {
    pub data: Vec<Box<dyn Render>>,
}

impl Pipelines {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn from(
        wallpaper: Wallpaper,
        device: &Device,
        queue: &Queue,
        bindgroup_layout_manager: Arc<Mutex<Manager<BindGroupLayout>>>,
        pipeline_manager: Arc<Mutex<Manager<RenderPipeline>>>,
    ) -> Self {
        let mut pipelines = Self::new();

        // Process all render layers in proper order
        let render_layers = wallpaper.get_layers();

        for render_layer in render_layers {
            match &render_layer.layer_type {
                common::wallpaper::LayerType::Color { color } => {
                    // Create color model
                    let color_model = ColorModelBuilder::from_hex_color(color, &render_layer.name)
                        .build(
                            device,
                            queue,
                            bindgroup_layout_manager.clone(),
                            pipeline_manager.clone(),
                        );
                    pipelines.data.push(Box::new(color_model));
                }
                common::wallpaper::LayerType::Image { image_path } => {
                    // Check if the image is potentially animated based on extension
                    let path_str = image_path.to_string_lossy().to_lowercase();
                    if path_str.ends_with(".webp") || path_str.ends_with(".gif") {
                        // Try to load as an animated texture
                        let model =
                            AnimatedTextureModelBuilder::new(image_path, &render_layer.name)
                                .looping(true)
                                .build(
                                    device,
                                    queue,
                                    bindgroup_layout_manager.clone(),
                                    pipeline_manager.clone(),
                                );
                        {
                            pipelines.data.push(Box::new(model));
                        }
                    } else {
                        // Load regular static image
                        let image = image::ImageReader::open(image_path)
                            .unwrap()
                            .decode()
                            .unwrap();

                        // Add the image layer
                        let texture = TextureModelBuilder::new(image, &render_layer.name).build(
                            device,
                            queue,
                            bindgroup_layout_manager.clone(),
                            pipeline_manager.clone(),
                        );
                        pipelines.data.push(Box::new(texture));
                    }
                }
                common::wallpaper::LayerType::Particle {
                    image_path,
                    script_path,
                    params,
                } => {
                    // Load particle image
                    let image = image::ImageReader::open(image_path)
                        .unwrap()
                        .decode()
                        .unwrap();

                    // Get max particles from params or use default
                    let max_particles = params
                        .get("max_particles")
                        .and_then(|v| v.as_integer())
                        .unwrap_or(1000) as u32;

                    // TODO: Implement particle system
                    let _ = image;
                    let _ = max_particles;
                    let _ = script_path;

                    // For now, just add the image as a texture
                    let texture = TextureModelBuilder::new(image, &render_layer.name).build(
                        device,
                        queue,
                        bindgroup_layout_manager.clone(),
                        pipeline_manager.clone(),
                    );
                    pipelines.data.push(Box::new(texture));
                }
                common::wallpaper::LayerType::Shader {
                    shader_type,
                    image_path,
                    uniforms,
                } => {
                    // Load image if present
                    let image = image_path
                        .as_ref()
                        .map(|path| image::ImageReader::open(path).unwrap().decode().unwrap());

                    // Get shader from shader type
                    let shader = match shader_type {
                        ShaderType::Wave => crate::shaders::WAVE_EFFECT_SHADER,
                        ShaderType::Glitch => crate::shaders::GLITCH_EFFECT_SHADER,
                        ShaderType::Gaussian => crate::shaders::GAUSSIAN_EFFECT_SHADER,
                        ShaderType::Custom(_) => panic!("Custom shaders not supported yet"),
                    };

                    // Build effect model
                    if let Some(img) = image {
                        // Get opacity from the render layer
                        let opacity = render_layer.opacity;

                        // Get shader type from the shader
                        let shader_name = shader.label.unwrap_or("unknown");
                        
                        // Create the effect builder and set parameters
                        let builder =
                            EffectModelBuilder::new(img, shader, render_layer.name.clone())
                                .with_params(uniforms.clone())
                                .with_opacity(opacity);

                        println!("Building effect for shader type: {}", shader_name);
                        
                        // Build the effect model
                        let effect = builder.build(
                            device,
                            queue,
                            bindgroup_layout_manager.clone(),
                            pipeline_manager.clone(),
                        );

                        // Add the effect to pipelines
                        pipelines.data.push(Box::new(effect));
                    } else {
                        // TODO: Handle effects without images
                        println!(
                            "Warning: Shader effect {} has no image and will be skipped",
                            render_layer.name
                        );
                    }
                }
            }
        }

        pipelines
    }
}

impl Deref for Pipelines {
    type Target = Vec<Box<dyn Render>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Pipelines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
