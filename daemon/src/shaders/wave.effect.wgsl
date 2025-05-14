// Wave effect shader with time uniform

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

// Wave parameters struct
struct WaveParams {
    // Amplitude of the wave effect
    amplitude: f32,
    // Frequency of the waves
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
var<uniform> params: WaveParams;

// Helper function for 2D noise
fn noise2D(p: vec2<f32>) -> f32 {
    let p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.13);
    let p3b = p3 + vec3<f32>(7.0, 157.0, 113.0);
    let h = fract(p3 * p3b);
    return fract(h.x + h.y + h.z);
}

// Fragment shader with simplified but more visible wave effect
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coords;
    
    // Significantly increase amplitude and scale for more visible effect
    // Apply a much more aggressive scaling to make effects very apparent
    let base_amplitude = (params.amplitude + 1) * 0.2 * (1+ params.strength); // Double the amplitude scale
    let base_frequency = params.frequency * 10.0; // Reduce frequency for larger waves
    
    // Dramatically speed up time for development/testing
    let time_speed = 5.0; // Much faster animation for testing
    
    // Create a multi-wave effect by combining several waves with exaggerated parameters
    // Primary wave - much faster animation
    let time1 = params.time * time_speed;
    let freq1 = base_frequency;
    let amp1 = base_amplitude * 1.5; // Increased primary wave amplitude
    
    // Secondary wave (higher frequency, higher amplitude for more visibility)
    let time2 = params.time * time_speed * 1.3;
    let freq2 = base_frequency * 1.7;
    let amp2 = base_amplitude * 1.2; // Stronger secondary wave
    
    // Simpler wave calculation for better performance and more obvious effect
    // Calculate primary wave offsets
    let wave1_x = sin(uv.y * freq1 + time1) * amp1;
    let wave1_y = cos(uv.x * freq1 + time1 * 0.7) * amp1;
    
    // Calculate secondary wave offsets
    let wave2_x = sin((uv.x + uv.y) * freq2 + time2) * amp2;
    let wave2_y = cos((uv.x - uv.y) * freq2 + time2 * 1.2) * amp2;
    
    // Combine wave offsets - use fewer components for stronger, more visible effect
    let total_offset_x = wave1_x + wave2_x;
    let total_offset_y = wave1_y + wave2_y;
    
    // Apply distortion at full strength everywhere
    let distorted_coords = vec2<f32>(
        uv.x + total_offset_x,
        uv.y + total_offset_y
    );
    
    // Sample color with the distorted coordinates
    let color = textureSample(t_diffuse, s_diffuse, distorted_coords);
    
    // Add more pronounced color shift based on distortion intensity
    let color_shift = abs(total_offset_x + total_offset_y) * 0.5; // Increased color shift
    
    // Animate color shift based on time - more dramatic color changes
    let r_shift = sin(params.time * time_speed * 0.3) * 0.5 + 0.5;
    let g_shift = sin(params.time * time_speed * 0.4 + 2.0) * 0.5 + 0.5;
    let b_shift = sin(params.time * time_speed * 0.5 + 4.0) * 0.5 + 0.5;
    
    // More pronounced color shifting
    let final_color = vec4<f32>(
        color.r + color_shift * 0.2 * r_shift, // More dramatic red shift
        color.g + color_shift * 0.15 * g_shift, // More dramatic green shift
        color.b + color_shift * 0.25 * b_shift, // More dramatic blue shift
        color.a
    );
    
    return final_color;
}
