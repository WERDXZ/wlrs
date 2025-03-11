use std::time::Duration;

use bevy::ecs::system::Resource;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum WindowResolution {
    Default,
    MonitorSize,
}

#[derive(Debug, Resource, Clone)]
pub struct SctkLayerWindowConfig {
    pub pace: Duration,
    pub resolution: WindowResolution,
}

impl SctkLayerWindowConfig {
    pub fn high() -> Self {
        Self {
            pace: Duration::from_secs_f64(1.0 / 60.0), // 60 Hz
            resolution: WindowResolution::MonitorSize,
        }
    }
    pub fn medium() -> Self {
        Self {
            pace: Duration::from_secs_f64(1.0 / 30.0), // 30 Hz
            resolution: WindowResolution::MonitorSize,
        }
    }
    pub fn low() -> Self {
        Self {
            pace: Duration::from_secs_f64(1.0 / 15.0), // 15 Hz
            resolution: WindowResolution::MonitorSize,
        }
    }
}

impl Default for SctkLayerWindowConfig {
    fn default() -> Self {
        Self::medium()
    }
}
