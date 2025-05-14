use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub pace: Duration,
}

impl OutputConfig {
    pub fn high() -> Self {
        Self {
            pace: Duration::from_secs_f64(1.0 / 60.0), // 60 Hz
        }
    }
    pub fn medium() -> Self {
        Self {
            pace: Duration::from_secs_f64(1.0 / 30.0), // 30 Hz
        }
    }
    pub fn low() -> Self {
        Self {
            pace: Duration::from_secs_f64(1.0 / 15.0), // 15 Hz
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self::medium()
    }
}
