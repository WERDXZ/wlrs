use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
    time::Duration,
};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_seat,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState},
    shell::wlr_layer::{Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{
        wl_output::{self, WlOutput},
        wl_seat, wl_surface,
    },
    Connection, EventQueue, QueueHandle,
};
use wgpu::{Adapter, BindGroupLayout, Device, Instance, Queue, RenderPipeline};

use super::{manager::Manager, wallpaper_layer::WallpaperLayer};

pub struct Client {
    pub namespace: Option<String>,

    pub compositor: CompositorState,
    pub layer: LayerShell,
    pub registry: RegistryState,
    pub seat: SeatState,
    pub output: OutputState,

    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,

    pub bindgroup_layout_manager: Arc<Mutex<Manager<BindGroupLayout>>>,
    pub pipeline_manager: Arc<Mutex<Manager<RenderPipeline>>>,

    pub wallpapers: Wallpapers,
}

#[derive(Default)]
pub struct Wallpapers(pub Vec<WallpaperLayer>);

impl Deref for Wallpapers {
    type Target = Vec<WallpaperLayer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Wallpapers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Client {
    pub fn new_layer(&self, qh: &QueueHandle<Self>, output: &WlOutput) -> LayerSurface {
        let surface = self.compositor.create_surface(qh);
        self.layer.create_layer_surface(
            qh,
            surface,
            Layer::Background,
            self.namespace.as_ref(),
            Some(output),
        )
    }

    pub fn new(namespace: Option<impl Into<String>>) -> (Self, EventQueue<Self>) {
        let connection = Connection::connect_to_env().unwrap();
        let (globals, event_queue) = registry_queue_init(&connection).unwrap();
        let qh = event_queue.handle();

        let compositor = CompositorState::bind(&globals, &qh).expect("No compositor available");
        let layer = LayerShell::bind(&globals, &qh).expect("No layer shell available");
        let registry = RegistryState::new(&globals);
        let seat = SeatState::new(&globals, &qh);
        let output = OutputState::new(&globals, &qh);

        let instance = Instance::default();
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("Failed to find suitable adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None))
            .expect("Failed to request device");
        let wallpapers = Wallpapers::default();

        (
            Self {
                namespace: namespace.map(Into::into),
                compositor,
                layer,
                registry,
                seat,
                output,
                instance,
                adapter,
                device,
                queue,
                bindgroup_layout_manager: Arc::new(Mutex::new(Manager::new())),
                pipeline_manager: Arc::new(Mutex::new(Manager::new())),
                wallpapers,
            },
            event_queue,
        )
    }

    pub fn get_recommended_update_interval(&self) -> Option<Duration> {
        self.wallpapers
            .iter()
            .filter_map(|v| v.get_recommended_update_interval())
            .max()
    }

    pub fn request_update(&mut self, qh: &QueueHandle<Self>) {
        self.wallpapers.iter_mut().for_each(|v| {
            v.request_compositor_update(qh);
        });
    }
}

impl CompositorHandler for Client {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // Draw all wallpapers that need updating
        self.wallpapers
            .iter_mut()
            .for_each(|v| WallpaperLayer::draw(v, qh, &self.device, &self.queue));
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl ProvidesRegistryState for Client {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers![OutputState, SeatState];
}

impl LayerShellHandler for Client {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        self.wallpapers.retain(|v| v.layer != *layer);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        println!("Received configure request");
        // iter_mut().find(|wallpaper| wallpaper.layer == *layer)
        if let Some(v) = self
            .wallpapers
            .iter_mut()
            .find(|wallpaper| wallpaper.layer == *layer)
        {
            println!(
                "Received configure layer {} with new size: {:?}",
                v.name, configure.new_size
            );
            v.set_size(configure.new_size.0, configure.new_size.1);
            if !v.configured {
                println!("Configuring layer: {}", v.name);
                v.configure(&self.adapter, &self.device);
                v.draw(qh, &self.device, &self.queue);
            }
        };
    }
}

impl SeatHandler for Client {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: Capability,
    ) {
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: Capability,
    ) {
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl OutputHandler for Client {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output
    }

    fn new_output(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        println!("Accepted new output: {output:?}");
        let wallpaper = WallpaperLayer::new(self, conn, qh, &output);
        self.wallpapers.push(wallpaper);
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

delegate_compositor!(Client);
delegate_layer!(Client);
delegate_registry!(Client);
delegate_seat!(Client);
delegate_output!(Client);
