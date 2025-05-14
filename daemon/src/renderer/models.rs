use std::sync::{Arc, Mutex};

use wgpu::{BindGroupLayout, Device, Queue, RenderPipeline};

use super::{manager::Manager, pipeline::Render};

pub mod animated_texture;
pub mod color;
pub mod effect;
// pub mod particle;
pub mod texture;

pub trait ModelBuilder {
    type Target: Render;
    fn build(
        &self,
        device: &Device,
        queue: &Queue,
        bindgroup_layout_manager: Arc<Mutex<Manager<BindGroupLayout>>>,
        pipeline_manager: Arc<Mutex<Manager<RenderPipeline>>>,
    ) -> Self::Target;
}
