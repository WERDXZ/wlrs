// Improved Glitch effect shader that uses the overlay as a mask
// The mask intensity controls where the effect is applied

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

// Full-screen triangle
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Use a large triangle to cover the entire screen
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    
    let tex_coords = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0),
    );
    
    out.clip_position = vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
    out.tex_coords = tex_coords[in_vertex_index];
    return out;
}

// Glitch parameters struct
struct GlitchParams {
    // Intensity of the glitch effect (from manifest)
    intensity: f32,
    // Frequency of glitch blocks
    frequency: f32,
    // Effect strength multiplier (from layer opacity)
    strength: f32, 
    // Time for animation
    time: f32,
};

// Texture bindings
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
// Parameters uniform
@group(0) @binding(2)
var<uniform> params: GlitchParams;

// Random function for glitch effect
fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// We don't need a separate function for the underlying screen anymore
// The shader will just apply the glitch effect to the mask texture itself

// Fragment shader with improved glitch effect
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get the original texture color
    let original = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Calculate mask strength from the texture itself
    let mask_strength = original.a;
    
    // If there's no mask (transparent), return transparent pixel
    if (mask_strength < 0.01) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
    // Glitch parameters - use values from uniform with strength multiplier
    // Increase the glitch amount for more visible effect
    let glitch_amount = params.intensity * 0.1 * params.strength;
    let slice_count = max(5.0, params.frequency * 30.0);
    let slice_time = params.time * (3.0 + params.frequency * 10.0); // Faster changes
    
    // Calculate glitch offset
    var uv = in.tex_coords;
    let slice_y = floor(uv.y * slice_count);
    let random_val = rand(vec2<f32>(slice_y, floor(slice_time)));
    
    // More frequent horizontal glitches
    if (random_val > 0.8) { // Increased frequency (0.93 -> 0.8)
        uv.x += glitch_amount * (random_val * 2.0 - 1.0);
    }
    
    // More frequent vertical glitching
    if (random_val < 0.15) { // Increased frequency (0.05 -> 0.15)
        uv.y += glitch_amount * 0.5; // Stronger effect (0.25 -> 0.5)
    }
    
    // Add block shifting based on time
    let block_shift = floor(uv.y * 15.0);
    if (mod(block_shift + floor(params.time * 2.0), 5.0) < 1.0) {
        uv.x += sin(params.time * 10.0 + block_shift) * glitch_amount * 0.3;
    }
    
    // Sample the texture with glitched coordinates
    let glitched = textureSample(t_diffuse, s_diffuse, uv);
    
    // Enhanced RGB split effect with more dramatic color separation
    let split_amount = 0.03 * params.strength; // Increased from 0.01
    let time_factor = params.time * 2.0; // Faster oscillation
    
    // Use separate timings for each color channel for more dynamic effect
    let uv_r = vec2<f32>(uv.x + sin(time_factor) * split_amount, uv.y + cos(time_factor * 0.7) * split_amount * 0.3);
    let uv_g = vec2<f32>(uv.x, uv.y);  // Keep green channel at original position
    let uv_b = vec2<f32>(uv.x - sin(time_factor * 1.3) * split_amount, uv.y - cos(time_factor) * split_amount * 0.3);
    
    // Sample all three color channels with their own offsets
    let color_r = textureSample(t_diffuse, s_diffuse, uv_r).r;
    let color_g = textureSample(t_diffuse, s_diffuse, uv_g).g;
    let color_b = textureSample(t_diffuse, s_diffuse, uv_b).b;
    
    // Add more intense noise with time variation
    let noise = rand(uv + vec2<f32>(params.time * 0.5, params.time * 0.3)) * 0.15;
    
    // Final color with RGB split and noise
    let final_color = vec4<f32>(
        color_r + noise, 
        color_g + noise * 0.5, 
        color_b + noise * 0.25, 
        original.a  // Keep original alpha
    );
    
    return final_color;
}

