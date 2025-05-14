use std::{
    sync::{Arc, Mutex},
    path::Path,
    fs,
};

use image::DynamicImage;
use mlua::{Lua, Function, Table, Value, FromLua};
use wgpu::{BindGroup, BindGroupLayout, Device, Queue, RenderPipeline, Buffer};

use crate::{
    asset::image::ImageTexture,
    renderer::{manager::Manager, models::ModelBuilder, pipeline::Render},
};

/// A single particle in the particle system
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Particle {
    pub position: [f32; 2], // x, y normalized coords (-1.0 to 1.0)
    pub velocity: [f32; 2], // movement direction and speed
    pub color: [f32; 4],    // rgba color
    pub size: f32,          // particle size
    pub rotation: f32,      // rotation in radians
    pub life: f32,          // remaining lifetime (0.0 to 1.0)
    pub alive: u32,         // 1 if alive, 0 if dead (for GPU filtering)
}

// Make Particle compatible with GPU buffers
unsafe impl bytemuck::Pod for Particle {}
unsafe impl bytemuck::Zeroable for Particle {}

impl Particle {
    pub fn new(x: f32, y: f32, size: f32) -> Self {
        Self {
            position: [x, y],
            velocity: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            size,
            rotation: 0.0,
            life: 1.0,
            alive: 1,
        }
    }
}

/// Represents a collection of particles controlled by a Lua script
#[derive(Debug)]
pub struct ParticleModel {
    /// The texture for rendering particles
    texture: ImageTexture,
    /// The render pipeline for particles
    render_pipeline: Arc<RenderPipeline>,
    /// The bind group for the particle system
    bind_group: Arc<BindGroup>,
    /// Buffer containing particle data
    particle_buffer: Buffer,
    /// Maximum number of particles
    max_particles: u32,
    /// Current number of active particles
    active_particles: u32,
    /// Lua context for particle simulation
    lua: Lua,
    /// Current simulation time
    time: f32,
    /// Update script
    update_script: Option<String>,
    /// Current particle data (CPU side)
    particles: Vec<Particle>,
}

impl ParticleModel {
    pub fn new(
        texture: ImageTexture,
        render_pipeline: Arc<RenderPipeline>,
        bind_group: Arc<BindGroup>,
        particle_buffer: Buffer,
        max_particles: u32,
        update_script: Option<String>,
    ) -> Self {
        // Create Lua environment and load standard libraries
        let lua = Lua::new();
        
        // Load the standard libraries
        lua.load_from_std_lib(mlua::StdLib::ALL).expect("Failed to load Lua standard libraries");
        
        // Pre-allocate particle array
        let mut particles = Vec::with_capacity(max_particles as usize);
        for _ in 0..max_particles {
            particles.push(Particle::new(0.0, 0.0, 0.1)); // Default inactive particles
        }

        // Initialize all particles as inactive
        for particle in &mut particles {
            particle.alive = 0;
        }

        Self {
            texture,
            render_pipeline,
            bind_group,
            particle_buffer,
            max_particles,
            active_particles: 0,
            lua,
            time: 0.0,
            update_script,
            particles,
        }
    }

    /// Update the particle simulation
    pub fn update(&mut self, delta_time: f32, queue: &Queue) {
        self.time += delta_time;

        if let Some(script_path) = &self.update_script {
            // Execute the Lua script to update particles
            self.update_particles_with_lua(delta_time);
        } else {
            // Use a simple built-in update if no script is provided
            self.update_particles_builtin(delta_time);
        }

        // Upload updated particle data to the GPU
        queue.write_buffer(
            &self.particle_buffer,
            0,
            bytemuck::cast_slice(&self.particles),
        );
    }

