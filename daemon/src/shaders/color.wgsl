// Color shader - Render a solid color rectangle

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

// Full-screen rectangle vertex shader
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Create a rectangle that covers the entire viewport (two triangles)
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );
    
    let pos = positions[in_vertex_index];
    
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    return out;
}

// Uniform binding for the color
struct ColorUniform {
    color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> u_color: ColorUniform;

// Simply output the uniform color
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return u_color.color;
}