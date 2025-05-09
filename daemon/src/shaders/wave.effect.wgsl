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

// Texture bindings
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
// Time uniform
@group(0) @binding(2)
var<uniform> time: f32;

// Fragment shader with wave effect
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Apply wave distortion based on time
    let amplitude = 0.01;
    let frequency = 10.0;
    let wave_offset_x = sin(in.tex_coords.y * frequency + time) * amplitude;
    let wave_offset_y = sin(in.tex_coords.x * frequency + time) * amplitude;
    
    // Sample with distorted coordinates
    let distorted_coords = vec2<f32>(
        in.tex_coords.x + wave_offset_x,
        in.tex_coords.y + wave_offset_y
    );
    
    return textureSample(t_diffuse, s_diffuse, distorted_coords);
}
