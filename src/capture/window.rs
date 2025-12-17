use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use xcap::Window;

pub struct SafeWindows {
    window: HashMap<String, Arc<RwLock<Window>>>,
}

impl SafeWindows {
    pub fn new() -> Self {
        Self {
            window: HashMap::new(),
        }
    }
}
