use std::ptr::NonNull;

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

    config: OutputConfig,
    surface: Surface<'static>,
    pipeline: Option<RenderPipeline>,
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

    pub fn draw(&mut self, qh: &QueueHandle<Client>, device: &Device, queue: &Queue) {
        if !self.damaged || self.wallpaper.is_empty() {
            return;
        }

        self.damaged = false;

        // Get a texture from the surface to render to
        let surface_texture = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                eprintln!("Failed to acquire next swapchain texture: {:?}", e);
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

            // Render all pipeline objects
            for renderer in self.wallpaper.iter() {
                render_pass.set_pipeline(&renderer.pipeline());
                render_pass.set_bind_group(0, Some(&*renderer.bindgroup()), &[]);
                render_pass.draw(0..6, 0..1); // Draw full-screen quad (6 vertices)
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
