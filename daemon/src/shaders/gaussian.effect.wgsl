// Improved Gaussian Blur Shader
// Performs a weighted Gaussian blur based on radius parameter

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

// Vertex shader for fullscreen quad
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Create a fullscreen triangle
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    
    // Texture coordinates
    var texcoords = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),  // Bottom-left
        vec2<f32>(2.0, 1.0),  // Bottom-right
        vec2<f32>(0.0, -1.0)  // Top-left
    );
    
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.tex_coords = texcoords[vertex_index];
    
    return output;
}

// Blur parameters struct
struct BlurParams {
    // Blur radius (from manifest)
    radius: f32,
    // Time for subtle animation
    time: f32, 
    // Effect strength multiplier (from layer opacity)
    strength: f32,
    // Padding to align to 16 bytes
    padding: f32,
};

// Binding group for texture operations
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> params: BlurParams;

// Fragment shader for Gaussian blur
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get texture dimensions
    let texture_size = vec2<f32>(
        f32(textureDimensions(input_texture).x), 
        f32(textureDimensions(input_texture).y)
    );
    
    // Get radius from uniforms and apply strength multiplier
    let blur_radius = max(params.radius, 0.1) * params.strength;  
    let blur_direction = vec2<f32>(1.0, 1.0);  // Blur in both directions
    
    // Calculate pixel size
    let pixel_size = 1.0 / texture_size;
    
    // Gaussian weights for 5 samples (optimized for better blur quality)
    let weights = array<f32, 5>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);
    
    // Sample center texel
    var result = textureSample(input_texture, texture_sampler, in.tex_coords) * weights[0];
    var total_weight = weights[0];
    
    // First pass - horizontal blur
    for (var i = 1; i < 5; i++) {
        let weight = weights[i];
        let offset = pixel_size * vec2<f32>(blur_direction.x, 0.0) * f32(i) * blur_radius;
        
        // Sample in positive direction
        result += textureSample(input_texture, texture_sampler, in.tex_coords + offset) * weight;
        // Sample in negative direction
        result += textureSample(input_texture, texture_sampler, in.tex_coords - offset) * weight;
        
        total_weight += 2.0 * weight;
    }
    
    // Reset for vertical pass
    var temp = result / total_weight;
    result = temp * weights[0];
    total_weight = weights[0];
    
    // Second pass - vertical blur
    for (var i = 1; i < 5; i++) {
        let weight = weights[i];
        let offset = pixel_size * vec2<f32>(0.0, blur_direction.y) * f32(i) * blur_radius;
        
        // Sample in positive direction
        result += textureSample(input_texture, texture_sampler, in.tex_coords + offset) * weight;
        // Sample in negative direction
        result += textureSample(input_texture, texture_sampler, in.tex_coords - offset) * weight;
        
        total_weight += 2.0 * weight;
    }
    
    // Use the time uniform to create a subtle pulsing effect (optional)
    // Scale the pulse by the strength parameter for more or less intensity
    let pulse_amount = params.strength * 0.1; // 0-10% pulse based on strength
    let pulse = (sin(params.time * 0.5) * pulse_amount) + 1.0;
    
    // Normalize the result and apply subtle time-based effect
    // Also apply the strength to the overall opacity to match layer opacity setting
    let final_color = (result / total_weight) * pulse;
    final_color.a *= params.strength; // Apply strength to alpha
    
    return final_color;
}
