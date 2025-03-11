use bevy::{
    app::App,
    ecs::{
        entity::{Entity, EntityHashMap},
        query::Added,
        system::SystemState,
        world::FromWorld,
    },
    log::info,
    utils::{info, HashMap},
    window::{RawHandleWrapper, Window, WindowCreated, WindowWrapper},
};
use raw_window_handle::DisplayHandle;
use raw_window_handle::HasDisplayHandle;
use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawDisplayHandle;
use raw_window_handle::RawWindowHandle;
use raw_window_handle::WaylandDisplayHandle;
use raw_window_handle::WaylandWindowHandle;
use smithay_client_toolkit::compositor::CompositorHandler;
use smithay_client_toolkit::compositor::CompositorState;
use smithay_client_toolkit::delegate_registry;
use smithay_client_toolkit::delegate_seat;
use smithay_client_toolkit::delegate_shm;
use smithay_client_toolkit::registry::ProvidesRegistryState;
use smithay_client_toolkit::registry_handlers;
use smithay_client_toolkit::seat::Capability;
use smithay_client_toolkit::shell::wlr_layer::Anchor;
use smithay_client_toolkit::shell::wlr_layer::Layer;
use smithay_client_toolkit::shell::wlr_layer::LayerShell;
use smithay_client_toolkit::shell::wlr_layer::{
    LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
};
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shm::ShmHandler;
use smithay_client_toolkit::{delegate_compositor, delegate_layer, delegate_output};
use smithay_client_toolkit::{
    output::{OutputHandler, OutputState},
    registry::RegistryState,
    seat::{SeatHandler, SeatState},
    shm::Shm,
};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::LazyLock;
use std::sync::Mutex;
use wayland_client::protocol::{
    wl_output::{self, WlOutput},
    wl_seat, wl_surface,
};
use wayland_client::Proxy;
use wayland_client::{Connection, QueueHandle};

use super::{config::SctkLayerWindowConfig, config::WindowResolution, CreateWindowParams};

static IDS: LazyLock<Mutex<u32>> = LazyLock::new(|| Mutex::new(0));

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SctkLayerWindowID(u32);

impl Deref for SctkLayerWindowID {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for SctkLayerWindowID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Clone)]
pub struct SctkLayerWindow {
    id: SctkLayerWindowID,

    exit: bool,
    first_configure: bool,
    width: u32,
    height: u32,
    layer: LayerSurface,
}

pub struct SctkLayerWindowWrapped {
    layer: LayerSurface,
    connection: Connection,
}

pub struct State {
    pub app: App,
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub compositor: CompositorState,
    pub connection: Connection,
    pub shm: Shm,
    pub layer_shell: LayerShell,
    pub windows: HashMap<SctkLayerWindowID, SctkLayerWindow>,
    pub outputs: Vec<WlOutput>,
}

#[derive(Default)]
pub struct SctkLayerWindows {
    pub wrapped_windows: HashMap<SctkLayerWindowID, WindowWrapper<SctkLayerWindowWrapped>>,
    pub entity_to_sctk: EntityHashMap<SctkLayerWindowID>,
    pub sctk_to_entity: HashMap<SctkLayerWindowID, Entity>,

    marker: PhantomData<*const ()>,
}

impl SctkLayerWindows {
    pub fn get_window(&self, entity: Entity) -> Option<&SctkLayerWindowID> {
        self.entity_to_sctk.get(&entity)
    }

    pub fn get_entity(&self, id: SctkLayerWindowID) -> Option<Entity> {
        self.sctk_to_entity.get(&id).copied()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_window(
        &mut self,
        layer: Layer,
        entity: Entity,
        window: &Window,
        compositor: &CompositorState,
        layer_shell: &LayerShell,
        connection: &Connection,
        qh: &QueueHandle<State>,
    ) -> (SctkLayerWindow, &WindowWrapper<SctkLayerWindowWrapped>) {
        let mut id = IDS.lock().unwrap();
        let window_id = SctkLayerWindowID(*id);
        *id += 1;
        let surface = compositor.create_surface(qh);
        let layer = layer_shell.create_layer_surface(qh, surface, layer, Some("info.werdxz"), None);

        layer.set_anchor(Anchor::TOP | Anchor::LEFT);
        layer.set_size(window.width() as u32, window.height() as u32);
        layer.commit();
        let window = SctkLayerWindow {
            id: window_id,
            exit: false,
            first_configure: true,
            width: window.width() as u32,
            height: window.height() as u32,
            layer,
        };

        info!(
            "creating window with width: {}, height: {}",
            window.width, window.height
        );

        self.entity_to_sctk.insert(entity, window_id);
        self.sctk_to_entity.insert(window_id, entity);

        (
            window.clone(),
            self.wrapped_windows
                .entry(window_id)
                .insert(WindowWrapper::new(window.wrapped(connection.clone())))
                .into_mut(),
        )
    }
}

impl CompositorHandler for State {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        info("CompositorHandler::scale_factor_changed");
        // Not needed for this example.
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        info("CompositorHandler::transform_changed");
        // Not needed for this example.
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        info("CompositorHandler::frame");

