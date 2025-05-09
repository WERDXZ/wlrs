use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use common::{
    manifest::{EffectType, ShaderType},
    wallpaper::Wallpaper,
};
use wgpu::{BindGroup, BindGroupLayout, Device, Queue, RenderPipeline};

use crate::renderer::{
    manager::Manager,
    models::{texture::TextureModelBuilder, ModelBuilder},
};

use super::models::effect::EffectModelBuilder;

pub trait Render: std::fmt::Debug + Send + Sync {
    fn pipeline(&self) -> Arc<RenderPipeline>;
    fn bindgroup(&self) -> Arc<BindGroup>;
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
        // background:
        {
            let image = wallpaper
                .manifest
                .background
                .image
                .map(|file| image::ImageReader::open(file).unwrap().decode().unwrap());
            let texture = image.map(|image| {
                TextureModelBuilder::new(image, "background-image").build(
                    device,
                    queue,
                    bindgroup_layout_manager.clone(),
                    pipeline_manager.clone(),
                )
            });
            if let Some(v) = texture {
                pipelines.data.push(Box::new(v))
            }
        }
        // Effects:
        {
            let effects = wallpaper.manifest.effects;
            effects
                .iter()
                .map(|effect| {
                    EffectModelBuilder::new(
                        effect
                            .image
                            .as_ref()
                            .map(|file| image::ImageReader::open(file).unwrap().decode().unwrap())
                            .expect("some image to be present"),
                        match &effect.effect_type {
                            EffectType::Particles => panic!("not supported yet"),
                            EffectType::Shader(v) => match &v {
                                ShaderType::Wave => crate::shaders::WAVE_EFFECT_SHADER,
                                ShaderType::Glitch => crate::shaders::GLITCH_EFFECT_SHADER,
                                ShaderType::Custom(v) => panic!("not supported yet"),
                            },
                        },
                        effect.name.clone(),
                    )
                    .build(
                        device,
                        queue,
                        bindgroup_layout_manager.clone(),
                        pipeline_manager.clone(),
                    )
                })
                .for_each(|v| {
                    pipelines.data.push(Box::new(v));
                });
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