    /// Simple built-in particle update function for when no Lua script is provided
    fn update_particles_builtin(&mut self, delta_time: f32) {
        // Count how many alive particles we have
        let mut alive_count = 0;

        // Update each particle
        for i in 0..self.max_particles as usize {
            let particle = &mut self.particles[i];
            
            if particle.alive == 1 {
                // Update position based on velocity
                particle.position[0] += particle.velocity[0] * delta_time;
                particle.position[1] += particle.velocity[1] * delta_time;
                
                // Update rotation
                particle.rotation += 0.1 * delta_time;
                
                // Decrease lifetime
                particle.life -= delta_time;
                
                // If lifetime is over, mark as dead
                if particle.life <= 0.0 {
                    particle.alive = 0;
                } else {
                    // Fade out as lifetime decreases
                    particle.color[3] = particle.life;
                    alive_count += 1;
                }
            }
        }

        // Spawn new particles if needed (simple fountain effect)
        if alive_count < (self.max_particles as usize / 2) {
            // Randomly spawn some new particles
            let spawn_count = 5.min(self.max_particles as usize - alive_count);
            
            for _ in 0..spawn_count {
                // Find an inactive particle slot
                if let Some(idx) = self.particles.iter().position(|p| p.alive == 0) {
                    let p = &mut self.particles[idx];
                    
                    // Reset the particle
                    p.position = [0.0, -0.5]; // Start at bottom center
                    p.velocity = [
                        (rand::random::<f32>() - 0.5) * 0.5, // Random x velocity
                        rand::random::<f32>() * 0.5,         // Upward y velocity
                    ];
                    p.color = [
                        rand::random::<f32>(), 
                        rand::random::<f32>(), 
                        rand::random::<f32>(), 
                        1.0,
                    ];
                    p.size = 0.02 + rand::random::<f32>() * 0.03;
                    p.rotation = rand::random::<f32>() * std::f32::consts::PI * 2.0;
                    p.life = 0.5 + rand::random::<f32>() * 1.0;
                    p.alive = 1;
                }
            }
        }

        // Update active count
        self.active_particles = alive_count as u32;
    }

    /// Update particles using the Lua script
    fn update_particles_with_lua(&mut self, delta_time: f32) {
        if let Some(script_path) = &self.update_script {
            // Try to load the script if not already loaded
            let script_content = match fs::read_to_string(script_path) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading Lua script {}: {}", script_path, err);
                    return;
                }
            };

            // Set up the Lua environment
            let globals = self.lua.globals();
            
            // Pass in delta_time and time to Lua
            globals.set("delta_time", delta_time).unwrap();
            globals.set("time", self.time).unwrap();
            globals.set("max_particles", self.max_particles).unwrap();
            globals.set("active_particles", self.active_particles).unwrap();

            // Helper function to emit a particle
            let emit_fn = self.lua.create_function(|lua, args: Table| {
                // Extract arguments
                let x = args.get::<_, Option<f32>>("x").unwrap_or(0.0);
                let y = args.get::<_, Option<f32>>("y").unwrap_or(0.0);
                let vx = args.get::<_, Option<f32>>("vx").unwrap_or(0.0);
                let vy = args.get::<_, Option<f32>>("vy").unwrap_or(0.0);
                let size = args.get::<_, Option<f32>>("size").unwrap_or(0.05);
                let life = args.get::<_, Option<f32>>("life").unwrap_or(1.0);
                let r = args.get::<_, Option<f32>>("r").unwrap_or(1.0);
                let g = args.get::<_, Option<f32>>("g").unwrap_or(1.0);
                let b = args.get::<_, Option<f32>>("b").unwrap_or(1.0);
                let a = args.get::<_, Option<f32>>("a").unwrap_or(1.0);
                let rotation = args.get::<_, Option<f32>>("rotation").unwrap_or(0.0);

                // This is now a Lua userdata - we'll need to convert it back to our particles Vec
                let particles_ref = lua.globals().get::<_, mlua::AnyUserData>("_particles_ref").expect("Particles reference not found");
                
                // Get a mutable reference to the particles vector
                let result: mlua::Result<()> = (|particles_ref: &mlua::AnyUserData| {
                    let mut particles = particles_ref.borrow_mut::<Vec<Particle>>()?;
                    
                    // Find an inactive particle
                    if let Some(idx) = particles.iter().position(|p| p.alive == 0) {
                        let p = &mut particles[idx];
                        
                        // Reset the particle with the specified values
                        p.position = [x, y];
                        p.velocity = [vx, vy];
                        p.color = [r, g, b, a];
                        p.size = size;
                        p.rotation = rotation;
                        p.life = life;
                        p.alive = 1;
                    }
                    
                    Ok(())
                })(particles_ref);

                if let Err(e) = result {
                    eprintln!("Error in emit_particle: {}", e);
                }
                
                Ok(())
            }).expect("Failed to create emit function");
            
