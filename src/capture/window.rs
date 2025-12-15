use anyhow::{Result, anyhow};
use std::sync::{Arc, LazyLock, RwLock};
use xcap::Window;

pub struct SafeWindows {
    window: HashMap<String, Arc<RwLock<Window>>>,
}

impl WindowCapture {
    pub fn new(window: Window) -> Self {
        Self { window }
    }
}
