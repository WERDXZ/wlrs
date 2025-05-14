//! Shader module containing compiled shader code as constants
//! This module provides easy access to all shader code used in the application

pub const TEXTURE_SHADER: wgpu::ShaderModuleDescriptor<'static> =
    wgpu::include_wgsl!("./texture.wgsl");
pub const COLOR_SHADER: wgpu::ShaderModuleDescriptor<'static> =
    wgpu::include_wgsl!("./color.wgsl");
pub const WAVE_EFFECT_SHADER: wgpu::ShaderModuleDescriptor<'static> =
    wgpu::include_wgsl!("./wave.effect.wgsl");
pub const GLITCH_EFFECT_SHADER: wgpu::ShaderModuleDescriptor<'static> =
    wgpu::include_wgsl!("./glitch.effect.wgsl");
pub const GAUSSIAN_EFFECT_SHADER: wgpu::ShaderModuleDescriptor<'static> =
    wgpu::include_wgsl!("./gaussian.effect.wgsl");
pub const PARTICLE_SHADER: wgpu::ShaderModuleDescriptor<'static> =
    wgpu::include_wgsl!("./particle.wgsl");