            globals.set("emit_particle", emit_fn).unwrap();
            
            // Helper function to update a particle
            let update_particle_fn = self.lua.create_function(|lua, (index, args): (usize, Table)| {
                // This is a Lua userdata - we'll need to convert it back to our particles Vec
                let particles_ref = lua.globals().get::<_, mlua::AnyUserData>("_particles_ref").expect("Particles reference not found");
                
                // Get a mutable reference to the particles vector
                let result: mlua::Result<()> = (|particles_ref: &mlua::AnyUserData| {
                    let mut particles = particles_ref.borrow_mut::<Vec<Particle>>()?;
                    
                    if index < particles.len() {
                        let p = &mut particles[index];
                        
                        // Only update the fields that are specified
                        if let Ok(x) = args.get::<_, f32>("x") { p.position[0] = x; }
                        if let Ok(y) = args.get::<_, f32>("y") { p.position[1] = y; }
                        if let Ok(vx) = args.get::<_, f32>("vx") { p.velocity[0] = vx; }
                        if let Ok(vy) = args.get::<_, f32>("vy") { p.velocity[1] = vy; }
                        if let Ok(size) = args.get::<_, f32>("size") { p.size = size; }
                        if let Ok(life) = args.get::<_, f32>("life") { p.life = life; }
                        if let Ok(r) = args.get::<_, f32>("r") { p.color[0] = r; }
                        if let Ok(g) = args.get::<_, f32>("g") { p.color[1] = g; }
                        if let Ok(b) = args.get::<_, f32>("b") { p.color[2] = b; }
                        if let Ok(a) = args.get::<_, f32>("a") { p.color[3] = a; }
                        if let Ok(rotation) = args.get::<_, f32>("rotation") { p.rotation = rotation; }
                        if let Ok(alive) = args.get::<_, bool>("alive") { p.alive = if alive { 1 } else { 0 }; }
                    }
                    
                    Ok(())
                })(particles_ref);

                if let Err(e) = result {
                    eprintln!("Error in update_particle: {}", e);
                }
                
                Ok(())
            }).expect("Failed to create update_particle function");
            
            globals.set("update_particle", update_particle_fn).unwrap();

            // Helper function to get particle data
            let get_particle_fn = self.lua.create_function(|lua, index: usize| {
                // This is a Lua userdata - convert it back to our particles Vec
                let particles_ref = lua.globals().get::<_, mlua::AnyUserData>("_particles_ref").expect("Particles reference not found");
                
                // Create a result table
                let result_table = lua.create_table()?;
                
                // Get a reference to the particles vector
                let result: mlua::Result<Table> = (|particles_ref: &mlua::AnyUserData| {
                    let particles = particles_ref.borrow::<Vec<Particle>>()?;
                    
                    if index < particles.len() {
                        let p = &particles[index];
                        
                        // Create a new table with the particle data
                        let result_table = lua.create_table()?;
                        result_table.set("x", p.position[0])?;
                        result_table.set("y", p.position[1])?;
                        result_table.set("vx", p.velocity[0])?;
                        result_table.set("vy", p.velocity[1])?;
                        result_table.set("size", p.size)?;
                        result_table.set("life", p.life)?;
                        result_table.set("r", p.color[0])?;
                        result_table.set("g", p.color[1])?;
                        result_table.set("b", p.color[2])?;
                        result_table.set("a", p.color[3])?;
                        result_table.set("rotation", p.rotation)?;
                        result_table.set("alive", p.alive == 1)?;
                        
                        Ok(result_table)
                    } else {
                        // Return an empty table if the index is out of bounds
                        Ok(lua.create_table()?)
                    }
                })(particles_ref);

                match result {
                    Ok(table) => Ok(table),
                    Err(e) => {
                        eprintln!("Error in get_particle: {}", e);
                        Ok(lua.create_table()?)
                    }
                }
            }).expect("Failed to create get_particle function");
            
