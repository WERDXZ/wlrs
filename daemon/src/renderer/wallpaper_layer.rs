use std::{
    ptr::NonNull,
    time::{Duration, Instant},
};

use crate::renderer::config::OutputConfig;
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::shell::{
    wlr_layer::{Anchor, KeyboardInteractivity, LayerSurface},
    WaylandSurface,
};
use wayland_client::{protocol::wl_output::WlOutput, Connection, Proxy, QueueHandle};
use wgpu::{
    Adapter, CompositeAlphaMode, Device, PresentMode, Queue, RenderPipeline, Surface,
    SurfaceConfiguration, SurfaceTargetUnsafe, TextureUsages,
};

use super::{client::Client, pipeline::Pipelines};

#[allow(dead_code)]
pub struct WallpaperLayer {
    pub name: String,
    pub layer: LayerSurface,
    pub output: WlOutput,
    pub damaged: bool,
    pub configured: bool,
    pub wallpaper: Pipelines, // Render pipelines for this wallpaper

    pub width: u32,
    pub height: u32,

    pub framerate: Option<u64>,
    pub tickrate: Option<u64>,

    config: OutputConfig,
    surface: Surface<'static>,
    pipeline: Option<RenderPipeline>,
    frame_counter: u32,
    frames_per_update: u32,
    tick_counter: u32,
    ticks_per_update: u32,

    // Animation timing
    last_animation_update: Instant,
}

impl PartialEq<WallpaperLayer> for WallpaperLayer {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl WallpaperLayer {
    pub fn new(
        state: &Client,
        connection: &Connection,
        qh: &QueueHandle<Client>,
        output: &WlOutput,
    ) -> Self {
        let info = state
            .output
            .info(output)
            .expect("An Wayland Output detected but not found");
        let layer = state.new_layer(qh, output);
        layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::BOTTOM | Anchor::RIGHT);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_margin(0, 0, 0, 0);
        layer.set_exclusive_zone(-1);

        layer.commit();

