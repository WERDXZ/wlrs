use std::{thread::sleep_until, time::Instant};

use bevy::{
    app::PluginsState,
    prelude::*,
    window::{RawHandleWrapperHolder, WindowCreated},
};
use config::SctkLayerWindowConfig;
use layers::{SctkLayerWindows, State};
use smithay_client_toolkit::{
    compositor::CompositorState, output::OutputState, registry::RegistryState, seat::SeatState,
    shell::wlr_layer::LayerShell, shm::Shm,
};
use wayland_client::{globals::registry_queue_init, Connection};
pub mod config;
pub mod layers;
pub mod systems;

#[derive(Default, Debug)]
pub struct SctkPlugin;

impl Plugin for SctkPlugin {
    fn name(&self) -> &str {
        "wlrs::SctkPlugin"
    }

    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<SctkLayerWindows>()
            .init_resource::<SctkLayerWindowConfig>()
            .add_plugins(systems::background::BackgroundPlugin::default())
            .set_runner(runner);
    }
}

pub type CreateWindowParams<'w, 's, F = ()> = (
    Commands<'w, 's>,
    Query<
        'w,
        's,
        (
            Entity,
            &'static mut Window,
            Option<&'static RawHandleWrapperHolder>,
        ),
        F,
    >,
    EventWriter<'w, WindowCreated>,
    NonSendMut<'w, SctkLayerWindows>,
);

fn runner(mut app: App) -> AppExit {
    if app.plugins_state() == PluginsState::Ready {
        app.finish();
        app.cleanup();
    }

    let connection = Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) = registry_queue_init(&connection).unwrap();
    let qh = event_queue.handle();

    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer_shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let mut state = State {
        app,
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        compositor: CompositorState::bind(&globals, &qh).unwrap(),
        connection,
        shm,
        layer_shell,
        windows: Default::default(),
        outputs: Default::default(),
    };

    loop {
        let start = Instant::now();
        let pace = state
            .app
            .world_mut()
            .resource::<SctkLayerWindowConfig>()
            .pace;
        event_queue.blocking_dispatch(&mut state).unwrap();
        sleep_until(start + pace);
        if state.app.plugins_state() == PluginsState::Cleaned {
            state.app.update();
        }
        if state.windows.is_empty() {
            break;
        }
    }

    AppExit::Success
}