            globals.set("get_particle", get_particle_fn).unwrap();

            // Register our particle array with Lua
            let particles_userdata = self.lua.create_userdata(self.particles.clone())
                .expect("Failed to create particles userdata");
            
            globals.set("_particles_ref", particles_userdata).unwrap();

            // Define a random function for Lua
            let random_fn = self.lua.create_function(|_, (min, max): (f32, f32)| {
                Ok(min + (max - min) * rand::random::<f32>())
            }).expect("Failed to create random function");
            
            globals.set("random", random_fn).unwrap();

            // Execute the Lua script
            if let Err(err) = self.lua.load(&script_content).exec() {
                eprintln!("Error running Lua script: {}", err);
                return;
            }

            // Call the update function if it exists
            if let Ok(update_fn) = globals.get::<_, Function>("update") {
                if let Err(err) = update_fn.call::<_, ()>(()) {
                    eprintln!("Error calling Lua update function: {}", err);
                }
            }

            // Retrieve the updated particles from Lua
            if let Ok(particles_ref) = globals.get::<_, mlua::AnyUserData>("_particles_ref") {
                if let Ok(updated_particles) = particles_ref.take::<Vec<Particle>>() {
                    self.particles = updated_particles;
                    
                    // Count active particles
                    self.active_particles = self.particles.iter().filter(|p| p.alive == 1).count() as u32;
                }
            }
        }
    }
}

impl Render for ParticleModel {
    fn pipeline(&self) -> Arc<RenderPipeline> {
        self.render_pipeline.clone()
    }

    fn bindgroup(&self) -> Arc<BindGroup> {
        self.bind_group.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct ParticleModelBuilder {
    /// Particle texture image
    particle_image: DynamicImage,
    /// Maximum number of particles to simulate
    max_particles: u32,
    /// Lua script to control particle behavior (relative to wallpaper path)
    script_path: Option<String>,
    /// Label for this particle system
    label: String,
}

impl ParticleModelBuilder {
    pub fn new(
        particle_image: DynamicImage,
        max_particles: u32,
        script_path: Option<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            particle_image,
            max_particles,
            script_path,
            label: label.into(),
        }
    }
}

impl ModelBuilder for ParticleModelBuilder {
    type Target = ParticleModel;

    fn build(
        &self,
        device: &Device,
        queue: &Queue,
        bindgroup_layout_manager: Arc<Mutex<Manager<BindGroupLayout>>>,
        pipeline_manager: Arc<Mutex<Manager<RenderPipeline>>>,
    ) -> Self::Target {
        // Create texture from the particle image
        let texture = ImageTexture::from_image(device, queue, &self.particle_image, &self.label);

        // Create the particle data buffer (initialize with zeros)
        let particle_buffer_size = std::mem::size_of::<Particle>() * self.max_particles as usize;
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Particle Buffer {}", self.label)),
            size: particle_buffer_size as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Get or create the bind group layout for particles
        let bind_group_layout = bindgroup_layout_manager.lock().unwrap().get_or_init(
            "particle_bind_group_layout",
            || {
                Arc::new(
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            // Particle texture
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                },
                                count: None,
                            },
                            // Sampler
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                            // Particle buffer (vertex buffer)
                            wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                        label: Some("particle_bind_group_layout"),
                    }),
                )
            },
        );

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create or get the pipeline
        let pipeline = pipeline_manager.lock().unwrap().get_or_init(
            "particle_render_pipeline",
            || {
                // Create the shader for particles
                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Particle Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/particle.wgsl").into()),
                });

                Arc::new(
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Particle Render Pipeline"),
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main"),
                            buffers: &[],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fs_main"),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                            compilation_options: wgpu::PipelineCompilationOptions::default(),
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            unclipped_depth: false,
                            conservative: false,
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                        cache: None,
                    }),
                )
            },
        );

        // Create bind group for this particle system
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: particle_buffer.as_entire_binding(),
                },
            ],
            label: Some(&format!("particle_bind_group_{}", self.label)),
        });

        ParticleModel::new(
            texture,
            pipeline.clone(),
            Arc::new(bind_group),
            particle_buffer,
            self.max_particles,
            self.script_path.clone(),
        )
    }
}