        self.app.update();
        self.draw(qh);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        info("CompositorHandler::surface_enter");
        // Not needed for this example.
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        info("CompositorHandler::surface_leave");
        // Not needed for this example.
    }
}

impl ShmHandler for State {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl State {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        for window in self.windows.values_mut() {
            window.first_configure = false;
            window.draw(qh);
        }
    }

    pub fn new_window(&mut self, qh: &QueueHandle<Self>) {
        let world = self.app.world_mut();
        let resolution = world.resource::<SctkLayerWindowConfig>().resolution;
        let mut create_window = SystemState::<CreateWindowParams<Added<Window>>>::from_world(world);

        let (mut commands, mut created_windows, mut window_created_events, mut windows) =
            create_window.get_mut(world);

        for (entity, mut window, handle_holder) in &mut created_windows {
            info!("looping windows");
            if windows.get_window(entity).is_some() {
                continue;
            }
            info!("creating windows");

            if resolution == WindowResolution::MonitorSize {
                if let Some(info) = self.output_state.info(&self.outputs[0]) {
                    let mode = info.modes.iter().filter(|x| x.current).collect::<Vec<_>>()[0];
                    let (x, y) = mode.dimensions;

                    window.resolution.set(x as f32, y as f32);
                }
            }

            let (window, wrapped) = windows.new_window(
                Layer::Background,
                entity,
                &window,
                &self.compositor,
                &self.layer_shell,
                &self.connection,
                qh,
            );
            self.windows.insert(window.id(), window);

            if let Ok(handle_wrapper) = RawHandleWrapper::new(wrapped) {
                commands.entity(entity).insert(handle_wrapper.clone());
                if let Some(handle_holder) = handle_holder {
                    *handle_holder.0.lock().unwrap() = Some(handle_wrapper);
                }
            }

            window_created_events.send(WindowCreated { window: entity });
        }

        create_window.apply(self.app.world_mut());
    }
}

impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        info("OutputHandler::new_output");
        self.outputs.push(output);
        self.new_window(qh);
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        info("OutputHandler::update_output");
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        info("OutputHandler::output_destroyed");
    }
}

impl SeatHandler for State {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {
        info("SeatHandler::new_seat");
    }

    fn new_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: Capability,
    ) {
        info("SeatHandler::new_capability");
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _capability: Capability,
    ) {
        info("SeatHandler::remove_capability");
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl LayerShellHandler for State {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        info!("LayerShellHandler::closed");
        let mut windows = self
            .app
            .world_mut()
            .non_send_resource_mut::<SctkLayerWindows>();
        let id = {
            let window = self
                .windows
                .iter_mut()
                .find(|(_, window)| window.layer == *layer);

            if let Some((id, window)) = window {
                window.exit = true;
                windows.sctk_to_entity.remove(id);
                windows.wrapped_windows.remove(id);
                Some(*id)
            } else {
                None
            }
        };

        self.windows.remove(&id.unwrap());
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        info!("LayerShellHandler::configure");
        for window in self.windows.values_mut() {
            if window.layer != *layer {
                continue;
            };
            window.width = configure.new_size.0;
            window.height = configure.new_size.1;

            if window.first_configure {
                window.first_configure = false;
                window.draw(qh);
            }
        }
    }
}

impl SctkLayerWindow {
    pub fn draw(&mut self, qh: &QueueHandle<State>) {
        info!(
            "drawing window: {}; with width {}, height {}",
            self.id().0,
            self.width,
            self.height
        );
        let width = self.width;
        let height = self.height;

        // Damage the entire window
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        // Request our next frame
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        // In a real implementation, we would:
        // 1. Create a buffer from Bevy's render results
        // 2. Attach the buffer to the surface
        // 3. Commit to present
        
        // For now, we'll just commit the surface so it creates a blank window
        // that Bevy can render to
        self.layer.commit();
    }

    pub fn id(&self) -> SctkLayerWindowID {
        self.id
    }

    pub fn wrapped(&self, connection: Connection) -> SctkLayerWindowWrapped {
        SctkLayerWindowWrapped {
            layer: self.layer.clone(),
            connection,
        }
    }
}

impl HasWindowHandle for SctkLayerWindowWrapped {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let raw = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(self.layer.wl_surface().id().as_ptr() as *mut _).unwrap(),
        ));

        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }
}

impl HasDisplayHandle for SctkLayerWindowWrapped {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        let raw = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(self.connection.backend().display_ptr() as *mut _).unwrap(),
        ));

        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}

delegate_compositor!(State);
delegate_output!(State);
delegate_shm!(State);

delegate_seat!(State);
delegate_layer!(State);

delegate_registry!(State);

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers!(OutputState);
}
