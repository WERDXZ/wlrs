// Gaussian Blur Shader
// Performs a two-pass Gaussian blur on the input texture

// Vertex shader for both horizontal and vertical blur passes
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Create a fullscreen triangle
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

// Binding group for texture operations
@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> params: BlurParams;

// Parameters for the blur effect
struct BlurParams {
    // Direction: (1.0, 0.0) for horizontal, (0.0, 1.0) for vertical
    direction: vec2<f32>,
    // Blur strength (sigma)
    strength: f32,
    // Texture size
    texture_size: vec2<f32>,
}

// Fragment shader for Gaussian blur
@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    // Calculate texture coordinates
    let tex_coords = vec2<f32>(pos.xy) / params.texture_size;
    
    // Normalized pixel size for the blur direction
    let pixel_size = 1.0 / params.texture_size;
    
    // Gaussian weights (approximation for a 5x5 kernel)
    let weights = array<f32, 5>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);
    
    // Sample center texel
    var result = textureSample(texture, texture_sampler, tex_coords) * weights[0];
    var total_weight = weights[0];
    
    // Apply blur in the specified direction
    for (var i = 1; i < 5; i++) {
        let weight = weights[i];
        let offset = pixel_size * params.direction * f32(i) * params.strength;
        
        // Sample in positive direction
        result += textureSample(texture, texture_sampler, tex_coords + offset) * weight;
        // Sample in negative direction
        result += textureSample(texture, texture_sampler, tex_coords - offset) * weight;
        
        total_weight += 2.0 * weight;
    }
    
    // Normalize the result
    return result / total_weight;
}
