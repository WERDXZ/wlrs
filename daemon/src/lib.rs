#![feature(thread_sleep_until)]

use std::sync::{LazyLock, Mutex};

pub mod asset;
pub mod renderer;
pub mod shaders;
pub mod utils;

pub static EXIT: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
