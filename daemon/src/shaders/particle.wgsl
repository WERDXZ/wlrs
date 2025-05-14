// Vertex shader for particle rendering
struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    color: vec4<f32>,
    size: f32,
    rotation: f32,
    life: f32,
    alive: u32,
};

@group(0) @binding(0)
var particle_texture: texture_2d<f32>;
@group(0) @binding(1)
var particle_sampler: sampler;
@group(0) @binding(2)
var<storage, read> particles: array<Particle>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32
) -> VertexOutput {
    var output: VertexOutput;
    var particle = particles[instance_idx];
    
    // Don't render dead particles
    if (particle.alive == 0u) {
        output.position = vec4<f32>(-10.0, -10.0, 0.0, 1.0); // Off-screen
        output.tex_coords = vec2<f32>(0.0, 0.0);
        output.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return output;
    }
    
    // Calculate corner of the quad based on vertex_idx
    // 0: bottom-left, 1: bottom-right, 2: top-left, 3: bottom-right, 4: top-left, 5: top-right
    var corner = vec2<f32>(0.0, 0.0);
    var uv = vec2<f32>(0.0, 0.0);
    
    switch vertex_idx % 6u {
        case 0u: { // Bottom-left
            corner = vec2<f32>(-1.0, -1.0);
            uv = vec2<f32>(0.0, 1.0);
        }
        case 1u: { // Bottom-right
            corner = vec2<f32>(1.0, -1.0);
            uv = vec2<f32>(1.0, 1.0);
        }
        case 2u: { // Top-left
            corner = vec2<f32>(-1.0, 1.0);
            uv = vec2<f32>(0.0, 0.0);
        }
        case 3u: { // Bottom-right (again for second triangle)
            corner = vec2<f32>(1.0, -1.0);
            uv = vec2<f32>(1.0, 1.0);
        }
        case 4u: { // Top-left (again for second triangle)
            corner = vec2<f32>(-1.0, 1.0);
            uv = vec2<f32>(0.0, 0.0);
        }
        case 5u: { // Top-right
            corner = vec2<f32>(1.0, 1.0);
            uv = vec2<f32>(1.0, 0.0);
        }
        default: {}
    }
    
    // Apply rotation to corner
    var sin_rot = sin(particle.rotation);
    var cos_rot = cos(particle.rotation);
    var rotated_corner = vec2<f32>(
        corner.x * cos_rot - corner.y * sin_rot,
        corner.x * sin_rot + corner.y * cos_rot
    );
    
    // Scale by particle size and position
    var final_position = particle.position + rotated_corner * particle.size;
    
    // Set position and pass through color and texture coordinates
    output.position = vec4<f32>(final_position, 0.0, 1.0);
    output.tex_coords = uv;
    output.color = particle.color * vec4<f32>(1.0, 1.0, 1.0, particle.life); // Fade out based on life
    
    return output;
}

@fragment
fn fs_main(
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>
) -> @location(0) vec4<f32> {
    var tex_color = textureSample(particle_texture, particle_sampler, tex_coords);
    return tex_color * color;
}