        let surface = unsafe {
            state
                .instance
                .create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                        NonNull::new(connection.backend().display_ptr() as *mut _).unwrap(),
                    )),
                    raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(
                        NonNull::new(layer.wl_surface().id().as_ptr() as *mut _).unwrap(),
                    )),
                })
                .unwrap()
        };

        Self {
            name: info.name.unwrap_or("UNKNOWN".to_string()),
            layer,
            output: output.clone(),
            damaged: true,
            configured: false,
            width: 0,
            height: 0,
            wallpaper: Pipelines::new(),
            config: OutputConfig::default(),
            surface,
            pipeline: None,
            framerate: None,
            tickrate: None,
            frame_counter: 0,
            frames_per_update: 1, // Will redraw every frame by default
            tick_counter: 0,
            ticks_per_update: 1, // Will update animations every frame by default
            last_animation_update: Instant::now(),
        }
    }

    pub fn request_compositor_update(&mut self, qh: &QueueHandle<Client>) {
        // Request a frame callback from the compositor
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());
    }

    pub fn get_recommended_update_interval(&self) -> Option<Duration> {
        match (self.framerate, self.tickrate) {
            (None, None) => None,
            (None, Some(tickrate)) => {
                // If only tickrate is set, use it for update interval
                Some(Duration::from_millis(1000 / tickrate))
            }
            (Some(framerate), None) => {
                // If only framerate is set, use it for update interval
                Some(Duration::from_millis(1000 / framerate))
            }
            (Some(framerate), Some(tickrate)) => {
                // If both are set, use the lower of the two
                let framerate_duration = Duration::from_millis(1000 / framerate);
                let tickrate_duration = Duration::from_millis(1000 / tickrate);
                Some(framerate_duration.min(tickrate_duration))
            }
        }
    }

    pub fn configure(&mut self, adapter: &Adapter, device: &Device) {
        self.configured = true;
        self.damaged = true;
        let capability = self.surface.get_capabilities(adapter);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: capability.formats[0],
            view_formats: capability.formats,
            alpha_mode: CompositeAlphaMode::Auto,
            width: self.width,
            height: self.height,
            desired_maximum_frame_latency: 2,
            present_mode: PresentMode::Mailbox,
        };

        // Configure the surface with the new configuration
        self.surface.configure(device, &config);
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            println!("No size change for layer {}", self.name);
            return;
        }
        self.width = width;
        self.height = height;
        self.damaged = true;
    }

    /// Set the frames per update rate based on the wallpaper's framerate
    /// This controls how often the wallpaper is redrawn
    pub fn set_framerate(&mut self, framerate: i32) {
        // Default system refresh rate assumed to be 60 Hz
        const SYSTEM_FPS: u32 = 60;

        if framerate < 0 {
            // Any negative value: Use compositor-driven timing
            // This means we'll redraw every time the compositor requests a frame
            self.frames_per_update = 0; // Special value - will trigger on frame callbacks
            println!("Layer {} set to compositor-driven framerate", self.name);
        } else if framerate == 0 {
            // If framerate is 0, only redraw on demand (never automatically)
            self.frames_per_update = u32::MAX;
            println!(
                "Layer {} set to static mode (no automatic updates)",
                self.name
            );
        } else if framerate >= SYSTEM_FPS as i32 {
            // If framerate is >= system rate, redraw every frame
            self.frames_per_update = 1;
            println!(
                "Layer {} set to {} FPS (redraw every frame)",
                self.name, framerate
            );
        } else {
            // Calculate how many system frames should pass before we redraw
            // For example: system fps = 60, wallpaper framerate = 30 => redraw every 2 frames
            self.frames_per_update = SYSTEM_FPS / framerate as u32;
            println!(
                "Layer {} set to {} FPS (redraw every {} frames)",
                self.name, framerate, self.frames_per_update
            );
        }
    }

    /// Set the ticks per update rate based on the wallpaper's tickrate
    /// This controls how often animations and logic are updated
    pub fn set_tickrate(&mut self, tickrate: i32) {
        // Default system update rate assumed to be 60 Hz
        const SYSTEM_TPS: u32 = 60;

        if tickrate < 0 {
            // Any negative value: Use compositor-driven timing for animation updates
            // This typically means update animations on every frame callback
            self.ticks_per_update = 0; // Special value - will update on each frame callback
            println!(
                "Layer {} set to compositor-driven animation rate",
                self.name
            );
        } else if tickrate == 0 {
            // If tickrate is 0, never update animations automatically
            self.ticks_per_update = u32::MAX;
            println!(
                "Layer {} set to static animation mode (no updates)",
                self.name
            );
        } else if tickrate >= SYSTEM_TPS as i32 {
            // If tickrate is >= system rate, update every frame
            self.ticks_per_update = 1;
            println!(
                "Layer {} set to {} TPS (update every frame)",
                self.name, tickrate
            );
        } else {
            // Calculate how many system frames should pass before we update animations
            // For example: system tps = 60, wallpaper tickrate = 15 => update every 4 frames
            self.ticks_per_update = SYSTEM_TPS / tickrate as u32;
            println!(
                "Layer {} set to {} TPS (update every {} frames)",
                self.name, tickrate, self.ticks_per_update
            );
        }
    }

    pub fn draw(&mut self, qh: &QueueHandle<Client>, device: &Device, queue: &Queue) {
        // Increment frame counter for rendering
        self.frame_counter = (self.frame_counter + 1) % 6000; // Avoid overflow, max ~1 minute at 100fps

        // Increment tick counter for animations
        self.tick_counter = (self.tick_counter + 1) % 6000; // Avoid overflow, max ~1 minute at 100fps

        // Handle special cases for compositor-driven timing (frames_per_update = 0)
        let should_redraw = if self.frames_per_update == 0 {
            // For compositor-driven timing, we'll decide on redraw through
            // the frame() callback from CompositorHandler instead of counter
            false
        } else {
            // Regular timing - check frame counter against update interval
            self.frame_counter % self.frames_per_update == 0
        };

        // Similarly for animation updates
        let update_animations = if self.ticks_per_update == 0 {
            // For compositor-driven animation updates
            true // Always update on frame callback
        } else if self.ticks_per_update == u32::MAX {
            // No animation updates
            false
        } else {
            // Regular timing - check tick counter
            self.tick_counter % self.ticks_per_update == 0
        };

        // Mark as damaged if we should redraw or if animations were updated
        if should_redraw || (update_animations && self.ticks_per_update < u32::MAX) {
            self.damaged = true;
        }

        if !self.damaged || self.wallpaper.is_empty() {
            return;
        }

        self.damaged = false;

        // Get a texture from the surface to render to
        let surface_texture = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                eprintln!("Failed to acquire next swapchain texture: {e:?}");
                return;
            }
        };

        // Create a view of the texture that we'll render to
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create a command encoder to record commands
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture Renderer Encoder"),
        });

        // Create the render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Texture Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Calculate real elapsed time since last animation update
            let now = Instant::now();
            let dt = now.duration_since(self.last_animation_update);

            // Update and render all pipeline objects
            for renderer in self.wallpaper.iter_mut() {
                // Update animated textures and other objects that need pre-render updates
                if update_animations {
                    // First call pre_render to do any necessary setup
                    renderer.pre_render(device, dt);

                    // Then, if this is an effect model, update time parameter
                    if let Some(effect) = renderer
                        .as_any()
                        .downcast_ref::<crate::renderer::models::effect::EffectModel>(
                    ) {
                        // Call the effect's update_time method if it's animated
                        if effect.is_animated() {
                            // Get and display the effect name more frequently
                            if effect.current_time < 0.5 || (effect.current_time % 5.0 < 0.1) {
                                println!("Rendering effect layer: {}", self.name);
                            }

                            // Here we need to use a mutable reference, so we'll have to downcast again
                            if let Some(effect_mut) = renderer
                                .as_any_mut()
                                .downcast_mut::<crate::renderer::models::effect::EffectModel>(
                            ) {
                                // Always update effect time to ensure animations work
                                // This ensures the shader gets time updates even if animations are disabled
                                effect_mut.update_time(dt, queue);
                                
                                // Force damage to ensure continuous redraw for wave effect debugging
                                if self.name.contains("effect-test") && self.frame_counter % 5 == 0 {
                                    self.damaged = true;
                                    println!("Forcing redraw for wave effect test");
                                }
                            }
                        }
                    }
                }

                render_pass.set_pipeline(&renderer.pipeline());
                render_pass.set_bind_group(0, Some(&*renderer.bindgroup()), &[]);
                render_pass.draw(0..6, 0..1); // Draw full-screen quad (6 vertices)
            }

            // Update the last animation time if animations were updated
            if update_animations {
                self.last_animation_update = now;
            }
        }

        // Submit the commands to the GPU queue
        queue.submit(Some(encoder.finish()));

        // Present the rendered image to the screen
        surface_texture.present();

        self.layer
            .wl_surface()
            .damage_buffer(0, 0, self.width as i32, self.height as i32);
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());
        self.layer.commit();
    }
}
