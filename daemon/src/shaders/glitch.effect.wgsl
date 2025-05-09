// Glitch effect shader with time uniform

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

// Random function for glitch effect
fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// Fragment shader with glitch effect
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Glitch parameters
    let glitch_amount = 0.005;
    let slice_height = 0.1;
    let slice_time = time * 10.0;
    
    // Calculate glitch offset
    var uv = in.tex_coords;
    let random_offset = rand(vec2<f32>(floor(uv.y * 10.0), floor(slice_time)));
    
    // Apply horizontal glitch offset to some rows
    if (random_offset > 0.95) {
        uv.x += glitch_amount * (random_offset * 2.0 - 1.0);
    }
    
    // RGB split effect
    let color_r = textureSample(t_diffuse, s_diffuse, vec2<f32>(uv.x + sin(time) * 0.01, uv.y)).r;
    let color_g = textureSample(t_diffuse, s_diffuse, uv).g;
    let color_b = textureSample(t_diffuse, s_diffuse, vec2<f32>(uv.x - sin(time) * 0.01, uv.y)).b;
    let color_a = textureSample(t_diffuse, s_diffuse, uv).a;
    
    return vec4<f32>(color_r, color_g, color_b, color_a);
}

