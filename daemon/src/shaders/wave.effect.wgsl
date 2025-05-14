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

// Fragment shader with enhanced dynamic wave effect
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coords;
    
    // Apply wave distortion based on parameters with strength multiplier
    let base_amplitude = params.amplitude * 0.05 * params.strength; // Scale amplitude (0-1 -> 0-0.05) and apply strength
    let base_frequency = params.frequency * 20.0; // Scale frequency (0-1 -> 0-20)
    
    // Create a multi-wave effect by combining several waves
    // Primary wave
    let time1 = params.time * 0.5;
    let freq1 = base_frequency;
    let amp1 = base_amplitude;
    
    // Secondary wave (higher frequency, lower amplitude)
    let time2 = params.time * 0.8;
    let freq2 = base_frequency * 2.3;
    let amp2 = base_amplitude * 0.6;
    
    // Tertiary wave (different direction, lower frequency)
    let time3 = params.time * 0.3;
    let freq3 = base_frequency * 0.7;
    let amp3 = base_amplitude * 0.4;
    
    // Add small noise variations to make waves less uniform
    let noise_factor = noise2D(vec2<f32>(uv.x * 5.0 + time1, uv.y * 5.0 - time2)) * 0.2 * base_amplitude;
    
    // Calculate primary wave offsets
    let wave1_x = sin(uv.y * freq1 + time1) * amp1;
    let wave1_y = cos(uv.x * freq1 + time1 * 0.7) * amp1;
    
    // Calculate secondary wave offsets (different direction)
    let wave2_x = sin((uv.x + uv.y) * freq2 + time2) * amp2;
    let wave2_y = cos((uv.x - uv.y) * freq2 + time2 * 1.2) * amp2;
    
    // Calculate tertiary wave offsets (circular pattern)
    let dist_from_center = length(uv - vec2<f32>(0.5, 0.5));
    let wave3_x = sin(dist_from_center * freq3 - time3) * amp3;
    let wave3_y = cos(dist_from_center * freq3 - time3 * 1.1) * amp3;
    
    // Combine all wave offsets
    let total_offset_x = wave1_x + wave2_x + wave3_x + noise_factor;
    let total_offset_y = wave1_y + wave2_y + wave3_y + noise_factor;
    
    // Apply dynamic amplitude scaling based on position
    // Waves are stronger near edges and weaker in the center
    let center_weight = 1.0 - smoothstep(0.0, 0.8, dist_from_center);
    let edge_factor = mix(1.0, 0.6, center_weight);
    
    // Sample with distorted coordinates
    let distorted_coords = vec2<f32>(
        uv.x + total_offset_x * edge_factor,
        uv.y + total_offset_y * edge_factor
    );
    
    // Sample color with the distorted coordinates
    let color = textureSample(t_diffuse, s_diffuse, distorted_coords);
    
    // Add subtle color shift based on distortion intensity
    let color_shift = abs(total_offset_x + total_offset_y) * 0.1;
    let final_color = vec4<f32>(
        color.r + color_shift * 0.02,
        color.g + color_shift * 0.01,
        color.b + color_shift * 0.03,
        color.a
    );
    
    return final_color;
}
